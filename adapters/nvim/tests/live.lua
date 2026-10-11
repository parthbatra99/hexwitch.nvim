-- Opt-in live smoke: nvim --clean --headless -l adapters/nvim/tests/live.lua
-- Only this adapter + engine + shipped fixture. No personal config/code is sent.
local file = debug.getinfo(1, "S").source:gsub("^@", "")
local root = vim.fn.fnamemodify(file, ":p:h:h:h:h")
vim.opt.rtp:prepend(root .. "/adapters/nvim")
vim.cmd("runtime plugin/hexwitch.lua")
local hx = require("hexwitch")
local session = require("hexwitch.session")
local function wait_for(start)
  local done, err, value = false, nil, nil
  start(function(e, v) err, value, done = e, v, true end)
  assert(vim.wait(150000, function() return done end, 20), "live operation timed out")
  assert(not err, tostring(err))
  return value
end
local function hl(name) return vim.api.nvim_get_hl(0, { name = name, link = false }) end
local ok, err = xpcall(function()
  local key_env = vim.env.HEXWITCH_LIVE_KEY_ENV or "OPENAI_API_KEY"
  assert(vim.env[key_env] and vim.env[key_env] ~= "", key_env .. " is not set; no request made")
  hx.setup({
    cmd = vim.env.HEXWITCH_BINARY or root .. "/core/target/release/hexwitch",
    base_url = assert(vim.env.HEXWITCH_LIVE_BASE_URL, "HEXWITCH_LIVE_BASE_URL required"),
    api_key_env = key_env,
    model = assert(vim.env.HEXWITCH_LIVE_MODEL, "HEXWITCH_LIVE_MODEL required"),
    response_format = vim.env.HEXWITCH_LIVE_FORMAT or "json_schema",
    auto_apply = false,
    interpretation = "v1",
  })
  assert(require("hexwitch.engine").find(), "engine binary not found")
  session.root = assert(vim.env.HEXWITCH_LIVE_ARTIFACTS, "private artifact directory required")
  local original = session.snapshot()
  wait_for(function(cb) hx.reference(root .. "/core/eval/themes/gruvbox-dark-medium.json", cb) end)
  assert(hl("Comment").italic == true, "reference didn't load italic comments")
  local normal_before, comment_before = hl("Normal"), hl("Comment")
  print("PASS isolated reference import/emission")
  local request = "No italics anywhere."
  local version = wait_for(function(cb) hx.refine(request, {}, cb) end)
  assert(version and version.id and session.pending == version, "live request did not produce a candidate")
  assert(not version.has_issues, "live revision contains refused or infeasible edits")
  assert(vim.deep_equal(hl("Comment"), comment_before), "candidate applied before review")
  hx.status()
  vim.cmd("close")
  assert(hx.apply(), "candidate apply failed")
  assert(hl("Comment").italic ~= true, "live interpretation did not remove comment italics")
  assert(hl("Normal").fg == normal_before.fg and hl("Normal").bg == normal_before.bg, "live revision moved base colors")
  assert(hl("Comment").fg == comment_before.fg, "live revision moved comment color")
  assert(hx.undo(), "undo failed")
  assert(vim.deep_equal(hl("Comment"), comment_before), "undo did not restore reference")
  assert(hx.redo(), "redo failed")
  assert(hl("Comment").italic ~= true, "redo failed")
  print("PASS live interpret -> review -> apply -> undo -> redo")
  if vim.env.HEXWITCH_LIVE_EXTENDED == "1" then
    local readable = wait_for(function(cb)
      hx.refine("Keep the background color and make comments easier to read.", {}, cb)
    end)
    assert(readable.id and not readable.has_issues, "readability request did not yield a satisfied candidate")
    assert(hx.apply())
    local active = session.active()
    local file = assert(io.open(active.theme, "rb"))
    local ir = vim.json.decode(file:read("*a")); file:close()
    assert(ir.styles["surface.base"].bg == "#282828", "readability request changed background")
    local imported_res = vim.system({ require("hexwitch.engine").find(), "--machine", "import", root .. "/core/eval/themes/gruvbox-dark-medium.json" }, { text = true }):wait()
    assert(imported_res.code == 0)
    local imported = vim.json.decode(imported_res.stdout).result.theme
    for role, style in pairs(imported.styles) do
      if role:match("^syntax%.") and role ~= "syntax.comment" then
        assert(ir.styles[role].fg == style.fg and ir.styles[role].bg == style.bg, "readability request changed unrelated syntax: " .. role)
      end
    end
    local fg = assert(hl("Comment").fg)
    local color = string.format("#%06x", fg)
    local res = vim.system({ require("hexwitch.engine").find(), "contrast", color, "#282828", "--json" }, { text = true }):wait()
    assert(res.code == 0 and vim.json.decode(res.stdout).ratio >= 4.5, "emitted comment contrast below 4.5")
    print("PASS live readability and protected background")
    local before = #session.versions
    local clarification = wait_for(function(cb)
      hx.refine("Make that blue quieter. I have not identified which role or color I mean.", { v1 = true }, cb)
    end)
    assert(clarification.decision == "needs_clarification", "ambiguous live request did not clarify")
    assert(not session.pending and #session.versions == before, "clarification created a candidate")
    print("PASS live clarification without mutation")
    hx.undo()
  end
  hx.undo()
  hx.undo()
  assert(vim.deep_equal(hl("Normal"), original.highlights.Normal), "original highlight state not restored")
end, debug.traceback)
if not ok then io.stderr:write(tostring(err) .. "\n"); vim.cmd("cquit 1") end
vim.cmd("qa!")
