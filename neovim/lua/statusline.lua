-- STATUSLINE
--	per-document status line at bottom of each window
--
--	https://elianiva.my.id/post/neovim-lua-statusline/
--
-- get(b:,'gitsigns_status','')
-- local stl = {
-- 	-- ' %{getcwd()}',
-- 	' %{pathshorten(expand("%:p"))}',
-- 	-- ' %{fnamemodify(expand("%"), ":~:.")}', -- current file
-- 	-- ' %{pathshorten(expand("%"), ":~:.")}',
-- 	'%=',
-- 	-- '  %{FugitiveStatusline()}',
-- 	--' %M', ' %y', ' %r'
-- }

local function lsp_connections()
	local status = ''
	local ids = vim.lsp.buf_get_clients(0)
	vim.cmd "highlight LspIcon guifg=#00DD88"
	for _, client in ipairs(ids) do
		status = status .. "  %#LspIcon#" .. client.name
	end
	return status
end

local function git_branch()
	local git_info = vim.b.gitsigns_status_dict
	if git_info then
		return "%#Normal#  " .. git_info.head
	else
		return ""
	end
end

StatusLine = function()
	-- local filetype = vim.api.nvim_buf_get_option(0, 'filetype')
	return table.concat {
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

vim.cmd [[
  augroup Statusline
  au!
  au WinEnter,BufEnter * setlocal statusline=%!v:lua.StatusLine('active')
  au WinLeave,BufLeave * setlocal statusline=%!v:lua.StatusLine('inactive')
  au WinEnter,BufEnter,FileType NvimTree setlocal statusline=%!v:lua.StatusLine('explorer')
  augroup END
]]