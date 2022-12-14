-- WINBAR
-- top bar per window

function LSPStatus()
  local count = {}
  local levels = {
    errors = "Error",
    warnings = "Warn",
    info = "Info",
    hints = "Hint",
  }

  for k, level in pairs(levels) do
    count[k] = vim.tbl_count(vim.diagnostic.get(0, { severity = level }))
  end

  local errors = ""
  local warnings = ""
  local hints = ""
  local info = ""

  if count["errors"] ~= 0 then
    errors = " %#LspDiagnosticsSignError# " .. count["errors"]
  end
  if count["warnings"] ~= 0 then
    warnings = " %#LspDiagnosticsSignWarning# " .. count["warnings"]
  end
  if count["hints"] ~= 0 then
    hints = " %#LspDiagnosticsSignHint# " .. count["hints"]
  end
  if count["info"] ~= 0 then
    info = " %#LspDiagnosticsSignInformation# " .. count["info"]
  end

  return errors .. warnings .. hints .. info .. "%#Normal#"
end

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
		return table.concat {"%#WinBar#", " "}
	end

	-- local filetype = vim.api.nvim_buf_get_option(0, 'filetype')
	--vim.api.nvim_set_hl(0, "WinBar", { fg = "#FFFFFF", bg = "#2222FF" })

	return table.concat {
		"%#WinBar#",
		"%{%v:lua.FileName()%}",
		-- vim.fn.fnamemodify(vim.fn.expand("%"), ":."),
		-- ":%l",
		-- " %m", -- modified
		-- -- " &modified?'[+]':''", -- modified switch
		-- "%r", -- readonly
		"%=%{%v:lua.LSPStatus()%}",
	}
end

vim.o.winbar = "%{%v:lua.WinBar()%}"
