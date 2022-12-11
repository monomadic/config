-- TABLINE
--
-- vim.opt.showtabline = 2 -- show the global tab line at the top of neovim

vim.opt.showtabline = 0 -- show the global tab line at the top of neovim
function TabLine()
	--vim.cmd "highlight PWD guifg=white guibg=#222222"
	return table.concat {
		"%#PWD#",
		vim.fn.fnamemodify(vim.fn.getcwd(), ":~"), -- project directory
		"%#Normal#",
		"%=",
		git_branch(),
	}
end

vim.opt.tabline = "%!v:lua.TabLine()"
-- vim.opt.tabline = "%!render_tabline()"
