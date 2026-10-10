-- Thin client: reference -> interpret -> revise -> review -> apply.
local M = {}
local engine = require("hexwitch.engine")
local session = require("hexwitch.session")
local ui = require("hexwitch.ui")

local function complete(cb, err, value)
  session.busy = false
  if err then
    ui.notify(err, vim.log.levels.ERROR)
  end
  if cb then
    cb(err, value)
  end
end

local function guard(cb)
  if session.busy then
    local err = "Hexwitch is already processing a request"
    ui.notify(err, vim.log.levels.WARN)
    if cb then cb(err) end
    return false
  end
  return true
end

local function absolute(path)
  return vim.fn.fnamemodify(vim.fn.expand(path), ":p")
end

local function read(path)
  local file, err = io.open(path, "rb")
  if not file then return nil, err end
  local text = file:read("*a")
  file:close()
  return text
end

local function write(path, text)
  local file, err = io.open(path, "wb")
  if not file then return nil, err end
  local ok, why = file:write(text)
  file:close()
  return ok, why
end

local function version(dir, prompt, parent)
  return {
    id = #session.versions + 1, theme = dir .. "/theme.json", lua = dir .. "/theme.lua",
    spec = dir .. "/spec.json", report_path = dir .. "/report.json",
    prompt = prompt, parent = parent and parent.id or nil,
  }
end

function M.setup(opts)
  if vim.fn.has("nvim-0.10") ~= 1 then
    error("hexwitch requires Neovim 0.10+ (vim.system)")
  end
  require("hexwitch.config").setup(opts)
end

-- Loading a reference applies it and starts a new history branch. The original
-- editor is retained so undoing the first version can restore it exactly.
function M.reference(path, cb)
  if not guard(cb) then return end
  if session.pending then
    return complete(cb, "Apply or discard the pending candidate before switching references")
  end
  local source, err = read(absolute(path))
  if not source then return complete(cb, "Cannot read reference: " .. tostring(err)) end
  local v = version(session.directory(), "Reference: " .. path)
  local ok, why = write(v.theme, source)
  if not ok then return complete(cb, "Cannot save reference: " .. tostring(why)) end
  session.busy = true
  return engine.emit(v.theme, v.lua, function(e, _, stderr)
    if e then return complete(cb, e) end
    v.report = stderr
    local loaded, load_err = session.accept(v, true)
    if not loaded then return complete(cb, "Cannot load emitted theme: " .. load_err) end
    table.insert(session.versions, v)
    ui.notify("Reference loaded. Use :Hexwitch to refine it.")
    complete(cb, nil, v)
  end)
end

local function prepare(v, parent, cb)
  return engine.revise(parent.theme, v.spec, v.theme, v.lua, v.report_path, function(err, _, stderr)
    if err then return complete(cb, err) end
    local raw, read_err = read(v.report_path)
    if not raw then return complete(cb, "Cannot read revision report: " .. tostring(read_err)) end
    local ok, report = pcall(vim.json.decode, raw)
    if not ok or type(report) ~= "table" or type(report.failed) ~= "table"
        or type(report.refused) ~= "table" then
      return complete(cb, "Invalid machine-readable revision report")
    end
    v.revision, v.report = report, stderr
    v.has_issues = #report.failed > 0 or #report.refused > 0
    table.insert(session.versions, v)
    session.pending = v
    session.busy = false
    if require("hexwitch.config").get().auto_apply and not v.has_issues then
      local accepted, e = session.accept(v)
      if not accepted then return complete(cb, e) end
      ui.notify("Revision applied; :HexwitchUndo restores the previous theme.")
    else
      local message = "Candidate " .. v.id .. " ready: :HexwitchStatus to review, :HexwitchApply to accept."
      if v.has_issues then message = message .. " Some edits were refused or failed." end
      ui.notify(message, v.has_issues and vim.log.levels.WARN or nil)
    end
    if require("hexwitch.config").get().report == "full" then
      ui.notify(stderr)
    end
    if cb then cb(nil, v) end
  end)
end

