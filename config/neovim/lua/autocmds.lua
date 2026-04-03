local autocmd = vim.api.nvim_create_autocmd

-- on document write
autocmd("BufWrite",
	{
		pattern = "*",
		callback = function()
			vim.cmd [[%s/\s\+$//e]] -- remove trailing whitespace
			vim.cmd [[%s/\n\+\%$//e]] -- remove trailing newlines
			vim.lsp.buf.format()
		end
	})

-- automatic insert mode when switching to terminal buffers
autocmd("BufEnter",
	{
		pattern = "term://*",
		callback = function()
			vim.cmd.startinsert()
		end
	})

autocmd("TermEnter",
	{
		pattern = "*",
		callback = function()
			-- vim.wo.relativenumber = false -- turn off line numbers
			-- vim.wo.number = false
		end
	})

-- don't list certain types of buffers (quickfix, drex)
autocmd("FileType", {
	pattern = "qf,drex",
	callback = function()
		vim.opt_local.buflisted = false
	end
})

local default_main_files = {
	"src/main.rs",
	"src/lib.rs",
	"main.go",
	"main.c",
	"init.lua",
	"main.py",
	"index.js",
	"src/index.ts",
	"src/index.js",
	"index.md",
	"README.md",
	"doc/README.md",
}

-- on vim open
vim.api.nvim_create_autocmd("VimEnter", {
	callback = function()
		-- if no args are passed
		if vim.fn.argc() == 0 then
			-- vim.cmd "enew"
			-- vim.cmd "setlocal bufhidden=wipe buftype=nofile nocursorcolumn nocursorline nolist nonumber noswapfile norelativenumber"
			-- vim.cmd([[call append('$', "")]])

			local filename
			for _, f in ipairs(default_main_files) do
				if vim.fn.filereadable(f) == 1 then
					filename = f
					break
				end
			end

			if filename then
				local bufnr = vim.fn.bufadd(filename)
				vim.cmd("buffer " .. bufnr)
				-- Detect filetype
				-- manually set the filetype
				-- otherwise lsp + treesitter won't load
				local ft = vim.filetype.match({ filename = filename })
				vim.bo.filetype = ft
			end
		end
	end
})

-- hide line-bar in insert-mode
-- vim.api.nvim_create_autocmd("InsertEnter", { pattern = "*", callback = function()
-- 	vim.o.cursorline = false
-- end })
-- vim.api.nvim_create_autocmd("InsertLeave", { pattern = "*", callback = function()
-- 	vim.o.cursorline = true
-- end })

local function branch_name()
	local branch = vim.fn.system("git branch --show-current 2> /dev/null | tr -d '\n'")
	if branch ~= "" then
		return branch
	else
		return " "
	end
end

vim.api.nvim_create_autocmd({ "FileType", "BufEnter", "FocusGained", "VimEnter" }, {
	callback = function()
		vim.b.branch_name = branch_name()
	end
})

-- only show cursor line-bar on current buffer, on active window
vim.api.nvim_create_autocmd("BufLeave", {
	pattern = "*",
	callback = function()
		vim.o.cursorline = false
	end
})
vim.api.nvim_create_autocmd("BufEnter", {
	pattern = "*",
	callback = function()
		vim.o.cursorline = true
	end
})
vim.api.nvim_create_autocmd("WinLeave", {
	pattern = "*",
	callback = function()
		vim.o.cursorline = false
	end
})
vim.api.nvim_create_autocmd("WinEnter", {
	pattern = "*",
	callback = function()
		vim.o.cursorline = true
	end
})

-- markdown
vim.api.nvim_create_autocmd("FileType", {
	pattern = "markdown",
	callback = function()
		vim.opt.autowriteall = true -- ensure write upon leaving a page
		vim.opt.wrap = true       -- display lines as one long line
		local md = require 'md-headers'
		vim.keymap.set("n", "\\", md.markdown_headers, { desc = "headings" })
	end
})
