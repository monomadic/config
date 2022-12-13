-- KEYMAPS
--
--	to view current mappings: :verbose nmap <C-]>
--

-- save / write
vim.keymap.set("n", "<C-s>", "<CMD>write<CR>");
vim.keymap.set({ "v", "i" }, "<C-s>", "<Esc><Cmd>write<CR>");

-- buffer unload
vim.keymap.set("n", "q", "<CMD>bun<CR>");

-- leader keys
-- vim.keymap.set("n", "<leader>s", "<cmd>write<CR>")
vim.keymap.set("n", "<leader>ww", "<cmd>wq!<CR>")
vim.keymap.set("n", "<leader>wq", "<cmd>wq<CR>")
vim.keymap.set("n", "<leader>q", "<cmd>quit<CR>")
vim.keymap.set("n", "<leader>!", "<cmd>quit!<CR>")

-- vim.keymap.set("n", "<leader>j", "<cmd>quit!<CR>") -- jump

vim.keymap.set("n", "<leader><tab>", "<cmd>Drex<CR>")
vim.keymap.set("n", "Q", "<cmd>quit<CR>")
vim.keymap.set("n", "WQ", "<cmd>wq<CR>")
--vim.keymap.set("n", "<leader>lf", "<cmd>Lf<CR>")
--vim.keymap.set("n", "<leader>lg", "<cmd>LazyGit<CR>")
vim.keymap.set("n", "<leader>h1", "<Esc>/1.<CR>")
vim.keymap.set("n", "<leader>h2", "<Esc>/2.<CR>")
vim.keymap.set("n", "<leader>n", function()
	vim.wo.relativenumber = false -- turn off line numbers
	vim.wo.number = false
end)

vim.keymap.set("n", '<leader>df',
	function()
		vim.lsp.buf.format { async = true }
	end
)

-- split navigation
vim.keymap.set("n", "<C-j>", "<C-w><C-j>")
vim.keymap.set("i", "<C-j>", "<Esc><C-w><C-j>")
vim.keymap.set("n", "<C-k>", "<C-w><C-k>", { remap = false })
vim.keymap.set("i", "<C-k>", "<Esc><C-w><C-k>")
vim.keymap.set("n", "<C-l>", "<C-w><C-l>", { remap = false })
vim.keymap.set("i", "<C-l>", "<Esc><C-w><C-l>")
vim.keymap.set("n", "<C-h>", "<C-w><C-h>")
vim.keymap.set("i", "<C-h>", "<Esc><C-w><C-h>")
vim.keymap.set("n", "<C-w><C-d>", "<cmd>vsplit<CR>")

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

-- use ; for commands instead of :
vim.keymap.set("n", ";", ":")
-- vim.keymap.set("n", "<Space>", ":")

-- go back
vim.keymap.set('n', '<bs>', ':edit #<cr>', { silent = true })

-- grep entire project
vim.keymap.set("n", "<C-f>", function()
	require('telescope.builtin').live_grep()
end)

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
-- vim.keymap.set("t", '<C-Space>', '<C-\\><C-n>', { remap = false })
vim.keymap.set("t", '<C-h>', '<C-\\><C-n><C-w><C-h>', { remap = false })
vim.keymap.set("t", '<C-j>', '<C-\\><C-n><C-w><C-j>', { remap = false })
vim.keymap.set("t", '<C-k>', '<C-\\><C-n><C-w><C-k>', { remap = false })
vim.keymap.set("t", '<C-l>', '<C-\\><C-n><C-w><C-l>', { remap = false })

local function file_exists(fname)
	local stat = vim.loop.fs_stat(fname)
	return (stat and stat.type) or false
end

-- go to root project file
vim.keymap.set("n", "gI", function() -- TODO: convert to array
	if file_exists("src/lib.rs") then
		vim.cmd ':edit src/lib.rs'
	elseif file_exists("src/main.rs") then
		vim.cmd ':edit src/main.rs'
	elseif file_exists("index.md") then
		vim.cmd ':edit index.md'
	elseif file_exists("src/index.ts") then
		vim.cmd ':edit src/index.ts'
	elseif file_exists("init.lua") then
		vim.cmd ':edit init.lua'
	else
		print("no root file found.")
	end
end)

vim.keymap.set("n", "gP", function()
	if file_exists("Cargo.toml") then
		vim.cmd ':edit Cargo.toml'
	elseif file_exists("package.json") then
		vim.cmd ':edit package.json'
	else
		print("no package manifest found.")
	end
end)

-- sessions
-- local session_dir = vim.fn.stdpath('data') .. '/sessions/'
-- vim.keymap.set("n", '<leader>mks', ':mks! ' .. session_dir)
-- vim.keymap.set("n", '<leader>lds', ':%bd | so ' .. session_dir)
