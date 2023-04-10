--
--
-- KEYMAPS
--
-- to view current mappings: :verbose nmap <C-]>
--


-- TODO:
-- terminals
-- <leader>1 etc is term window
-- <C-c> kills window
-- <C-h> hides window
-- <C-1> switch to window
-- show mapping of active terminals in bottom bar
--
-- tab / smartjump
-- if errors exist, tab should jump to them
-- otherwise modules and types

local M = {}
local map = vim.keymap.set
local keymap = vim.keymap.set
local utils = require 'utils'
local key = utils.key
local lf = require 'lf'
local icons = require 'icons'

M.telescope = function()
	local pickers = require 'pickers'
	local builtin = require 'telescope.builtin'

	-- single letter actions
	key('n', '<leader>b', builtin.buffers, 'buffer')
	key('n', '<leader><tab>', pickers.open_same_filetype, 'source')
	keymap("n", "<leader>\\", "<cmd>ChatGPT<CR>", { desc = " ChatGPT" })
	keymap("n", "<leader><leader>", "<cmd>Lspsaga term_toggle<CR>", { desc = " ChatGPT" })
	keymap('n', '<leader>d', "<Cmd>Trouble workspace_diagnostics<CR>", { desc = "diagnostics" })
	keymap('n', '<leader>h', builtin.oldfiles, { desc = "history" })
	keymap('n', '<leader>t', ':TestFile<CR>', { desc = "history", silent = true })

	-- list
	keymap('n', "<leader>lgb", pickers.git_branches, { desc = "branches" })
	keymap('n', "<leader>lgc", pickers.git_commits, { desc = "commits" })
	keymap('n', '<leader>lb', builtin.buffers, { desc = "buffers…" })
	keymap('n', '<leader>ld', builtin.diagnostics, { desc = "diagnostics" })
	keymap('n', '<leader>lk', pickers.list_keymaps, { desc = "keymaps" })
	keymap('n', '<leader>ls', ":FzfLua lsp_workspace_symbols<CR>", { desc = "symbols" })

	-- document
	keymap('n', '<leader>De', pickers.lsp_document_enums, { desc = " enums…" })
	keymap('n', '<leader>Df', pickers.lsp_document_functions, { desc = " functions…" })
	keymap('n', "<leader>Dc", builtin.git_status, { desc = "changes" })
	keymap('n', "<leader>Dt", builtin.treesitter, { desc = " treesitter…" })

	-- git
	keymap('n', "<leader>Gb", pickers.git_branches, { desc = "branches" })
	keymap('n', "<leader>Gc", pickers.git_commits, { desc = "commits" })
	keymap('n', "<leader>Gs", pickers.git_status, { desc = "status" })

	-- open (various filters of file open pickers)
	keymap('n', "<leader>OT", pickers.open_template, { desc = "template" })
	keymap('n', "<leader>Ot", pickers.open_test, { desc = "test" })
	keymap('n', "<leader>Ow", pickers.wiki_open_page, { desc = "wiki page" })
	keymap('n', "<leader>OO", pickers.open_same_filetype, { desc = "same filetype" })
	keymap('n', "<leader>Os", function()
		require('telescope.builtin').find_files({ cwd = "~/.config/nvim/snippets/", follow = true })
	end, { desc = "snippet" })

	-- config (neovim related items)
	keymap('n', "<leader>Cf", pickers.open_config_file, { desc = "file..." })

	-- jump (locations related to current pos)
	keymap('n', '<leader>jf', pickers.lsp_document_functions,
		{ desc = icons.lspkind.Function .. "function" .. icons.icons.ellipsis })
	keymap('n', '<leader>jr', "<Cmd>Trouble lsp_references<CR>",
		{ desc = icons.lspkind.Reference .. "reference" .. icons.icons.ellipsis })
	-- keymap('n', '<leader>jm', pickers.lsp_document_functions,
	-- 	{ desc = icons.lspkind.Method .. "method" .. icons.icons.ellipsis })
	keymap('n', "<leader>ji", builtin.lsp_implementations, { desc = "implementations" })
	keymap('n', "<leader>jd", builtin.lsp_definitions, { desc = "definition" })

	-- goto (locations not related to current pos)
	keymap('n', "<leader>gb", pickers.git_branches, { desc = "git branches" })
	keymap('n', "<leader>gd", builtin.lsp_definitions, { desc = "definitions" })
	keymap('n', '<leader>gf', pickers.lsp_document_functions, { desc = " function…" })
	keymap('n', "<leader>gs", pickers.git_status, { desc = "change (git)" })
	keymap('n', '<leader>gt', '<Cmd>TodoTelescope<cr>', { desc = "todo" })
	keymap('n', "<leader>gT", pickers.open_template, { desc = "template" })
	keymap('n', "<leader>gw", pickers.wiki_open_page, { desc = "wiki page" })
	keymap('n', "gw", pickers.wiki_open_page, { desc = "wiki page" })
	keymap('n', "<Enter>", builtin.lsp_definitions, { desc = "definitions" })
	keymap('n', "gd", builtin.lsp_definitions)

	keymap('n', "gD", function()
		builtin.lsp_definitions {
			jump_type = "vsplit"
		}
	end, { desc = "definition split" })
	--
	-- insert
	keymap('n', '<leader>Nt', pickers.insert_template, { desc = "from template" })

	keymap('n', '<leader>o', pickers.open_files, { desc = "open…" })
	-- keymap('n', '<leader>t', '<C-w><C-s>:term<CR>i', { desc = " terminal" })

	keymap('n', 'tk', pickers.list_keymaps, { desc = "keymaps" })
	keymap('n', 'tld', '<Cmd>Telescope lsp_definitions<cr>')
	keymap('n', 'tli', '<Cmd>Telescope lsp_implementations<cr>')
	keymap('n', 'tlw', pickers.lsp_document_functions, { desc = "functions" })
	keymap('n', 'tlw', pickers.lsp_workspace_symbols, { desc = "symbols" })
	keymap('n', 'to', '<cmd>Telescope oldfiles<cr>')
	keymap('n', 'tr', '<Cmd>Telescope resume<cr>')
	keymap('n', 'tt', '<Cmd>TodoTelescope<cr>')

	-- wiki
	keymap('n', "<leader>Wg", pickers.wiki_search, { desc = "grep" })
	keymap('n', "<leader>Wo", pickers.wiki_open_page, { desc = "open" })
	keymap('n', "<leader>Ws", pickers.wiki_search, { desc = "search" })

	-- workspace
	keymap('n', '<leader>wd', builtin.diagnostics, { desc = "diagnostics" })
	keymap('n', '<leader>wM', '<cmd>Telescope marks<cr>', { desc = "mark…" })
	keymap('n', '<leader>wt', '<cmd>TodoTelescope<cr>', { desc = "todo…" })

	-- keymap('n', "\\o", pickers.open_files, { desc = "open…" })

	keymap('n', '<C-f>', builtin.live_grep, { desc = "find" })

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
-- actions
keymap('n', '<leader>f', utils.format, { desc = " format" })
keymap('n', "<leader>q", "<CMD>hide<CR>", { desc = "hide window" })
keymap('n', "<leader>!", "<cmd>quit!<CR>")
keymap('n', "<leader>a", vim.lsp.buf.code_action, { desc = "code-actions (saga)" })
--
-- git
keymap('n', "<leader>Gb", ":Telescope git_branches<CR>", { desc = "branches" })
keymap('n', "<leader>Gc", ":Telescope git_commits<CR>", { desc = "commits" })
keymap('n', "<leader>Gs", ":Telescope git_status<CR>", { desc = "status" })
--
-- run
map('n', "<leader>r", RunFile, { desc = "run" })
keymap('n', "<leader>Rb", Build, { desc = " build" })
--
-- goto
keymap('n', "<leader>gc", utils.open_config, { desc = "config" })
--
-- buffers
keymap('n', "<leader>Bu", "<CMD>bun<CR>", { desc = "unload" })
--
-- toggle
keymap('n', "<leader>Td", ":DrexDrawerToggle<CR>", { desc = "drex" })
keymap('n', "<leader>Tl", ToggleLineNumbers, { desc = "line numbers" })
keymap('n', "<leader>Tt", ":TransparentToggle<CR>", { desc = "tranparency" })

