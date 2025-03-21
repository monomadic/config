-- COLORS
-- https://ofstack.com/Linux/25280/vim-custom-highlighted-groups-and-some-practical-tips.html

local M = {}
local autocmd = vim.api.nvim_create_autocmd
local hl = vim.api.nvim_set_hl

M.telescope = function()
	local prompt_bg = "#000000"
	local results_bg = "#000000"
	local preview_bg = "#000000"

	hl(0, "TelescopeBorder", { fg = prompt_bg, bg = prompt_bg })
	hl(0, "TelescopePromptBorder", { fg = prompt_bg, bg = prompt_bg })
	hl(0, "TelescopePromptNormal", { fg = "White", bg = prompt_bg })
	hl(0, "TelescopePromptPrefix", { fg = "White" }) -- the icon
	hl(0, "TelescopePromptTitle", { fg = prompt_bg, bg = prompt_bg })
	hl(0, "TelescopePreviewTitle", { fg = preview_bg, bg = preview_bg })
	hl(0, "TelescopePreviewBorder", { fg = preview_bg, bg = preview_bg })
	hl(0, "TelescopePreviewNormal", { bg = preview_bg })
	hl(0, "TelescopeResultsTitle", { fg = results_bg, bg = results_bg })
	hl(0, "TelescopeResultsBorder", { fg = results_bg, bg = results_bg })
	hl(0, "TelescopeResultsNormal", { bg = results_bg })
	hl(0, "TelescopeResultsNormal", { bg = results_bg })
	hl(0, "CursorLine", { bg = results_bg })
end

autocmd({ "ColorScheme", "VimEnter" },
	{
		pattern = "*",
		callback = function()
			local document_grey = "#1E1D3D";
			local light_grey = "#323246";
			local TITLEBAR_BG = "#111111"

			-- local TAB_BG = "#323246"
			-- local title_bar = { bg = dark_grey };
			-- local tabs = { fg = "white", bg = dark_grey };
			-- local tabs_bg = { bg = document_grey };

			local title_bar = { bg = document_grey };
			local tabs = { fg = "white", bg = light_grey };
			local tabs_bg = { bg = "None" };

			hl(0, "ScrollbarHandle", { bg = document_grey })

			-- hl(0, "BarDiagnosticError", { fg = "white", bg = "#F02282" })
			-- hl(0, "BarDiagnosticHint", { fg = "white", bg = "#F0F0AA" })
			-- hl(0, "BarDiagnosticInformation", { fg = "white", bg = "#3070FF" })
			-- hl(0, "BarDiagnosticWarn", { fg = "white", bg = "#FFFF00" })
			-- hl(0, "LineNr", { fg = "#222222" }) -- active
			-- hl(0, "TabLineFill", { fg = "white", bg = "#262639" })
			-- hl(0, "WinBarNC", { fg = "white", bg = "#2222FF" })
			-- hl(0, "WinSeparator", { fg = "bg", bg = "bg" }) -- inactive

			hl(0, "CursorLine", { bg = "black" })
			hl(0, "DiagnosticUnderlineError", { fg = "#F02282", underdotted = true })
			hl(0, "DiagnosticUnderlineWarn", { fg = "#F0F0AA", underdotted = true })
			hl(0, "EndOfBuffer", { fg = "#444444" })
			hl(0, "Float", { bg = "#111111" })
			hl(0, "FloatBorder", { fg = "black", bg = "black" })
			hl(0, "FzfLuaBorder", { fg = "black", bg = "black" })
			hl(0, "FzfLuaNormal", { fg = "white", bg = "black" })
			hl(0, "LspActive", { fg = "#00FF00" })
			hl(0, "NormalFloat", { bg = "black" }) -- terminal?
			hl(0, "StatusLine", {})             -- active
			hl(0, "StatusLineNC", {})           -- inactive
			hl(0, "TabLine", { fg = "white", bg = "black" })
			hl(0, "TabLineFill", { bg = TITLEBAR_BG })
			hl(0, "TodoBgTODO", { bg = "#FFFF00", fg = "black" })
			hl(0, "TodoFgTODO", { fg = "#FFFF00" })
			hl(0, "WinBar", tabs_bg)
			hl(0, "WinBarFileName", tabs)

			hl(0, "FidgetTitle", { fg = "#00FF00" })
			hl(0, "FidgetTask", { fg = "#CC00FF" })

			-- #CCFF00
			-- #FFFF00
			-- #CC88FF
			-- #33CCFF
			-- #CC00FF

			hl(0, "Title", { fg = "#F1FA8C" })
			hl(0, "VimwikiBold", { fg = "#CC88FF" })
			hl(0, "VimwikiItalic", { fg = "#FFFF00" })
			hl(0, "VimwikiLink", { fg = "#33CCFF" })
			--hl(0, "VimwikiLink", { fg = "#3377FF", underdashed = true })
			hl(0, "VimwikiHeaderChar", { link = "Comment" })
			hl(0, "VimwikiCode", { bg = "#222222", fg = "#F1FA8C" })

			hl(0, "MsgArea", { bg = "None" }) -- CommandLine

			M.telescope()
		end
	})

return M
