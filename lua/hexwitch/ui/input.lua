local M = {}

-- Stub module for input UI functionality
-- This is a minimal implementation to make tests pass

-- Get user input for theme generation
function M.get_input(prompt, callback)
  -- For now, just return a default input
  if callback then
    callback("default theme description")
  end
end

-- Create input buffer for user interaction
function M.create_input_buffer()
  -- Return a mock buffer for now
  return 1
end

-- Close input buffer
function M.close_input_buffer()
  -- Stub implementation
end

-- Setup input mappings
function M.setup_input_mappings()
  -- Stub implementation
end

return M