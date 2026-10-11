-- Argument-list process calls only: no shell quoting, color math or API keys here.
local M = {}

local source = debug.getinfo(1, "S").source:gsub("^@", "")
local checkout = vim.fn.fnamemodify(source, ":p:h:h:h:h:h")

function M.candidates()
  local cfg = require("hexwitch.config").get()
  if cfg.cmd then return { vim.fn.expand(cfg.cmd) } end
  local suffix = vim.fn.has("win32") == 1 and ".exe" or ""
  return { "hexwitch", checkout .. "/core/target/release/hexwitch" .. suffix,
    checkout .. "/core/target/debug/hexwitch" .. suffix }
end

function M.find()
  for _, cmd in ipairs(M.candidates()) do
    if vim.fn.executable(cmd) == 1 then return cmd end
  end
end

-- cb(err, stdout, stderr); callbacks always run on Neovim's main loop.
function M.exec(args, cb)
  local cmd = M.find()
  if not cmd then
    vim.schedule(function()
      cb("hexwitch binary not found. Tried: " .. table.concat(M.candidates(), ", ")
        .. ". Run cargo build --release --manifest-path " .. checkout .. "/core/Cargo.toml -p hexwitch-cli, or set cmd to your built executable.", "", "")
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

-- v1 clients inspect structured outcomes; stderr is not part of the protocol.
function M.exec_machine(args, cb)
  local argv = { "--machine" }
  vim.list_extend(argv, args)
  return M.exec(argv, function(err, stdout, stderr)
    local ok, response = pcall(vim.json.decode, stdout)
    if not ok or type(response) ~= "table" or response.protocol_version ~= 1
        or type(response.outcome) ~= "string" then
      return cb(err or "Invalid versioned engine response", stdout, stderr)
    end
    if response.outcome == "error" then
      local failure = type(response.errors) == "table" and response.errors[1] or nil
      local message = type(failure) == "table" and (tostring(failure.code) .. ": " .. tostring(failure.message))
          or "Engine returned a protocol error"
      return cb(message, stdout, stderr, response)
    end
    if err then return cb(err, stdout, stderr, response) end
    cb(nil, stdout, stderr, response)
  end)
end

function M.interpret(theme_path, prompt, spec_path, cb, opts)
  local cfg = require("hexwitch.config").get()
  local args = { "interpret", theme_path }
  if opts and opts.v1 then
    table.insert(args, "--v1")
    if opts.context then vim.list_extend(args, { "--context", opts.context }) end
  end
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
    if cfg.base_url then vim.list_extend(args, { "--base-url", cfg.base_url }) end
    if cfg.api_key_env then vim.list_extend(args, { "--api-key-env", cfg.api_key_env }) end
    vim.list_extend(args, { "--response-format", cfg.response_format })
  end
  vim.list_extend(args, { "--out", spec_path })
  if not (opts and opts.from_llm) or (opts and opts.v1) then
    vim.list_extend(args, { "--", prompt })
  end
  if opts and opts.v1 then return M.exec_machine(args, cb) end
  return M.exec(args, cb)
end

function M.revise(theme_path, spec_path, next_theme, lua_out, report_path, cb, opts)
  local args = { "revise", theme_path, "--spec", spec_path, "--out", next_theme,
    "--emit-nvim", lua_out, "--report-json", report_path }
  if opts and opts.v1 then return M.exec_machine(args, cb) end
  return M.exec(args, cb)
end

function M.emit(theme_path, lua_out, cb)
  return M.exec({ "emit", theme_path, "--target", "nvim", "--out", lua_out, "--report" }, cb)
end

function M.analyze(theme_path, cb)
  return M.exec({ "analyze", theme_path }, cb)
end

return M
