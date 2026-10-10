-- Standalone headless suite. No Plenary, Telescope, API keys or user config.
local root = assert(vim.env.HEXWITCH_TEST_ROOT)
vim.opt.rtp:prepend(root .. "/adapters/nvim")
vim.cmd("runtime plugin/hexwitch.lua")
local hx = require("hexwitch")
local engine = require("hexwitch.engine")
local session = require("hexwitch.session")
local tmp = vim.fn.tempname() .. " space ' reference"
vim.fn.mkdir(tmp, "p")
local fixtures = root .. "/core/eval/"
local real_exec = engine.exec
local passed = 0
vim.notify = function() end

local function eq(a, b, message)
  assert(vim.deep_equal(a, b), (message or "not equal") .. "\nexpected " .. vim.inspect(b) .. "\ngot " .. vim.inspect(a))
end
local function async(start)
  local done, err, value = false
  start(function(e, v) err, value, done = e, v, true end)
  assert(vim.wait(15000, function() return done end, 10), "async operation timed out")
  return err, value
end
local function success(start)
  local err, value = async(start)
  assert(not err, tostring(err))
  return value
end
local function fg(name) return vim.api.nvim_get_hl(0, { name = name, link = false }).fg end
local function reset(opts)
  engine.exec = real_exec
  session.versions, session.history, session.cursor = {}, {}, 0
  session.pending, session.original, session.busy, session.clarification = nil, nil, false, nil
  session.root = tmp .. "/session-" .. passed
  vim.fn.mkdir(session.root, "p")
  hx.setup(vim.tbl_extend("force", { cmd = vim.env.HEXWITCH_BINARY }, opts or {}))
end
local function test(name, fn)
  reset()
  fn()
  passed = passed + 1
  print("PASS " .. name)
end
local function reference()
  return success(function(cb) hx.reference(fixtures .. "themes/gruvbox-dark-medium.json", cb) end)
end
local function specfile(name, spec)
  local path = tmp .. "/" .. name .. ".json"
  vim.fn.writefile({ vim.json.encode(spec) }, path)
  return path
end
local function revise_spec(path)
  return success(function(cb) hx.refine("Reviewed spec", { spec = path }, cb) end)
end

