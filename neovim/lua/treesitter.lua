-- https://github.com/nvim-treesitter/nvim-treesitter/blob/master/lua/nvim-treesitter/ts_utils.lua
-- https://github.com/s1n7ax/nvim-ts-utils/blob/main/lua/ts-utils/treesitter.lua
-- https://github.com/s1n7ax/youtube-neovim-treesitter-query
local ts = vim.treesitter

local M = {}

-- Returns the root node of the first syntax tree
-- @returns {Node}
-- @see {@link lua-treesitter-node| https://neovim.io/doc/user/treesitter.html#lua-treesitter-node} node on the cursor
function M:get_root_node()
	return M:get_syntax_trees()[1]:root()
end

-- Refresh the syntax tree and returns the syntax tree
-- @returns {Array<Tree>}
-- @see {@link lua-treesitter-tree| https://neovim.io/doc/user/treesitter.html#lua-treesitter-tree}
function M:get_syntax_trees()
  return M:get_parser():parse()
end

-- Refresh the syntax tree
function M:refresh()
  M:get_parser():parse()
end

function M:get_parser()
	local parsers = require "nvim-treesitter.parsers"
	return parsers.get_parser()
  -- return ts.get_parser(self.buffer, self.language)
end

function M:query()
	-- local query = ts.parse_query('rust', '(use_declaration (scoped_identifier (name: (identifier) @name)))')
	local query = ts.parse_query('rust', '(mod_item (visibility_modifier) (identifier) @name)')
	local node = M:get_root_node()
	local q = require 'vim.treesitter.query'

	for _, captures, metadata in query:iter_matches(node, 0) do
		i(q.get_node_text(captures[1], 0))
		M.goto_node(captures[1])
	end
end

-- Returns list of nodes that is in between node at the given position and the root node
-- In following node tree, if the node at position is "D" then the scope would
-- return [Node(D), Node(B), Node(R)]
--      ┌─────┐
--    ┌─┤  R  ├─┐
--    │ └─────┘ │
--    │         │
-- ┌──┴──┐   ┌──┴──┐
-- │  A  │ ┌─┤  B  ├─┐
-- └─────┘ │ └─────┘ │
--         │         │
--         │         │
--      ┌──┴──┐   ┌──┴──┐
--      │  C  │   │  D  │
--      └─────┘   └─────┘
--
-- @returns {Array<Node>} list of nodes
-- @see {@link lua-treesitter-node| https://neovim.io/doc/user/treesitter.html#lua-treesitter-node} node on the cursor
function M:get_scope_at_pos(row, column)
	local root = M:get_root_node()
	local get_scope = M:__get_scope_at_pos_finder(row, column)

	return get_scope(root)
end

-- Returns the node text
-- @param {Node} to get the text of
-- @returns {string} text of the node
-- @see {@link lua-treesitter-node| https://neovim.io/doc/user/treesitter.html#lua-treesitter-node} node on the cursor
function M:get_node_text(node)
	---@diagnostic disable-next-line: undefined-global
	return vim.treesitter.query.get_node_text(node, self.buffer)
end

M.get_node_at_pos = function(line, column)
	local scope = M:get_scope_at_pos(line, column)

	if #scope == 0 then
		return
	end

	return scope[1]
end

-- Returns the nearest root node
function M:get_nearest_root_node()
	local row = Utils.current_row() - 1
	local col = Utils.current_col()
	local scope = M:get_scope_at_pos(row, col)
	return scope[#scope]
end

M.jump_next_root_node = function()
	local ts_utils = require("nvim-treesitter.ts_utils")
	local curr_root = M:get_nearest_root_node()
	ts_utils.goto_node(ts_utils.get_next_node(curr_root, true, true))
end

M.goto_node = function(node)
	local ts_utils = require("nvim-treesitter.ts_utils")
	ts_utils.goto_node(node)
end

-- Returns the node on the cursor
-- @returns {Node}
-- @see {@link lua-treesitter-node| https://neovim.io/doc/user/treesitter.html#lua-treesitter-node} node on the cursor
function M:get_curr_node()
	local ts_utils = require("nvim-treesitter.ts_utils")
  return ts_utils.get_node_at_cursor()
end

-- M.jump_next_named_node = function()
-- 	local ts_utils = require("nvim-treesitter.ts_utils")
-- 	local current_node = ts_utils.get_node_at_cursor()
-- 	local next_node = ts_utils.get_next_node(current_node, true, true)
-- 	for _, child in ipairs(M.named_nodes(next_node)) do
-- 		print(child:type(), ts_utils.get_node_text(child)[1])
-- 	end
-- end

M.named_nodes = function(root_node)
	local ts = require('nvim-treesitter.ts_utils')
	local named_nodes = {}

	if named_nodes then
		for node in root_node:walk() do
			if node:type() ~= nil and node:has_name() then
				table.insert(named_nodes, node)
			end
		end
	end

	return named_nodes
end

M.jump_next_named_node = function()
	local ts_utils = require("nvim-treesitter.ts_utils")

	function next_named_node(current_node)
		local next_node = ts_utils.get_next_node(current_node, true, true)
		if next_node then
			-- for child in next_node:field('name') do
			-- 	print(child)
			-- end
			-- print(next_node:named_child_count())


			for _, child in ipairs(ts_utils.get_named_children(next_node)) do
				print(child:type(), ts_utils.get_node_text(child)[1])
			end
			-- for _, child in ipairs(next_node:field('name')) do
			-- 	print(child:type(), ts.get_node_text(child)[1])
			-- end


			--ts.goto_node(next_node)

			-- if next_node:type() == 'identifier' then
			-- 	print(next_node:type())
			-- 	ts.goto_node(next_node)
			-- 	return
			-- end
			next_named_node(next_node)
		end
	end

	local current_node = M.get_nearest_root_node()
	if current_node then
		next_named_node(current_node)
	end
end

function M:__get_scope_at_pos_finder(row, column)
	local scope = {}

	local function find(root)
		for node in root:iter_children() do
			local sraw, scolumn = node:start()
			local eraw, ecolumn = node:end_()

			if row >= sraw and row <= eraw then
				if (row > sraw and row < eraw)
						or (row > sraw and column <= ecolumn)
						or (row < eraw and column >= scolumn)
						or (
						(row == sraw or row == eraw)
						and column >= scolumn
						and column <= ecolumn
						)
				then
					table.insert(scope, find(node))
				end
			end
		end

		return root
	end

	return function(root)
		find(root)
		return scope
	end
end

return M