-- opts.from_llm is the offline replay path: the real engine validates a saved
-- model response instead of making a provider call. opts.spec accepts an
-- already-reviewed EditSpec. Neither bypasses revision or emission.
function M.refine(prompt, opts, cb)
  opts = opts or {}
  if not guard(cb) then return end
  if session.pending then
    return complete(cb, "Apply or discard the pending candidate before refining again")
  end
  local parent = session.active()
  if not parent then return complete(cb, "Load a reference with :HexwitchReference <theme.json> first") end
  if not opts.spec and not opts.from_llm and (type(prompt) ~= "string" or not prompt:find("%S")) then
    return complete(cb, "A non-empty refinement prompt is required")
  end
  local v = version(session.directory(), prompt or "Edit spec", parent)
  session.busy = true
  if opts.spec then
    local raw, e = read(absolute(opts.spec))
    if not raw then return complete(cb, "Cannot read edit spec: " .. tostring(e)) end
    local ok, why = write(v.spec, raw)
    if not ok then return complete(cb, "Cannot save edit spec: " .. tostring(why)) end
    return prepare(v, parent, cb)
  end
  local interpret_opts = opts.from_llm and { from_llm = absolute(opts.from_llm) } or nil
  return engine.interpret(parent.theme, prompt, v.spec, function(err, _, stderr)
    if err then return complete(cb, err) end
    v.notes = stderr
    return prepare(v, parent, cb)
  end, interpret_opts)
end

function M.prompt()
  if not guard() then return end
  vim.ui.input({ prompt = "Hexwitch refinement: " }, function(input)
    if input and input:find("%S") then M.refine(input) end
  end)
end

function M.apply(id)
  if not guard() then return end
  local v = id and session.versions[tonumber(id)] or session.pending
  if not v then return complete(nil, "No candidate to apply") end
  if v.has_issues then
    ui.notify("Accepting a partial revision: inspect :HexwitchStatus for refused/failed edits", vim.log.levels.WARN)
  end
  local ok, err = session.accept(v)
  if not ok then return complete(nil, err) end
  ui.notify("Applied version " .. v.id)
  return v
end

function M.discard()
  if not guard() then return end
  session.pending = nil
  ui.notify("Candidate discarded; current theme unchanged.")
end

function M.undo()
  if not guard() then return end
  if session.pending then return complete(nil, "Discard or apply the pending candidate first") end
  if session.cursor == 0 then return complete(nil, "Nothing to undo") end
  local next_cursor = session.cursor - 1
  if next_cursor == 0 then
    session.restore(session.original)
  else
    local ok, err = session.load(session.versions[session.history[next_cursor]])
    if not ok then return complete(nil, err) end
  end
  session.cursor = next_cursor
  return true
end

function M.redo()
  if not guard() then return end
  if session.pending then return complete(nil, "Discard or apply the pending candidate first") end
  local id = session.history[session.cursor + 1]
  if not id then return complete(nil, "Nothing to redo") end
  local ok, err = session.load(session.versions[id])
  if not ok then return complete(nil, err) end
  session.cursor = session.cursor + 1
  return true
end

function M.state()
  return vim.deepcopy({ versions = session.versions, history = session.history,
    cursor = session.cursor, pending = session.pending, busy = session.busy })
end

function M.status()
  local lines = { "Hexwitch " .. (session.busy and "(processing)" or "(idle)"), "" }
  local active = session.active()
  for _, v in ipairs(session.versions) do
    local label = active == v and "active" or (session.pending == v and "pending" or "saved")
    table.insert(lines, string.format("%d [%s] %s", v.id, label, v.prompt))
  end
  local v = session.pending or active
  if v then
    table.insert(lines, "\n" .. (v.notes or "") .. (v.report or ""))
    if v.spec and vim.fn.filereadable(v.spec) == 1 then
      table.insert(lines, "Edit spec:\n" .. (read(v.spec) or ""))
    end
    table.insert(lines, "\nArtifact: " .. v.lua)
  end
  table.insert(lines, "\n:HexwitchApply [id]  :HexwitchDiscard  :HexwitchUndo  :HexwitchRedo")
  ui.show("Hexwitch status", table.concat(lines, "\n"))
end

function M.analyze()
  if not guard() then return end
  local v = session.pending or session.active()
  if not v then return complete(nil, "Load a reference first") end
  session.busy = true
  return engine.analyze(v.theme, function(err, stdout)
    complete(nil, err)
    if not err then ui.show("Hexwitch analysis", stdout) end
  end)
end

return M
