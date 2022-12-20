-- KEYMAPS
--
--	to view current mappings: :verbose nmap <C-]>
--

local M = {}
local keymap = vim.keymap.set

M.telescope = function()
	keymap("n", "<leader>Gb", ":Telescope git_branches<CR>", { desc = "branches" })
	keymap("n", "<leader>Gc", ":Telescope git_commits<CR>", { desc = "commits" })
	keymap("n", "<leader>Gs", ":Telescope git_status<CR>", { desc = "status" })

	keymap("n", "<leader>o", ":Telescope find_files<CR>", { desc = "open" })
end

M.whichkey = function()
end

-- save / write
vim.keymap.set("n", "<C-s>", "<CMD>write<CR>", { desc = "save" });
vim.keymap.set("n", "<C-S>", "<CMD>wall<CR>", { desc = "save all" });
vim.keymap.set({ "v", "i" }, "<C-s>", "<Esc><Cmd>write<CR>", { desc = "save" });

-- window hide
vim.keymap.set("n", "q", "<CMD>hide<CR>");
-- buffer unload
vim.keymap.set("n", "<leader>q", "<CMD>hide<CR>", { desc = "hide window" })
vim.keymap.set("n", "<leader>u", "<CMD>bun<CR>", { desc = "unload buffer" })
-- fast quit
vim.keymap.set("n", "W", "<cmd>wall<CR>")
vim.keymap.set("n", "Q", "<cmd>wall<CR><cmd>qall<CR>")
-- leader
vim.keymap.set("n", "<leader>!", "<cmd>quit!<CR>")
vim.keymap.set("n", "<leader><tab>", "<cmd>Drex<CR>", { desc = " Drex" })
vim.keymap.set("n", "<C-n>", ToggleLineNumbers, { desc = "toggle line numbers" })

vim.keymap.set("n", '<leader>df', function() vim.lsp.buf.format { async = true } end, { desc = "format" })
vim.keymap.set("n", '<leader>F', function() vim.lsp.buf.format { async = true } end, { desc = "format" })

vim.keymap.set("n", "<leader>a", vim.lsp.buf.code_action, { desc = "code-actions (saga)" })

-- split navigation
vim.keymap.set("n", "<C-j>", "<C-w><C-j>")
vim.keymap.set("i", "<C-j>", "<Esc><C-w><C-j>")
vim.keymap.set("n", "<C-k>", "<C-w><C-k>", { remap = false })
vim.keymap.set("i", "<C-k>", "<Esc><C-w><C-k>")
vim.keymap.set("n", "<C-l>", "<C-w><C-l>", { remap = false })
vim.keymap.set("i", "<C-l>", "<Esc><C-w><C-l>")
vim.keymap.set("n", "<C-h>", "<C-w><C-h>")
vim.keymap.set("i", "<C-h>", "<Esc><C-w><C-h>")
vim.keymap.set({ "n", "t" }, "<C-w><C-d>", "<cmd>vsplit<CR>")

-- maximize
vim.keymap.set("n", "<C-w>m", "<CMD>only<CR>", { desc = "Maximize" })
vim.keymap.set("n", "<C-w><C-m>", "<CMD>only<CR>", { desc = "Maximize" })

-- hide
vim.keymap.set("n", "<C-w><C-h>", "<CMD>hide<CR>", { desc = "Hide" })

-- jump to next paragraph
vim.keymap.set("n", "}", "}j")
vim.keymap.set("n", "{", "k{j")

-- jump page up/down 5 lines
--vim.keymap.set("n", ">", "5j")
--vim.keymap.set("n", "{", "k{j")

-- indent in insert mode
vim.keymap.set("i", "<C-.>", "<C-t>")
vim.keymap.set("n", "<C-.>", "i<C-t><C-f><C-f><Esc>") -- note: ctrl-f is fwd
vim.keymap.set("v", "<C-.>", "<Esc><C-t>")

vim.keymap.set("i", "<C-,>", "<C-d>")
vim.keymap.set("n", "<C-,>", "i<C-d><C-f><Esc>")
-- vim.keymap.set("v", "<C-,>", "<Esc><C-h>")

vim.keymap.set("n", "gd", function() vim.lsp.buf.definition() end)
vim.keymap.set("n", "<Enter>", function() vim.lsp.buf.definition() end)
vim.keymap.set("n", "gc", function() vim.lsp.buf.declaration() end)
-- vim.keymap.set("n", "gr", function() vim.lsp.buf.references() end)

vim.keymap.set("n", "<C-b>", Build, { desc = " build" })
vim.keymap.set("n", "<leader>rb", Build, { desc = " build" })

-- use ; for commands instead of :
vim.keymap.set("n", ";", ":")
-- vim.keymap.set("n", "<Space>", ":")

-- go back
vim.keymap.set('n', '<bs>', ':edit #<cr>', { silent = true })

-- grep entire project
vim.keymap.set("n", "<C-f>", function()
	require('telescope.builtin').live_grep()
end)

vim.keymap.set("n", "\\o", OpenFiles, { desc = "open file" })
vim.keymap.set("n", "\\d", ":Drex<CR>", { desc = "drex" })
vim.keymap.set("n", "\\f", ":DrexDrawerOpen<CR>", { desc = "filetree" })
vim.keymap.set("n", "<C-b>", ":DrexDrawerToggle<CR>", { desc = "filetree" })
vim.keymap.set("n", "\\t", ShowTerminal, { desc = "terminal" })
vim.keymap.set("n", "<Tab>", ShowTerminal, { desc = "terminal" })
-- vim.keymap.set("n", "<Tab>l", ShowTerminal, { desc = "lf" })

