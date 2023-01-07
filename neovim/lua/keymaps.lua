-- KEYMAPS
--
--	to view current mappings: :verbose nmap <C-]>
--

local M = {}
local map = vim.keymap.set
local keymap = vim.keymap.set
local utils = require 'utils'
local lf = require 'lf'
local term = require 'term'

M.telescope = function()
	local pickers = require 'pickers'
	local builtin = require 'telescope.builtin'

	-- single letter actions
	keymap('n', '<leader>b', builtin.buffers, { desc = "buffer" })
	keymap('n', "<leader>s", pickers.open_same_filetype, { desc = "source" })

	-- list
	keymap('n', "<leader>lgb", pickers.git_branches, { desc = "branches" })
	keymap('n', '<leader>lb', builtin.buffers, { desc = "buffers…" })
	keymap('n', "<leader>lgc", pickers.git_commits, { desc = "commits" })
	keymap('n', '<leader>lk', pickers.list_keymaps, { desc = "keymaps" })

	-- document
	keymap('n', '<leader>De', pickers.lsp_document_enums, { desc = " enums…" })
	keymap('n', '<leader>Df', pickers.lsp_document_functions, { desc = " functions…" })

	-- git
	keymap('n', "<leader>Gb", pickers.git_branches, { desc = "branches" })
	keymap('n', "<leader>Gc", pickers.git_commits, { desc = "commits" })
	keymap('n', "<leader>Gs", pickers.git_status, { desc = "changed files" })

	-- open
	keymap('n', "<leader>OT", pickers.open_template, { desc = "template" })
	keymap('n', "<leader>Ot", pickers.open_test, { desc = "test" })
	keymap('n', "<leader>Ow", pickers.wiki_open_page, { desc = "wiki page" })
	keymap('n', "<leader>Os", pickers.open_same_filetype, { desc = "source file" })

	-- config
	keymap('n', "<leader>Cf", pickers.open_config_file, { desc = "file..." })

	-- goto
	keymap('n', "<leader>go", pickers.open_files, { desc = "open" })
	keymap('n', "<leader>gt", pickers.open_template, { desc = "template" })
	keymap('n', "<leader>gw", pickers.wiki_open_page, { desc = "wiki page" })

	-- insert
	keymap('n', '<leader>Nt', pickers.insert_template, { desc = "template" })

	keymap('n', '<leader>o', pickers.open_files, { desc = "open…" })
	keymap('n', '<leader>t', term.show, { desc = " terminal" })

	keymap('n', 'tk', pickers.list_keymaps, { desc = "keymaps" })
	keymap('n', 'tld', '<Cmd>Telescope lsp_definitions<cr>')
	keymap('n', 'tli', '<Cmd>Telescope lsp_implementations<cr>')
	keymap('n', 'tlw', pickers.lsp_document_functions, { desc = "functions" })
	keymap('n', 'tlw', pickers.lsp_workspace_symbols, { desc = "symbols" })
	keymap('n', 'to', '<cmd>Telescope oldfiles<cr>')
	keymap('n', 'tr', '<Cmd>Telescope resume<cr>')
	keymap('n', 'tt', '<Cmd>TodoTelescope<cr>')

	-- wiki
	keymap('n', "<leader>Wo", pickers.wiki_open_page, { desc = "open page" })
	keymap('n', "<leader>Wf", pickers.wiki_search, { desc = "find" })

	-- keymap('n', "ts", function()
	-- 	require("luasnip.loaders.from_snipmate").lazy_load()
	-- 	require('telescope').load_extension('luasnip')
	-- 	vim.api.nvim_command('Telescope luasnip')
	-- end)
end


