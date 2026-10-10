local logger = require("hexwitch.utils.logger")

local M = {}

-- Registry of available providers
local providers = {}

-- Register a provider
---@param name string Provider name
---@param provider_module table Provider implementation
function M.register(name, provider_module)
  providers[name] = provider_module
  logger.debug("ai.providers", "register", string.format("Registered AI provider: %s", name))
end

-- Get a provider by name
---@param name string Provider name
---@return table|nil provider Provider implementation
function M.get(name)
  return providers[name]
end

-- Get list of available providers
---@return table names List of provider names
function M.list_available()
  local names = {}
  for name, _ in pairs(providers) do
    table.insert(names, name)
  end
  return names
end

-- Factory to create configured provider instance
---@param config table Provider configuration
---@return table|nil provider Configured provider instance
---@return string|nil error Error message
function M.create(config)
  local name = config.ai_provider or "openai"
  local provider = providers[name]

  if not provider then
    local available = table.concat(M.list_available(), ", ")
    local error_msg = string.format("Unknown AI provider '%s'. Available: %s", name, available)
    logger.error("ai.providers", "create", error_msg, { requested = name, available = available })
    return nil, error_msg
  end

  logger.debug("ai.providers", "create", string.format("Creating AI provider: %s", name))
  return provider.new(config)
end

-- Auto-discover and register providers in this directory
local function discover_providers()
  local provider_dir = debug.getinfo(1, "S").source:sub(2):gsub("init.lua", "")
  local files = vim.fn.glob(provider_dir .. "*.lua", false, true)

  for _, file in ipairs(files) do
    local name = vim.fn.fnamemodify(file, ":t:r")
    if name ~= "init" and name ~= "base" then
      local success, provider_module = pcall(require, "hexwitch.ai.providers." .. name)
      if success and provider_module and provider_module.is_available() then
        M.register(name, provider_module)
      else
        logger.debug("ai.providers", "discover_providers",
          string.format("Provider %s not available or failed to load", name))
      end
    end
  end
end

-- Initialize provider registry
discover_providers()

return M