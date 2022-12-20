-- WINBAR
-- top bar per window

function FileName()
	local filetype = vim.bo.filetype

	if filetype == "drex" or filetype == "" then
		return ""
	else
		return string.format(" %s:%%l %%m %%r ", vim.fn.fnamemodify(vim.fn.expand("%"), ":."))
	end
end

function WinBar()
	local filetype = vim.bo.filetype
	if filetype == "drex" then
		return table.concat {"%#Normal#"}
	end

	return table.concat {
		"%#WinBar#",
		"%{%v:lua.FileName()%}",
		-- vim.fn.fnamemodify(vim.fn.expand("%"), ":."),
		-- ":%l",
		-- " %m", -- modified
		-- -- " &modified?'[+]':''", -- modified switch
		-- "%r", -- readonly
		-- " %#Normal#",
		-- "%=",
		"%=%{%v:lua.LSPWorkspaceDiagnostics(0)%}",
	}
end

vim.o.winbar = "%{%v:lua.WinBar()%}"
