-- WINBAR

function WinBar()
	-- local filetype = vim.api.nvim_buf_get_option(0, 'filetype')

	vim.cmd("hi WinBar guifg=#FFFFFF guibg=#2222FF"); -- I think this is the split column

	return table.concat {
		"%#WinBar# ",
		vim.fn.fnamemodify(vim.fn.expand("%"), ":.")
	}
end

vim.o.winbar = "%{%v:lua.WinBar()%}"
