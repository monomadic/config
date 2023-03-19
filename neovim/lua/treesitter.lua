-- https://github.com/nvim-treesitter/nvim-treesitter/blob/master/lua/nvim-treesitter/ts_utils.lua
-- https://github.com/s1n7ax/nvim-ts-utils/blob/main/lua/ts-utils/treesitter.lua
-- https://github.com/s1n7ax/youtube-neovim-treesitter-query
local ts = vim.treesitter

local M = {}

function M:t()
	local ts_utils = require('nvim-treesitter.ts_utils')
	local parsers = require('nvim-treesitter.parsers')

	local bufnr = vim.api.nvim_get_current_buf()
	local cursor = vim.api.nvim_win_get_cursor(0)

	if not parsers.has_parser() then
		print("No Treesitter parser available for current buffer")
		return
	end

	local root = M.get_root_node()
	--local query = ts.get_query(parsers.get_buf_lang(bufnr), "functions")

	local lang = parsers.get_buf_lang(bufnr)
	local query = vim.treesitter.parse_query(lang, "(function_item) @function")

	if not query then
		print("Query returned no results")
		return
	end

	local next_function_node = nil
	local function_iter = query:iter_captures(root, bufnr)

	for id, node in function_iter do
		local node_start = { node:start() }
		if ts_utils.compare_positions(cursor, node_start) < 0 then
			if next_function_node == nil or ts_utils.compare_positions(node_start, { next_function_node:start() }) < 0 then
				next_function_node = node
			end
		end
	end

	if next_function_node then
		local start_row, start_col, end_row, end_col = next_function_node:range()
		vim.api.nvim_win_set_cursor(0, { start_row + 1, start_col })
		vim.api.nvim_exec("normal! v", false)
		vim.api.nvim_win_set_cursor(0, { end_row + 1, end_col })
	else
		print("No next function found")
	end

	-- return {
	-- 	select_next_function = M:t(),
	-- }
end

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

function M:query_tree(query)
	-- Parse the current buffer with Treesitter
	local parser = M:get_root_node()

	-- Find all nodes in the tree that match the query
	local matches = parser:query(query):captures()

	-- Map each match to its node object
	local nodes = {}
	for _, match in ipairs(matches) do
		table.insert(nodes, match.node)
	end

	return nodes
end

function M:mods()
	local mod_nodes = M:query_tree("(call_expression function_name:_)")
	for _, node in ipairs(mod_nodes) do
		print(node:type())
	end
end

function M:query__()
	-- local query = ts.parse_query('rust', '(use_declaration (scoped_identifier (name: (identifier) @name)))')
	local query = ts.parse_query('rust', '(mod_item (visibility_modifier) (identifier) @name)')
	--local tree = M:get_parser():parse()
	local node = M:get_root_node()
	local q = require 'vim.treesitter.query'
	--local l = require 'vim.treesitter.languagetree'

	for capture, node, metadata in query:iter_matches(node, 0) do
		for id, node in ipairs(node) do
			i(q.get_node_text(node[1], 0))
			i(node)
			i(metadata[id])
		end
	end

	for _, captures, metadata in query:iter_matches(node, 0) do
		local node = captures[1]

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