M.whichkey = function()
	return {
		D = {
			s = { function()
				require('telescope.builtin').lsp_document_symbols { symbols = "struct" }
			end, " structs…" },
			S = { require('telescope.builtin').lsp_document_symbols, " symbols…" },
			t = { function()
				require('telescope.builtin').treesitter()
			end, " treesitter…" },
			m = { function()
				require('telescope.builtin').lsp_document_symbols { symbols = "module" }
			end, " modules…" },
		},
		w = {
			e = { function()
				require('telescope.builtin').lsp_workspace_symbols { symbols = "enum" }
			end, " enums…" },
			f = { function()
				require('telescope.builtin').lsp_workspace_symbols { symbols = "function", prompt_title = "", preview_title = "" }
			end, " functions…" },
			m = { function()
				require('telescope.builtin').lsp_workspace_symbols { symbols = "module" }
			end, " modules…" },
			s = { function()
				require('telescope.builtin').lsp_workspace_symbols { symbols = "struct" }
			end, " structs…" },
			S = { ":FzfLua lsp_workspace_symbols<CR>", " symbols…" },
		},
	}
end

-- NEXT/PREV
keymap('n', ']w', '*', { desc = "Next word" })
keymap('n', '[w', '#', { desc = "Previous word" })

-- GOTO
M.glance = function()
	keymap('n', "gR", "<CMD>Glance references<CR>")
	keymap('n', "<leader>Sd", "<CMD>Glance definitions<CR>", { desc = "glance definition" })
	keymap('n', "gY", "<CMD>Glance type_definitions<CR>")
	keymap('n', "gM", "<CMD>Glance implementations<CR>")
end

-- LEADER MENU
--
-- goto
keymap('n', "<leader>gc", utils.open_config, { desc = "config" })
--
-- buffers
--
-- toggle
keymap('n', "<leader>Td", ":DrexDrawerToggle<CR>", { desc = "drex" })
keymap('n', "<leader>Tl", ToggleLineNumbers, { desc = "line numbers" })
keymap('n', "<leader>Tt", ":TransparentToggle<CR>", { desc = "tranparency" })
--
-- actions
keymap('n', '<leader>f', utils.format, { desc = " format" })

keymap('n', '<C-Space>', lf.show, { desc = "lf" })
keymap('n', '<Tab>', term.show)
keymap('t', '<C-Space>', function()
	vim.api.nvim_win_hide(0)
end)

-- save / write
keymap('n', "<C-s>", "<CMD>write<CR>", { desc = "save" });
keymap('n', "<C-S>", "<CMD>wall<CR>", { desc = "save all" });
keymap({ "v", "i" }, "<C-s>", "<Esc><Cmd>write<CR>", { desc = "save" });

-- window hide
keymap('n', "q", "<CMD>hide<CR>");
keymap('n', "<leader>q", "<CMD>hide<CR>", { desc = "hide window" })
keymap('n', "<leader>Bu", "<CMD>bun<CR>", { desc = "unload" })
-- fast quit
keymap('n', "W", "<cmd>wall<CR>")
keymap('n', "Q", "<cmd>wall<CR><cmd>qall<CR>")
-- leader
keymap('n', "<leader>!", "<cmd>quit!<CR>")
keymap('n', "<leader><tab>", "<cmd>Drex<CR>", { desc = "drex" })
keymap('n', "<C-n>", ToggleLineNumbers, { desc = "toggle line numbers" })

keymap('n', "<leader>a", vim.lsp.buf.code_action, { desc = "code-actions (saga)" })

-- split navigation
keymap('n', "<C-j>", "<C-w><C-j>")
keymap("i", "<C-j>", "<Esc><C-w><C-j>")
keymap('n', "<C-k>", "<C-w><C-k>", { remap = false })
keymap("i", "<C-k>", "<Esc><C-w><C-k>")
keymap('n', "<C-l>", "<C-w><C-l>", { remap = false })
keymap("i", "<C-l>", "<Esc><C-w><C-l>")
keymap('n', "<C-h>", "<C-w><C-h>")
keymap("i", "<C-h>", "<Esc><C-w><C-h>")
keymap({ 'n', "t" }, "<C-w><C-d>", "<cmd>vsplit<CR>")

-- maximize
keymap('n', "<C-w>m", "<CMD>only<CR>", { desc = "Maximize" })
keymap('n', "<C-w><C-m>", "<CMD>only<CR>", { desc = "Maximize" })

