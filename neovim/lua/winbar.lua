-- WINBAR
-- top bar per window

local utils = require('utils')

function FileName()
	local filetype = vim.bo.filetype

	if filetype == "drex" or filetype == "" then
		return ""
	else
		return string.format("  %s %s:%%l %%m %%r", utils.get_icon(filetype), vim.fn.fnamemodify(vim.fn.expand("%"), ":."))
	end
end

function WinBar()
	local filetype = vim.bo.filetype
	if filetype == "drex" then
		return table.concat { "%#Normal#" }
	end

	if filetype == "aerial" then
		return table.concat { "%#Normal#" }
		--return table.concat { "%#Winbar#", " symbols" }
	end

	if filetype == "Trouble" then
		return table.concat { "%#WinBarFileName#" .. "  󱍧 diagnostics  " .. "%#Normal#" }
		--return table.concat { "%#Winbar#", " symbols" }
	end

	return table.concat {
		"%#WinBar#",
		"%#WinBarFileName#",
		"%{%v:lua.FileName()%}",
		"%#WinBar#",
		" %{%v:lua.LSPWorkspaceDiagnostics(0)%}",
		-- vim.fn.fnamemodify(vim.fn.expand("%"), ":."),
		-- ":%l",
		-- " %m", -- modified
		-- -- " &modified?'[+]':''", -- modified switch
		-- "%r", -- readonly
		-- " %#Normal#",
		-- "%=",
		"%=",
	}
end

vim.o.winbar = "%{%v:lua.WinBar()%}"
