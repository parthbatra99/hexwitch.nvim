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
      vim.health.ok((res.stdout or ""):gsub("%s+$", ""))
    else
      vim.health.error("Engine failed: " .. (res.stderr or ""))
    end
  else
    vim.health.error("Engine missing; build core/ and configure cmd with the binary path")
  end
  if vim.fn.executable("curl") == 1 then
    vim.health.ok("curl available for live interpretation")
  else
    vim.health.warn("curl missing; offline replay and edit specs still work")
  end
end
return M