-- hide
keymap('n', "<C-w><C-h>", "<CMD>hide<CR>", { desc = "Hide" })


-- jump to next paragraph
keymap('n', "}", "}j")
keymap('n', "{", "k{j")

-- jump page up/down 5 lines
--keymap('n', ">", "5j")
--keymap('n', "{", "k{j")

-- indent in insert mode
keymap("i", "<C-.>", "<C-t>")
keymap('n', "<C-.>", "i<C-t><C-f><C-f><Esc>") -- note: ctrl-f is fwd
keymap("v", "<C-.>", "<Esc><C-t>")

keymap("i", "<C-,>", "<C-d>")
keymap('n', "<C-,>", "i<C-d><C-f><Esc>")
-- keymap("v", "<C-,>", "<Esc><C-h>")

keymap('n', "gd", function() vim.lsp.buf.definition() end)
keymap('n', "<Enter>", function() vim.lsp.buf.definition() end)
keymap('n', "gc", function() vim.lsp.buf.declaration() end)
-- keymap('n', "gr", function() vim.lsp.buf.references() end)

keymap('n', "<C-b>", Build, { desc = " build" })
keymap('n', "<leader>Rb", Build, { desc = " build" })

-- use ; for commands instead of :
keymap('n', ";", ":")
-- keymap('n', "<Space>", ":")

-- go back
keymap('n', '<bs>', ':edit #<cr>', { silent = true })

-- grep entire project
keymap('n', "<C-f>", function()
	require('telescope.builtin').live_grep()
end)

keymap('n', "\\o", OpenFiles, { desc = "open file" })
keymap('n', "\\d", ":Drex<CR>", { desc = "drex" })
keymap('n', "\\f", ":DrexDrawerOpen<CR>", { desc = "filetree" })
keymap('n', "<C-b>", ":DrexDrawerToggle<CR>", { desc = "filetree" })
--keymap('n', "\\t", ShowTerminal, { desc = "terminal" })
--keymap('n', "<Tab>", ShowTerminal, { desc = "terminal" })
-- keymap('n', "<Tab>l", ShowTerminal, { desc = "lf" })

-- emacs style shortcuts in insert mode (yes, i am like that)
keymap("i", "<C-n>", "<Down>")
keymap("i", "<C-p>", "<Up>")
keymap("i", "<C-b>", "<Left>")
keymap("i", "<C-f>", "<Right>")
keymap("i", "<C-e>", "<End>")
keymap("i", "<C-a>", "<Home>")
keymap("i", "<C-s>", "<Esc>:write<CR>")
-- move lines up and down in visual mode
keymap("x", "K", ":move '<-2<CR>gv-gv")
keymap("x", "J", ":move '>+1<CR>gv-gv")
--
-- quote quickly
--keymap("i", '<leader>"', '<Esc>viw<Esc>a"<Esc>bi"<Esc>leli')
keymap("v", '"', '<Esc>`<i"<Esc>`>ea"<Esc>')
-- substitute shortcut
-- keymap('n', "S", ":%s//g<Left><Left>")
-- keymap("v", "S", ":s//g<Left><Left>")
-- more reachable line start/end
keymap('n', "H", "^")
keymap('n', "L", "$")

-- terminal
keymap('n', '<C-t>', '<C-w><C-s>:term<CR>i', { remap = false })
keymap("t", '<C-\\>', '<C-\\><C-n>', { remap = false })
keymap("t", '<C-h>', '<C-\\><C-n><C-w><C-h>', { remap = false })
-- keymap("t", '<C-j>', '<C-\\><C-n><C-w><C-j>', { remap = false })
keymap("t", '<C-k>', '<C-\\><C-n><C-w><C-k>', { remap = false })
keymap("t", '<C-l>', '<C-\\><C-n><C-w><C-l>', { remap = false })

-- git
keymap('n', "<leader>Gb", ":Telescope git_branches<CR>", { desc = "branches" })
keymap('n', "<leader>Gc", ":Telescope git_commits<CR>", { desc = "commits" })
keymap('n', "<leader>Gs", ":Telescope git_status<CR>", { desc = "status" })

