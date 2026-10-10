-- Compatibility utilities for different Neovim versions

local M = {}

-- Provide vim.deepcopy for older Neovim versions
if not vim.deepcopy then
  function vim.deepcopy(orig)
    local orig_type = type(orig)
    local copy
    if orig_type == 'table' then
      copy = {}
      for orig_key, orig_value in next, orig, nil do
        copy[M.deepcopy(orig_key)] = M.deepcopy(orig_value)
      end
      setmetatable(copy, M.deepcopy(getmetatable(orig)))
    else -- number, string, boolean, etc
      copy = orig
    end
    return copy
  end
end

-- Provide vim.list_slice for older Neovim versions
if not vim.list_slice then
  function vim.list_slice(list, start_idx, end_idx)
    local result = {}
    end_idx = end_idx or #list

    for i = start_idx, end_idx do
      if list[i] then
        table.insert(result, list[i])
      end
    end

    return result
  end
end

return M