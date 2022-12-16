require 'colors'
-- STATUSLINE
--	per-document status line at bottom of each window
--
--	https://elianiva.my.id/post/neovim-lua-statusline/
--

local function lsp_connections()
	local status = ''
	local ids = vim.lsp.get_active_clients()

	for _, client in ipairs(ids) do
		if vim.lsp.buf_is_attached(0, client.id) then
			status = status .. "%#LspActive# " .. client.name .. " "
		else
			status = status .. "%#LspInactive# " .. client.name .. " "
		end
	end
	return status .. "%#Normal#"
end

local function git_branch()
	local current_branch = vim.fn.system("git branch --show-current 2> /dev/null | tr -d '\n'")
	if current_branch then
		return "%#GitBranch#  " .. current_branch
	else
		return ""
	end
end

StatusLine = function()
	-- local filetype = vim.api.nvim_buf_get_option(0, 'filetype')
	local diag = LSPDiagnostics(nil)
	return table.concat {
		diag,
		"%#Directory#",
		vim.fn.fnamemodify(vim.fn.getcwd(), ":~"), -- project directory
		"%#Normal#",
		-- "  ",
		-- vim.fn.fnamemodify(vim.fn.expand("%"), ":."), -- project directory
		"%=",
		-- filetype,
		lsp_connections(),
		" ",
		git_branch()
	}
end

vim.opt.statusline = "%!v:lua.StatusLine()"

-- vim.cmd [[
--   augroup Statusline
--   au!
--   au WinEnter,BufEnter * setlocal statusline=%!v:lua.StatusLine('active')
--   au WinLeave,BufLeave * setlocal statusline=%!v:lua.StatusLine('inactive')
--   au WinEnter,BufEnter,FileType NvimTree setlocal statusline=%!v:lua.StatusLine('explorer')
--   augroup END
-- ]]
