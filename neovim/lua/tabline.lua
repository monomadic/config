-- TABLINE
--	topmost bar

vim.opt.showtabline = 2 -- show the global tab line at the top of neovim
function TabLine()
	return table.concat {
		" %#TabLineDir#",
		vim.fn.fnamemodify(vim.fn.getcwd(), ":~"), -- project directory
		"%=",
		"%#TablineDiagnostics#",
		LSPWorkspaceDiagnostics(nil),
		-- "%#TablineLSPClients#",
		-- LSPClients(),
	}
end

vim.opt.tabline = "%!v:lua.TabLine()"
