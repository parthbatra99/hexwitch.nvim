-- Argument-list process calls only: no shell quoting, color math or API keys here.
local M = {}

function M.find()
  local cmd = require("hexwitch.config").get().cmd
  return vim.fn.executable(cmd) == 1 and cmd or nil
end

-- cb(err, stdout, stderr); callbacks always run on Neovim's main loop.
function M.exec(args, cb)
  local cmd = M.find()
  if not cmd then
    vim.schedule(function()
      cb("hexwitch binary not found; build core/ and set setup({ cmd = '/path/to/hexwitch' })", "", "")
    end)
    return
  end
  local argv = { cmd }
  vim.list_extend(argv, args)
  local ok, process = pcall(vim.system, argv, { text = true, timeout = 260000 }, function(res)
    vim.schedule(function()
      local stdout, stderr = res.stdout or "", res.stderr or ""
      if res.code ~= 0 then
        cb(string.format("engine exited %d: %s", res.code, stderr:gsub("%s+$", "")), stdout, stderr)
      else
        cb(nil, stdout, stderr)
      end
    end)
  end)
  if not ok then
    vim.schedule(function()
      cb("cannot start engine: " .. tostring(process), "", "")
    end)
    return
  end
  return process
end

function M.interpret(theme_path, prompt, spec_path, cb, opts)
  local cfg = require("hexwitch.config").get()
  local args = { "interpret", theme_path }
  if opts and opts.from_llm then
    vim.list_extend(args, { "--from-llm", opts.from_llm })
  else
    -- -- ends option parsing: prompts beginning with '-' are still text.
    if cfg.provider then
      vim.list_extend(args, { "--provider", cfg.provider })
    end
    if cfg.model then
      vim.list_extend(args, { "--model", cfg.model })
    end
  end
  vim.list_extend(args, { "--out", spec_path })
  if not (opts and opts.from_llm) then
    vim.list_extend(args, { "--", prompt })
  end
  return M.exec(args, cb)
end

function M.revise(theme_path, spec_path, next_theme, lua_out, report_path, cb)
  return M.exec({ "revise", theme_path, "--spec", spec_path, "--out", next_theme,
    "--emit-nvim", lua_out, "--report-json", report_path }, cb)
end

function M.emit(theme_path, lua_out, cb)
  return M.exec({ "emit", theme_path, "--target", "nvim", "--out", lua_out, "--report" }, cb)
end

function M.analyze(theme_path, cb)
  return M.exec({ "analyze", theme_path }, cb)
end

return M
