require 'colors'
-- STATUSLINE
--	per-document status line at bottom of each window
--
--	https://elianiva.my.id/post/neovim-lua-statusline/
--

StatusLine = function()
	return table.concat {
		GitBranch(),
		"%=",
		LSPClients(),
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
