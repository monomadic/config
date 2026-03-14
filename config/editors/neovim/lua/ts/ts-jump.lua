-- ~/.config/nvim/lua/treesitter_functions.lua

local ts_utils = require("nvim-treesitter.ts_utils")
local parsers = require("nvim-treesitter.parsers")

function SelectNextFunction()
  local bufnr = vim.api.nvim_get_current_buf()
  local lang = parsers.get_buf_lang(bufnr)

  if not lang then
    return
  end

  local cursor = vim.api.nvim_win_get_cursor(0)
  local cursor_pos = { cursor[1] - 1, cursor[2] }

  local parser = parsers.get_parser(bufnr, lang)
  local tree = parser:parse()[1]

  local function_node = nil

  tree:for_each_tree(function(subtree)
    local node = subtree:root()
    if ts_utils.is_in_node_range(node, cursor_pos[1], cursor_pos[2]) then
      local next_fn = ts_utils.get_next_node(node, "function", true)
      if next_fn and (not function_node or next_fn:start() < function_node:start()) then
        function_node = next_fn
      end
    end
  end)

  if function_node then
    ts_utils.update_selection(bufnr, function_node)
  else
    vim.api.nvim_echo({"No next function found", "WarningMsg"}, true, {})
  end
end

return {
  select_next_function = SelectNextFunction
}