-- go
keymap('n', "gr", GoRoot, { desc = "root" })
keymap('n', "gp", GoPackagerFile, { desc = "package manifest" })
keymap('n', "<leader>gr", GoRoot, { desc = "root" })
keymap('n', "<leader>gp", GoPackagerFile, { desc = "package manifest" })
keymap('n', "<leader>gs", function()
	require('telescope.builtin').find_files({ cwd = "~/.config/nvim/snippets/", follow = true })
end, { desc = "snippet" })

-- list
keymap('n', '<leader>lb', '<cmd>Telescope buffers<cr>', { desc = "buffers…" })
keymap('n', '<leader>lc', '<cmd>FzfLua colorschemes<cr>', { desc = "colorschemes" })
keymap('n', '<leader>lC', '<cmd>Telescope commands<cr>', { desc = "commands (telescope)" })
--keymap('n', '<leader>lh', '<cmd>Telescope command_history<cr>', { desc = "command history (telescope)" })
keymap('n', '<leader>ld', '<cmd>Telescope diagnostics<cr>', { desc = "diagnostics" })
keymap('n', '<leader>lf', '<cmd>Telescope filetypes<cr>', { desc = "filetypes…" })
keymap('n', '<leader>lh', '<cmd>Telescope highlights<cr>', { desc = "highlights" })
keymap('n', '<leader>lm', '<cmd>Telescope marks<cr>', { desc = "marks…" })
keymap('n', '<leader>lr', '<cmd>Telescope oldfiles<cr>', { desc = "recent files" })
keymap('n', '<leader>lt', '<cmd>TodoTelescope<cr>', { desc = "todos…" })

-- symbol
keymap('n', '<leader>Ss', '<cmd>Telescope spell_suggest<cr>', { desc = "spelling" })

-- workspace
keymap('n', '<leader>wM', '<cmd>Telescope marks<cr>', { desc = "mark…" })
keymap('n', '<leader>wt', '<cmd>TodoTelescope<cr>', { desc = "todo…" })

-- sessions
-- local session_dir = vim.fn.stdpath('data') .. '/sessions/'
-- keymap('n', '<leader>mks', ':mks! ' .. session_dir)
-- keymap('n', '<leader>lds', ':%bd | so ' .. session_dir)

-- visual mode
map('v', '<leader>u', ':UpperCase<CR>')
map('v', '<leader>l', ':LowerCase<CR>')
map('v', '<leader>C', ':CamelCase<CR>')
map('v', '<leader>c', ':Capitalize<CR>')

map('n', '<leader>?', ":Telescope help_tags<CR>", { desc = "help" })

-- lsp
vim.api.nvim_create_autocmd("LspAttach", {
	callback = function()
		-- if not (args.data and args.data.client_id) then
		-- 	return
		-- end
		--
		-- local bufnr = args.buf
		-- local client = vim.lsp.get_client_by_id(args.data.client_id)
		-- require("lsp-inlayhints").on_attach(client, bufnr)
		keymap('n', '<leader>d', '<Cmd>Telescope diagnostics<cr>', { desc = "diagnostics" })
		keymap('n', '<leader>Dd', '<Cmd>FzfLua diagnostics_document<cr>', { desc = "diagnostics" })
		keymap('n', '<leader>wd', '<Cmd>Telescope diagnostics<cr>', { desc = "diagnostics" })

		-- next/prev: [ and ]

		map('n', ']d', vim.diagnostic.goto_next, { desc = "Next diagnostic" })
		map('n', '[d', vim.diagnostic.goto_prev, { desc = "Previous diagnostic" })
	end
})

-- rust
-- vim.api.nvim_create_autocmd("FileType", { pattern = "rust", callback = function()
-- end })

-- lua
vim.api.nvim_create_autocmd("FileType", { pattern = "lua", callback = function()
	map('n', "<leader>r", ":source %<CR>", { desc = "run" })
	map('n', "<C-r>", ":source %<CR>", { desc = "run" })
	map('n', "<leader>Rr", ":source %<CR>", { desc = "run" })
end })

return M
