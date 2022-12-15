return { 'folke/which-key.nvim', config = function()

		require("which-key").setup {
			window = {
				border = "single",
				position = "top",
			},
			key_labels = {
				["<CR>"] = "",
				["<Tab>"] = "",
			},
			popup_mappings = {
				scroll_down = '<c-n>', -- binding to scroll down inside the popup
				scroll_up = '<c-p>', -- binding to scroll up inside the popup
			},
		}

		require("which-key").register({
			f = {
				name = "file",
				f = { function() vim.lsp.buf.format { async = true } end, " format" },
				o = { function()
					require('telescope.builtin').find_files { path_display = { "truncate" }, prompt_title = "", preview_title = "" }
				end, "open" },
			},
			d = {
				name = "document",

				e = { function()
					require('telescope.builtin').lsp_document_symbols { symbols = "enum" }
				end, " enums…" },

				f = { function()
					require('telescope.builtin').lsp_document_symbols { symbols = "function" }
				end, " functions…" },
				F = { function() vim.lsp.buf.format { async = true } end, " format" },

				s = { function()
					require('telescope.builtin').lsp_document_symbols { symbols = "struct" }
				end, " structs…" },
				S = { require('telescope.builtin').lsp_document_symbols, " symbols…" },
				m = { function()
					require('telescope.builtin').lsp_document_symbols { symbols = "module" }
				end, " modules…" },
			},
			g = {
				name = "go",
			},

			l = {
				name = "list",
				c = { ":FzfLua commands<CR>", "commands" },
				h = { ":FzfLua command_history<CR>", "command history" },
				H = { ":FzfLua highlights<CR>", "highlight colors" },
				k = { ":FzfLua keymaps<CR>", "keymaps" },
				r = { ":FzfLua oldfiles<CR>", "recent files" },
				t = { ":FzfLua colorschemes<CR>", "themes" },
			},

			p = { name = "peek" },
			r = { name = "run",
				-- d = { "", "debug" }
			},
			s = {
				name = "symbol",
				h = {vim.lsp.buf.signature_help, "help"}
			},
			t = { ShowTerminal, " terminal" },
			w = { name = "workspace",
				e = { function()
					require('telescope.builtin').lsp_workspace_symbols { symbols = "enum" }
				end, " enums…" },
				f = { function()
					require('telescope.builtin').lsp_workspace_symbols { symbols = "function", prompt_title = "", preview_title = "" }
				end, " functions…" },
				m = { function()
					require('telescope.builtin').lsp_workspace_symbols { symbols = "module" }
				end, " modules…" },
				s = { function()
					require('telescope.builtin').lsp_workspace_symbols { symbols = "struct" }
				end, " structs…" },
				S = { ":FzfLua lsp_workspace_symbols<CR>", " symbols…" },
			}
		}, {
			prefix = "<leader>",
		})

		vim.api.nvim_set_hl(0, "WhichKey", { fg = "#FFFF00" })
		vim.api.nvim_set_hl(0, "WhichKeyDesc", { bg = "none" })
		vim.api.nvim_set_hl(0, "WhichKeyFloat", { bg = "black" })
		vim.api.nvim_set_hl(0, "WhichKeyBorder", { bg = "black", fg = "#222222" })
		vim.api.nvim_set_hl(0, "WhichKeyGroup", { fg = "#00FFFF" })
		vim.api.nvim_set_hl(0, "WhichKeySeparator", { fg = "#888888" })

		print("whichkey loaded")
	end
}
