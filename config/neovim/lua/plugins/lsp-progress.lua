-- show lsp progress
return {
	'j-hui/fidget.nvim',
	tag = 'legacy',
	config = function()
		require("fidget").setup {
			text = {
				spinner = "dots",
				commenced = "",
				completed = "",
			},
			fmt = {
				leftpad = true,
				stack_upwards = false,
				max_width = 0,
				fidget = function(fidget_name, spinner)
					return string.format("  %20s %s  ", spinner, fidget_name)
				end,
				task = function(task_name, message, percentage)
					return string.format(
						"  %20s  ",
						message
					--percentage and string.format(" (%.0f%%)", percentage) or "-"
					-- task_name
					)
					-- return string.format(
					-- 	"  %s  ",
					-- 	--message,
					-- 	percentage and string.format(" (%.0f%%)", percentage) or "-"
					-- -- task_name
					-- )
				end,
			},
			timer = {
				spinner_rate = 125,
				fidget_decay = 200,
				task_decay = 200,
			},
			align = { bottom = false },
			window = {
				border = "solid",
				relative = "editor",
				blend = 20,
			},
		}
	end
}