local ok, err = xpcall(function()
  test("config and commands", function()
    for _, opt in ipairs({ { cmd = "" }, { auto_apply = "yes" }, { provider = "wrong" }, { model = false } }) do
      assert(not pcall(hx.setup, opt), "invalid config accepted")
    end
    hx.setup({ cmd = vim.env.HEXWITCH_BINARY })
    for _, name in ipairs({ "Hexwitch", "HexwitchReference", "HexwitchApply", "HexwitchReplay",
      "HexwitchSpec", "HexwitchUndo", "HexwitchRedo", "HexwitchStatus", "HexwitchAnalyze" }) do
      eq(vim.fn.exists(":" .. name), 2, "missing command " .. name)
    end
  end)

  test("real process preserves stdout and stderr", function()
    success(function(cb)
      engine.exec({ "convert", "#fb4934" }, function(e, stdout)
        if not e then assert(stdout:find("oklch", 1, true)) end
        cb(e)
      end)
    end)
    local dir = session.directory()
    success(function(cb)
      engine.emit(fixtures .. "themes/gruvbox-dark-medium.json", dir .. "/theme.lua", function(e, _, stderr)
        if not e then assert(stderr:find("emit report", 1, true), "stderr was lost") end
        cb(e)
      end)
    end)
  end)

  test("paths are argument data; reference copied and original undoable", function()
    local source = tmp .. "/theme '$ not shell.json"
    vim.fn.writefile(vim.fn.readfile(fixtures .. "themes/gruvbox-dark-medium.json"), source)
    vim.api.nvim_set_hl(0, "Normal", { fg = "#123456", bg = "#101010" })
    vim.g.colors_name, vim.g.terminal_color_1 = "original", "#112233"
    local before = session.snapshot()
    local v = success(function(cb) hx.reference(source, cb) end)
    eq(fg("Comment"), tonumber("928374", 16))
    assert(v.theme ~= source)
    vim.fn.writefile({ "{}" }, source)
    assert(vim.fn.readfile(v.theme)[1] ~= "{}", "reference wasn't copied")
    assert(hx.undo())
    eq(fg("Normal"), tonumber("123456", 16))
    eq(vim.g.colors_name, "original")
    eq(vim.g.terminal_color_1, "#112233")
    assert(hx.redo())
    eq(fg("Comment"), tonumber("928374", 16))
    assert(before.highlights.Normal)
  end)

  test("offline interpret -> candidate -> apply -> undo -> redo", function()
    local parent = reference()
    local oldbg = vim.api.nvim_get_hl(0, { name = "Normal", link = false }).bg
    local v = success(function(cb)
      hx.refine("darker, readable, keep syntax", {
        from_llm = fixtures .. "llm/darker-readable-comments.json",
      }, cb)
    end)
    eq(v.parent, parent.id)
    eq(hx.state().pending.id, v.id)
    assert(v.notes:find("notes:", 1, true), "interpretation notes dropped")
    assert(v.report:find("revision:", 1, true), "revision report dropped")
    eq(vim.api.nvim_get_hl(0, { name = "Normal", link = false }).bg, oldbg, "candidate auto-applied")
    assert(not hx.apply("not-a-version"))
    assert(not hx.apply(parent.id), "saved-version switch discarded pending candidate")
    eq(hx.state().pending.id, v.id)
    assert(hx.apply())
    eq(vim.api.nvim_get_hl(0, { name = "Normal", link = false }).bg, tonumber("1e1e1e", 16))
    eq(fg("Keyword"), tonumber("fb4934", 16))
    assert(hx.undo())
    eq(vim.api.nvim_get_hl(0, { name = "Normal", link = false }).bg, oldbg)
    assert(hx.redo())
    eq(vim.api.nvim_get_hl(0, { name = "Normal", link = false }).bg, tonumber("1e1e1e", 16))
  end)

  test("iterating replays protection against revised IR", function()
    reference()
    local opts = { from_llm = fixtures .. "llm/darker-readable-comments.json" }
    success(function(cb) hx.refine("first", opts, cb) end)
    hx.apply()
    local second = success(function(cb) hx.refine("second", opts, cb) end)
    eq(second.parent, 2)
    eq(#second.revision.failed, 0)
    eq(#second.revision.refused, 0)
    hx.apply()
    eq(fg("Keyword"), tonumber("fb4934", 16))
    eq(hx.state().cursor, 3)
  end)

  test("auto-apply and partial revision review", function()
    hx.setup({ cmd = vim.env.HEXWITCH_BINARY, auto_apply = true })
    reference()
    local path = specfile("partial", { protect = { "syntax.keyword" }, changes = {
      { target = "syntax.keyword", op = "lighter", amount = 0.1 },
      { target = "syntax.comment", op = "min_contrast", against = "surface.base", min = 21 },
    } })
    local v = revise_spec(path)
    eq(#v.revision.refused, 1)
    eq(#v.revision.failed, 1)
    eq(hx.state().cursor, 1, "partial revision auto-applied")
    assert(v.has_issues)
    hx.discard()
    local clean = specfile("clean", { changes = { { target = "surface.base", op = "darker", amount = 0.01 } } })
    revise_spec(clean)
    eq(hx.state().cursor, 2, "clean auto-apply failed")
    eq(hx.state().pending, nil)
  end)

  test("property-aware replay applies independent edits and handles clarification", function()
    reference()
    local oldbg = vim.api.nvim_get_hl(0, { name = "Normal", link = false }).bg
    local v = success(function(cb)
      hx.refine("Keep the background color, but remove its transparency.", {
        v1 = true, from_llm = fixtures .. "llm/property-opaque-background.json",
      }, cb)
    end)
    assert(v.v1 and v.revision.properties)
    assert(hx.apply())
    eq(vim.api.nvim_get_hl(0, { name = "Normal", link = false }).bg, oldbg)
    local before = hx.state()
    local clarification = success(function(cb)
      hx.refine("Make that blue quieter.", { v1 = true, from_llm = fixtures .. "llm/clarify-blue.json" }, cb)
    end)
    eq(clarification.decision, "needs_clarification")
    eq(hx.state().pending, nil)
    eq(#hx.state().versions, #before.versions)
    eq(hx.state().busy, false)
    eq(#hx.state().clarification, 1)
    local path = specfile("v1-rollback", { version = 1, actions = {
      { op = "set_color", target = { roles = { "syntax.comment" }, property = "fg" }, color = "#282828" },
    }, requirements = {
      { role = "syntax.comment", property = "fg", backgrounds = { { surface = "surface.base", overlays = {} } }, min = 4.5 },
    } })
    local err = async(function(cb) hx.refine("Reviewed invalid floor", { v1 = true, spec = path }, cb) end)
    assert(err and err:find("rolled back", 1, true))
    eq(hx.state().pending, nil)
    eq(#hx.state().versions, #before.versions)
  end)

  test("invalid model replay never creates a candidate", function()
    reference()
    local before = hx.state()
    local path = specfile("invalid-model", { actions = {
      { action = "recolor", target = "syntax.comment" },
    } })
    local e = async(function(cb) hx.refine("invalid replay", { from_llm = path }, cb) end)
    assert(e and e:find("unknown action", 1, true))
    eq(hx.state().history, before.history)
    eq(#hx.state().versions, #before.versions)
    eq(hx.state().pending, nil)
    eq(hx.state().busy, false)
  end)

  test("failure preserves editor and releases busy state", function()
    reference()
    local oldfg, oldhistory = fg("Keyword"), hx.state().history
    local path = specfile("bad", { changes = { { target = "syntax.typo", op = "lighter", amount = 0.1 } } })
    local e = async(function(cb) hx.refine("bad", { spec = path }, cb) end)
    assert(e, "invalid spec should fail")
    eq(fg("Keyword"), oldfg)
    eq(hx.state().history, oldhistory)
    eq(hx.state().busy, false)
    eq(hx.state().pending, nil)
    local e2 = async(function(cb) hx.reference(tmp .. "/missing.json", cb) end)
    assert(e2)
    eq(fg("Keyword"), oldfg)
  end)

  test("missing binary and spawn exceptions complete exactly once", function()
    hx.setup({ cmd = tmp .. "/no-binary" })
    local e = async(function(cb) engine.exec({ "--version" }, cb) end)
    assert(e:find("binary not found", 1, true))
    hx.setup({ cmd = vim.env.HEXWITCH_BINARY })
    local system = vim.system
    vim.system = function() error("spawn blocked") end
    local e2 = async(function(cb) engine.exec({ "--version" }, cb) end)
    vim.system = system
    assert(e2:find("cannot start", 1, true))
  end)

  test("overlapping requests and stale pending candidates blocked", function()
    reference()
    local resume
    engine.exec = function(_, cb) resume = cb end
    hx.refine("waiting")
    assert(hx.state().busy)
    local e = async(function(cb) hx.refine("overlap", {}, cb) end)
    assert(e:find("already processing", 1, true))
    assert(hx.state().busy, "overlap released the active request")
    resume("simulated failure", "", "")
    eq(hx.state().busy, false)
    engine.exec = real_exec
    revise_spec(specfile("pending", { changes = {} }))
    local blocked = async(function(cb) hx.refine("next", {}, cb) end)
    assert(blocked:find("pending", 1, true))
    hx.discard()
    eq(hx.state().pending, nil)
  end)

  test("branching after undo clears redo without deleting artifacts", function()
    reference()
    local path = specfile("branch", { changes = { { target = "surface.base", op = "darker", amount = 0.01 } } })
    local a = revise_spec(path)
    hx.apply()
    hx.undo()
    local b = revise_spec(path)
    hx.apply()
    eq(hx.state().history, { 1, b.id })
    eq(hx.state().versions[a.id].id, a.id)
    eq(hx.redo(), nil)
  end)

  test("leading-hyphen prompt remains one argument", function()
    local argv
    engine.exec = function(args, cb) argv = args; cb(nil, "", "notes: replay") end
    engine.interpret("theme.json", "--keep blue; 'quote' $(no)", "spec.json", function(e) assert(not e) end)
    eq(argv[#argv - 1], "--")
    eq(argv[#argv], "--keep blue; 'quote' $(no)")
  end)

  test("bad artifact rolls back partial changes", function()
    reference()
    local before = fg("Keyword")
    local path = tmp .. "/bad.lua"
    vim.fn.writefile({ "vim.api.nvim_set_hl(0, 'Keyword', { fg = '#ffffff' })", "error('invalid artifact')" }, path)
    local loaded, why = session.load({ lua = path })
    assert(not loaded and why:find("invalid artifact", 1, true))
    eq(fg("Keyword"), before)
  end)

  test("status and analysis are usable without third-party UI", function()
    reference()
    hx.status()
    assert(vim.api.nvim_get_current_line():find("Hexwitch", 1, true))
    vim.cmd("close")
    hx.analyze()
    assert(vim.wait(15000, function() return not hx.state().busy end, 10))
    assert(vim.api.nvim_get_current_line():find("Gruvbox", 1, true))
    vim.cmd("close")
  end)
end, debug.traceback)
engine.exec = real_exec
-- Only remove the directory created by this suite.
vim.fn.delete(tmp, "rf")
if not ok then
  io.stderr:write(tostring(err) .. "\n")
  vim.cmd("cquit 1")
else
  print(string.format("%d adapter tests passed", passed))
  vim.cmd("qa!")
end
