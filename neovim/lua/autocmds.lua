-- AUTOCMDS

-- on document write
vim.api.nvim_create_autocmd("BufWrite", { pattern = "*", callback = function()
	vim.cmd [[%s/\s\+$//e]] -- remove trailing whitespace
	vim.cmd [[%s/\n\+\%$//e]] -- remove trailing newlines
end })

-- on vim open
vim.api.nvim_create_autocmd("VimEnter", { pattern = "*", callback = function()
	-- if no args are passed
	if vim.fn.argc() == 0 then
		vim.cmd "enew"
		vim.cmd "setlocal bufhidden=wipe buftype=nofile nocursorcolumn nocursorline nolist nonumber noswapfile norelativenumber"
		vim.cmd([[call append('$', "")]])
	end
end })

-- hide line-bar in insert-mode
vim.api.nvim_create_autocmd("InsertEnter", { pattern = "*", callback = function()
	vim.o.cursorline = false
end })
vim.api.nvim_create_autocmd("InsertLeave", { pattern = "*", callback = function()
	vim.o.cursorline = true
end })

-- only show line-bar on current buffer, on active window
vim.api.nvim_create_autocmd("BufLeave", { pattern = "*", callback = function()
	vim.o.cursorline = false
end })
vim.api.nvim_create_autocmd("BufEnter", { pattern = "*", callback = function()
	vim.o.cursorline = true
end })
vim.api.nvim_create_autocmd("WinLeave", { pattern = "*", callback = function()
	vim.o.cursorline = false
end })
vim.api.nvim_create_autocmd("WinEnter", { pattern = "*", callback = function()
	vim.o.cursorline = true
end })

-- vim.api.nvim_create_autocmd("BufWinEnter", { pattern = "*", callback = function()
-- 	vim.o.wbr = vim.fn.fnamemodify(vim.fn.expand("%"), ":.") -- project directory
-- end })

-- markdown
vim.api.nvim_create_autocmd("FileType", { pattern = "markdown", callback = function()
	vim.opt.autowriteall = true -- ensure write upon leaving a page
	vim.opt.wrap = true -- display lines as one long line
end })

-- build command
vim.api.nvim_create_autocmd("FileType", { pattern = "solidity", callback = function()
	vim.keymap.set("n", "<C-b>", function()
		vim.cmd ':split|terminal forge build'
	end)
end })