-- WINBAR
-- top bar per window

function WinBar()
	-- local filetype = vim.api.nvim_buf_get_option(0, 'filetype')

	vim.api.nvim_set_hl(0, "WinBar", { fg = "#FFFFFF", bg = "#2222FF" })

	return table.concat {
		"%#WinBar# ",
		vim.fn.fnamemodify(vim.fn.expand("%"), ":.")
	}
end

vim.o.winbar = "%{%v:lua.WinBar()%}"
