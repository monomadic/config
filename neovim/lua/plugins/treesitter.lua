local treesitter = require "vim.treesitter"
-- treesitter
return {
	'nvim-treesitter/nvim-treesitter',

	dependencies = {
		"p00f/nvim-ts-rainbow",
		'nvim-treesitter/nvim-treesitter-textobjects',
		'nvim-treesitter/playground',
	},

	config = function()
		require 'nvim-treesitter.configs'.setup {
			ensure_installed = { "rust", "bash", "yaml", "typescript", "javascript", "markdown", "lua" },
			auto_install = true, -- install missing when entering buffer
			highlight = { enable = true },
			rainbow = { enable = true, colors = {
				"#9944FF",
				"#45F588",
				"#FFFF00",
				"#FF44FF",
				"#00BBFF",
				"#FFAACC",
				"#AAFF66",
			} },
			matchup = {
				enable = true, -- mandatory, false will disable the whole extension
				disable = {}, -- optional, list of language that will be disabled
			},

			playground = {
				enable = true,
				disable = {},
				updatetime = 25, -- Debounced time for highlighting nodes in the playground from source code
				persist_queries = false, -- Whether the query persists across vim sessions
				keybindings = {
					toggle_query_editor = 'o',
					toggle_hl_groups = 'i',
					toggle_injected_languages = 't',
					toggle_anonymous_nodes = 'a',
					toggle_language_display = 'I',
					focus_language = 'f',
					unfocus_language = 'F',
					update = 'R',
					goto_node = '<cr>',
					show_help = '?',
				},
			},

			textobjects = {
				lookahead = true, -- Automatically jump forward to textobj, similar to targets.vim
				select = {
					enable = true,
					lookahead = true,
					-- The keymaps are defined in the configuration table, no way to get our Mapper in there !
					keymaps = { -- prefix v
						["af"] = "@function.outer",
						["if"] = "@function.inner",
						["ac"] = "@class.outer",
						["ic"] = "@class.inner"
					}
				},
				move = {
					enable = true,
					set_jumps = true,
					goto_next_start = {
						["]f"] = { query = "@function.outer", desc = "Next function start" },
						["]]"] = { query = "@function.outer", desc = "Next function start" },
					},
					goto_next_end = {
						["]F"] = "@function.outer",
						["]["] = "@function.outer",
					},
					goto_previous_start = {
						["[f"] = "@function.outer",
						["[["] = "@function.outer",
					},
					goto_previous_end = {
						["[F"] = "@function.outer",
						["[]"] = "@function.outer",
					},
				},
			},
			incremental_selection = {
				enable = true,
				keymaps = {
					init_selection = "vv",
					node_incremental = "v",
					scope_incremental = "<CR>",
					node_decremental = "V",
				},
			},
		}

		-- vim.keymap.set("n", "", function()
		-- 	local tsparser = vim.treesitter.get_parser()
		--
		-- end)
		function JumpModule()
			local ts_utils = require("nvim-treesitter.ts_utils")
			local node = ts_utils.get_node_at_cursor()

			if not node then
				return
			end

			--while ts_utils.get_next_node()

			--ts_utils.goto_node(parent)
		end

		function JumpRoot()
			local ts_utils = require("nvim-treesitter.ts_utils")
			local node = ts_utils.get_node_at_cursor()
			local root = ts_utils.get_root_for_node(node)
			ts_utils.goto_node(root)
		end

		function JumpNodeParent()
			local ts_utils = require("nvim-treesitter.ts_utils")
			local node = ts_utils.get_node_at_cursor()
			local parent = node
			local result = node

			while parent:type() ~= 'source_file' do
				result = parent
				parent = result:parent()
			end

			ts_utils.goto_node(result)
			return result
		end

		function JumpNodeParent1()
			local ts_utils = require("nvim-treesitter.ts_utils")
			local node = ts_utils.get_node_at_cursor()
			local parent = node:parent()

			while parent:type() ~= 'source_file' do
				ts_utils.goto_node(parent)
				parent = parent:parent()
			end
		end

		function JumpNext()
			local ts_utils = require("nvim-treesitter.ts_utils")
			local node = ts_utils.get_node_at_cursor()
			local next_node = ts_utils.get_next_node(node)
			ts_utils.goto_node(next_node)
		end

		function JumpNextModule1()
			JumpNodeParent()
			JumpNext()

			local ts_utils = require("nvim-treesitter.ts_utils")
			local node = ts_utils.get_node_at_cursor()

			while ts_utils.get_next_node(node) do
				node = ts_utils.get_next_node(node)
				if node:type() == 'mod_item' then
					for child in node:iter_children() do
						if child:type() == 'identifier' then
							print(ts_utils.get_node_text(child)[1])
							ts_utils.goto_node(child)
							return
						end
					end
				end
			end

			print("no module found")
		end

		function JumpNextModule()
			local ts_utils = require("nvim-treesitter.ts_utils")
			local node = ts_utils.get_node_at_cursor()

			if not node then
				print 'no node at cursor'
				return
			end

			-- go to start of the node
			local parent = node:parent()
			if not parent then return end

			while not parent:type() == 'source_file' do
				parent = node:parent()
				print("parent "..parent:type())
			end


			local next_node = ts_utils.get_next_node(parent)

			if next_node then
				node = next_node
			end

			ts_utils.goto_node(node)

			while node do
				if node:type() == 'mod_item' then
					for child in node:iter_children() do
						if child:type() == 'identifier' then
							print(ts_utils.get_node_text(child)[1])
							ts_utils.goto_node(child)
							return
						end
					end
				end
				node = ts_utils.get_next_node(node)
			end

			print("no module found")
		end
		vim.keymap.set('n', '<Tab>', JumpNextModule, { silent = true })

		function JumpPrevModule()
			local ts_utils = require("nvim-treesitter.ts_utils")
			local node = ts_utils.get_node_at_cursor()

			-- go to start of the node
			local parent = node:parent()
			while not parent:type() == 'source_file' do
				parent = node:parent()
			end

			print("parent "..parent:type())

			local prev_node = ts_utils.get_previous_node(parent)

			if prev_node then
				node = prev_node
			end

			ts_utils.goto_node(node)

			while node do
				if node:type() == 'mod_item' then
					for child in node:iter_children() do
						if child:type() == 'identifier' then
							print(ts_utils.get_node_text(child)[1])
							ts_utils.goto_node(child)
							return
						end
					end
				end
				node = ts_utils.get_previous_node(node)
			end

			print("no module found")
		end
		vim.keymap.set('n', '<S-Tab>', JumpPrevModule, { silent = true })

		function JumpParent()
			local ts_utils = require("nvim-treesitter.ts_utils")
			local node = ts_utils.get_node_at_cursor()

			if not node then
				return
			end

			-- local sibling = node:next_sibling()
			-- if (sibling == nil) then
			-- 	return
			-- end

			local parent = node:parent()
			if (parent == nil) then
				return
			end

			ts_utils.goto_node(parent)
		end

	end
}
