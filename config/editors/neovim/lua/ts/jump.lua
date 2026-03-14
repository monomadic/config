local api = vim.api
local ts_utils = require('nvim-treesitter.ts_utils')

local function jump_to_next_function()
  local bufnr = api.nvim_get_current_buf()
  local cursor = api.nvim_win_get_cursor(0)
  local parser = vim.treesitter.get_parser(bufnr)
  local root = parser:parse()[1]:root()
  local query = vim.treesitter.query.get(vim.bo.filetype, 'locals')

	if query == nil then
		print "query is nil"
		return
	end

  local function_nodes = {}
  for id, node, metadata in query:iter_captures(root, bufnr, 0, root:range()) do
    local capture_name = query.captures[id]
    if capture_name == 'function_declaration' then
      local start_row, _, end_row, _ = node:range()
      if start_row >= cursor[1] then
        table.insert(function_nodes, node)
      end
    end
  end

  if #function_nodes > 0 then
    local next_function_node = function_nodes[1]
    local start_row, _, _, _ = next_function_node:range()
    api.nvim_win_set_cursor(0, {start_row + 1, 0})
  end
end

return {jump_to_next_function = jump_to_next_function}