-- settings
keymap('n', "<leader>,l", ToggleLineNumbers, { desc = "toggle line numbers" })
keymap('n', "<leader>,t", ":TransparentToggle<CR>", { desc = "toggle tranparency" })
keymap('n', '<leader>,c', '<cmd>FzfLua colorschemes<cr>', { desc = "colorscheme" })
keymap('n', '<leader>,t', '<cmd>FzfLua colorschemes<cr>', { desc = "theme" })
keymap('n', '<leader>,u', '<cmd>Lazy update<cr>', { desc = "update plugins" })

-- floats
keymap('n', '<C-Space>', lf.show, { desc = "lf", remap = false })
keymap('t', '<C-Space>', function()
	vim.api.nvim_win_hide(0)
end)

keymap('t', '<C-t>', function()
	vim.api.nvim_buf_delete(0, { force = true })
	--vim.api.nvim_win_hide(0)
end)

-- save / write
keymap('n', "<C-s>", vim.cmd.write, { desc = "save" });
keymap('n', "<C-S>", vim.cmd.wall, { desc = "save all" });
keymap({ "v", "i" }, "<C-s>", "<Esc><Cmd>write<CR>", { desc = "save" });
-- window hide
keymap('n', "q", vim.cmd.hide, { desc = "hide" });
-- fast quit
keymap('n', "W", vim.cmd.wall)
keymap('n', "Q", "<cmd>wall<CR><cmd>qall<CR>")
-- leader
keymap('n', "<C-n>", ToggleLineNumbers, { desc = "toggle line numbers" })

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

