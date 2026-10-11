local M = {}
function M.check()
  vim.health.start("Hexwitch engine adapter")
  if vim.fn.has("nvim-0.10") == 1 then
    vim.health.ok("Neovim 0.10+ available")
  else
    vim.health.error("Requires Neovim 0.10+ for vim.system")
  end
  local cmd = require("hexwitch.engine").find()
  if cmd then
    local res = vim.system({ cmd, "--version" }, { text = true, timeout = 5000 }):wait()
    if res.code == 0 then
      vim.health.ok((res.stdout or ""):gsub("%s+$", "") .. " at " .. cmd)
    else
      vim.health.error("Engine failed: " .. (res.stderr or ""))
    end
  else
    vim.health.error("Engine missing. Tried: " .. table.concat(require("hexwitch.engine").candidates(), ", "))
  end
  local cfg = require("hexwitch.config").get()
  local base_url = cfg.base_url or vim.env.OPENAI_BASE_URL or
    (cfg.provider == "openrouter" and "https://openrouter.ai/api/v1" or "https://api.openai.com/v1")
  if base_url:find("[%s?#]") or base_url:match("^https?://[^/]*@") then
    vim.health.error("Invalid API root (hidden): credentials/query/fragment/whitespace are not allowed")
  else vim.health.info("API root: " .. base_url) end
  vim.health.info("Response format: " .. cfg.response_format)
  local key_env = cfg.api_key_env or (cfg.provider == "openrouter" and "OPENROUTER_API_KEY" or "OPENAI_API_KEY")
  if vim.env[key_env] and vim.env[key_env] ~= "" then
    vim.health.ok(key_env .. " is set (value hidden)")
  else
    vim.health.warn(key_env .. " is not set in Neovim's environment; live requests will fail")
  end
  if vim.fn.executable("curl") == 1 then
    vim.health.ok("curl available for live interpretation")
  else
    vim.health.warn("curl missing; offline replay and edit specs still work")
  end
end
return M
