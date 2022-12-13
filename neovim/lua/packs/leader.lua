return {
	'folke/which-key.nvim', config = function()
		-- vim.g.timeoutlen=100
		-- vim.g.ttimeoutlen=100

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
			b = "buffers...",
			f = {
				name = "file",
				f = { function() vim.lsp.buf.format { async = true } end, " format" },
				o = { function()
					require('telescope.builtin').find_files { path_display = { "truncate" }, prompt_title = "", preview_title = "" }
				end, "open" },
			},
			d = {
				name = "document",
				f = { function() vim.lsp.buf.format { async = true } end, "format" }
			},
			g = {
				name = "go",
				r = { GoRoot, "root" },
				p = { GoPackagerFile, "package manifest" }
			},


			j = {
				name = "jump",
				e = { function()
					require('telescope.builtin').lsp_document_symbols { symbols = "enum" }
				end, "enum" },
				E = { function()
					require('telescope.builtin').lsp_workspace_symbols { symbols = "enum" }
				end, "enum (workspace)" },

				f = { function()
					require('telescope.builtin').lsp_document_symbols { symbols = "function" }
				end, "function" },
				F = { function()
					require('telescope.builtin').lsp_workspace_symbols { symbols = "function" }
				end, "function (workspace)" },

				m = { function()
					require('telescope.builtin').lsp_document_symbols { symbols = "module" }
				end, "module" },
				M = { function()
					require('telescope.builtin').lsp_workspace_symbols { symbols = "module" }
				end, "module (workspace)" },

				s = { function()
					require('telescope.builtin').lsp_document_symbols { symbols = "struct" }
				end, "struct" },
				S = { function()
					require('telescope.builtin').lsp_workspace_symbols { symbols = "struct" }
				end, "struct (workspace)" },

				a = { ":FzfLua lsp_document_symbols<CR>", "symbol (all types)" },
				A = { ":FzfLua lsp_workspace_symbols<CR>", "symbol (all types, workspace)" },
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

			s = {
				name = "symbol",
				a = { ":Lspsaga code_action<CR>", "code actions" }
			},
			t = { function() ShowTerminal() end, "terminal" },
			w = { name = "workspace",
				f = { function()
					require('telescope.builtin').lsp_workspace_symbols { symbols = "function", prompt_title = "", preview_title = "" }
				end, "function" },
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

		print("WhichKey loaded.")
	end
}