-- use ; for commands instead of :
keymap('n', ";", ":")
-- keymap('n', "<Space>", ":")

keymap('i', "<C-v>", vim.cmd.put)

-- go back
keymap('n', '<bs>', ':edit #<cr>', { silent = true })

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
keymap("v", '"', '<Esc>`<i"<Esc>`>ea"<Esc>')
-- substitute shortcut
-- keymap('n', "S", ":%s//g<Left><Left>")
-- keymap("v", "S", ":s//g<Left><Left>")
-- more reachable line start/end
keymap('n', "H", "^")
keymap('n', "L", "$")

-- terminal
keymap('n', '<C-t>', '<C-w><C-s>:term<CR>i', { remap = false, silent = true })
keymap("t", '<C-\\>', '<C-\\><C-n>', { remap = false })
keymap("t", '<C-h>', '<C-\\><C-n><C-w><C-h>', { remap = false })
-- keymap("t", '<C-j>', '<C-\\><C-n><C-w><C-j>', { remap = false })
keymap("t", '<C-k>', '<C-\\><C-n><C-w><C-k>', { remap = false })
keymap("t", '<C-l>', '<C-\\><C-n><C-w><C-l>', { remap = false })

-- go
keymap('n', "gr", utils.go_root, { desc = "root" })
keymap('n', "gp", GoPackagerFile, { desc = "package manifest" })
keymap('n', "<leader>gr", utils.go_root, { desc = "root" })
keymap('n', "<leader>gp", GoPackagerFile, { desc = "package manifest" })
keymap('n', "<leader>gs", function()
	require('telescope.builtin').find_files({ cwd = "~/.config/nvim/snippets/", follow = true })
end, { desc = "snippet" })

-- list
keymap('n', '<leader>lc', '<cmd>FzfLua colorschemes<cr>', { desc = "colorschemes" })
keymap('n', '<leader>lC', '<cmd>Telescope commands<cr>', { desc = "commands (telescope)" })
--keymap('n', '<leader>lh', '<cmd>Telescope command_history<cr>', { desc = "command history (telescope)" })
keymap('n', '<leader>lf', '<cmd>Telescope filetypes<cr>', { desc = "filetypes…" })
keymap('n', '<leader>lh', '<cmd>Telescope highlights<cr>', { desc = "highlights" })
keymap('n', '<leader>lm', '<cmd>Telescope marks<cr>', { desc = "marks…" })
keymap('n', '<leader>lr', '<cmd>Telescope oldfiles<cr>', { desc = "recent files" })
keymap('n', '<leader>lt', '<cmd>TodoTelescope<cr>', { desc = "todos…" })

-- symbol
keymap('n', '<leader>Ss', '<cmd>Telescope spell_suggest<cr>', { desc = "spelling" })


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
		keymap('n', '<leader>Dd', '<Cmd>FzfLua diagnostics_document<cr>', { desc = "diagnostics" })

		-- next/prev: [ and ]

		-- map('n', ']d', vim.diagnostic.goto_next, { desc = "Next diagnostic" })
		-- map('n', '[d', vim.diagnostic.goto_prev, { desc = "Previous diagnostic" })
	end
})



-- rust
-- vim.api.nvim_create_autocmd("FileType", { pattern = "rust", callback = function()
-- end })

-- lua
vim.api.nvim_create_autocmd("FileType", { pattern = "lua", callback = function()
	map('n', "<C-r>", ":source %<CR>", { desc = "run" })
	map('n', "<leader>Rr", ":source %<CR>", { desc = "run" })
end })

return M