-- emacs style shortcuts in insert mode (yes, i am like that)
vim.keymap.set("i", "<C-n>", "<Down>")
vim.keymap.set("i", "<C-p>", "<Up>")
vim.keymap.set("i", "<C-b>", "<Left>")
vim.keymap.set("i", "<C-f>", "<Right>")
vim.keymap.set("i", "<C-e>", "<End>")
vim.keymap.set("i", "<C-a>", "<Home>")
vim.keymap.set("i", "<C-s>", "<Esc>:write<CR>")
-- move lines up and down in visual mode
vim.keymap.set("x", "K", ":move '<-2<CR>gv-gv")
vim.keymap.set("x", "J", ":move '>+1<CR>gv-gv")
--
-- quote quickly
--vim.keymap.set("i", '<leader>"', '<Esc>viw<Esc>a"<Esc>bi"<Esc>leli')
vim.keymap.set("v", '"', '<Esc>`<i"<Esc>`>ea"<Esc>')
-- substitute shortcut
-- vim.keymap.set("n", "S", ":%s//g<Left><Left>")
-- vim.keymap.set("v", "S", ":s//g<Left><Left>")
-- more reachable line start/end
vim.keymap.set("n", "H", "^")
vim.keymap.set("n", "L", "$")

-- terminal
vim.keymap.set("n", '<C-t>', '<C-w><C-s>:term<CR>i', { remap = false })
vim.keymap.set("t", '<C-\\>', '<C-\\><C-n>', { remap = false })
vim.keymap.set("t", '<C-h>', '<C-\\><C-n><C-w><C-h>', { remap = false })
vim.keymap.set("t", '<C-j>', '<C-\\><C-n><C-w><C-j>', { remap = false })
vim.keymap.set("t", '<C-k>', '<C-\\><C-n><C-w><C-k>', { remap = false })
vim.keymap.set("t", '<C-l>', '<C-\\><C-n><C-w><C-l>', { remap = false })

-- git
vim.keymap.set("n", "<leader>Gb", ":Telescope git_branches<CR>", { desc = "branches" })
vim.keymap.set("n", "<leader>Gc", ":Telescope git_commits<CR>", { desc = "commits" })
vim.keymap.set("n", "<leader>Gs", ":Telescope git_status<CR>", { desc = "status" })

-- go
vim.keymap.set("n", "gr", GoRoot, { desc = "root" })
vim.keymap.set("n", "gp", GoPackagerFile, { desc = "package manifest" })
vim.keymap.set("n", "<leader>gr", GoRoot, { desc = "root" })
vim.keymap.set("n", "<leader>gp", GoPackagerFile, { desc = "package manifest" })
vim.keymap.set("n", "<leader>gs", function()
	require('telescope.builtin').find_files({ cwd = "~/.config/nvim/snippets/", follow = true })
end, { desc = "snippet" })
vim.keymap.set("n", "<leader>gt", function()
	require('telescope.builtin').find_files({ cwd = "~/.config/nvim/templates/", follow = true })
end, { desc = "template" })
vim.keymap.set("n", "<leader>gw", function()
	require('telescope.builtin').find_files({ cwd = "~/wiki/" })
end, { desc = "wiki page" })

vim.keymap.set("n", "<leader>Wo", function()
	require('telescope.builtin').find_files({ cwd = "~/wiki/" })
end, { desc = "open page" })
vim.keymap.set("n", "<leader>Wf", function()
	require('telescope.builtin').live_grep({ cwd = "~/wiki/" })
end, { desc = "find" })

-- list
vim.keymap.set("n", '<leader>lC', '<cmd>Telescope command_history<cr>', { desc = "command history (telescope)" })
vim.keymap.set("n", '<leader>lb', '<cmd>Telescope buffers<cr>', { desc = "buffers…" })
vim.keymap.set("n", '<leader>lc', '<cmd>Telescope commands<cr>', { desc = "commands (telescope)" })
vim.keymap.set("n", '<leader>lf', '<cmd>Telescope filetypes<cr>', { desc = "filetypes…" })
vim.keymap.set("n", '<leader>lm', '<cmd>Telescope marks<cr>', { desc = "marks…" })
vim.keymap.set("n", '<leader>lt', '<cmd>TodoTelescope<cr>', { desc = "todos…" })

-- symbol
vim.keymap.set("n", '<leader>ss', '<cmd>Telescope spell_suggest<cr>', { desc = "spelling" })

-- workspace
vim.keymap.set("n", '<leader>wM', '<cmd>Telescope marks<cr>', { desc = "mark…" })
vim.keymap.set("n", '<leader>wt', '<cmd>TodoTelescope<cr>', { desc = "todo…" })

-- sessions
-- local session_dir = vim.fn.stdpath('data') .. '/sessions/'
-- vim.keymap.set("n", '<leader>mks', ':mks! ' .. session_dir)
-- vim.keymap.set("n", '<leader>lds', ':%bd | so ' .. session_dir)

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
		vim.keymap.set('n', '<leader>D', '<Cmd>Telescope diagnostics<cr>', { desc = "diagnostics" })
		vim.keymap.set('n', '<leader>dd', '<Cmd>FzfLua diagnostics_document<cr>', { desc = "diagnostics" })
		vim.keymap.set('n', '<leader>wd', '<Cmd>Telescope diagnostics<cr>', { desc = "diagnostics" })
	end
})

-- rust
vim.api.nvim_create_autocmd("FileType", { pattern = "rust", callback = function()
end })

return M
