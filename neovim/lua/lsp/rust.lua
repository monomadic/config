-- rust-tools: a rust lsp server specific to rust
-- https://github.com/simrat39/rust-tools.nvim
return {
	'simrat39/rust-tools.nvim',
	ft = 'rust',
	requires = { 'neovim/nvim-lspconfig', 'nvim-lua/plenary.nvim', 'mfussenegger/nvim-dap' }, -- last 2 for debug
	config = function()
		local rust_tools = require('rust-tools')

		rust_tools.setup({
			tools = {
				autosethints = true,
				runnables = {
					use_telescope = true
				},
				inlay_hints = {
					auto = true,
					show_parameter_hints = true,
				},
				hover_actions = { auto_focus = false },
			},

			-- see https://github.com/neovim/nvim-lspconfig/blob/master/CONFIG.md#rust_analyzer
			server = {
				on_attach = function(client, bufnr)
					require("lsp-format").on_attach(client)
					-- require("virtualtypes").on_attach(client, bufnr)

					require("which-key").register({
						R = {
							name = "Rust",
							r = { ":RustRunnables", "runnables" },
						}
					})

					-- vim.keymap.set("n", "<leader>R", ":RustCodeAction<CR>", { desc = "RUST" })

					vim.keymap.set("n", "K", rust_tools.hover_actions.hover_actions, { buffer = bufnr })
					vim.keymap.set("n", "<leader>sa", ":RustCodeAction<CR>",
						{ buffer = bufnr, desc = " code action", remap = false })
					vim.keymap.set("n", "<leader>a", rust_tools.code_action_group.code_action_group,
						{ buffer = bufnr, desc = " code action" })
					vim.keymap.set('n', '<leader>gp', rust_tools.open_cargo_toml.open_cargo_toml,
						{ buffer = bufnr, desc = " cargo.toml", remap = false })
					vim.keymap.set('n', '<leader>gu', rust_tools.parent_module.parent_module,
						{ buffer = bufnr, desc = " up (parent module)" })
					-- vim.keymap.set('v', '<C-j>', rust_tools.move_item.move_item(false), { buffer = bufnr }) -- down
					-- vim.keymap.set('v', '<C-k>', rust_tools.move_item.move_item(true), { buffer = bufnr }) -- up
					-- vim.keymap.set("n", "gi", function()
					-- 	vim.cmd ':edit src/lib.rs'
					-- end)

					vim.keymap.set("n", "<leader>gd", ":RustOpenExternalDocs<CR>", { buffer = bufnr, desc = " open docs" })

					vim.keymap.set("n", "<leader>se", ":RustExpand<CR>", { buffer = bufnr, desc = " expand" })
					vim.keymap.set("n", "<leader>sE", ":RustExpandMacro<CR>", { buffer = bufnr, desc = " expand macro" })

					vim.keymap.set("n", "<C-b>", ":RustRun<CR>", { buffer = bufnr, desc = " run" })

					-- vim.keymap.set("n", "<leader>r", "", { buffer = bufnr, desc = " rust" })
					vim.keymap.set("n", "<leader>rR", ":RustRunnables<CR>", { buffer = bufnr, desc = " run" })
					vim.keymap.set("n", "<leader>rr", ":RustRunnables<CR>", { buffer = bufnr, desc = " runnables…" })
					vim.keymap.set("n", "<leader>rd", ":RustDebuggables<CR>", { buffer = bufnr, desc = " debuggables…" })

					vim.keymap.set("n", "<leader>df", ":RustFmt<CR>", { buffer = bufnr, desc = " rustfmt" })
				end,
				settings = {
					-- to enable rust-analyzer settings visit:
					-- https://github.com/rust-analyzer/rust-analyzer/blob/master/docs/user/generated_config.adoc
					["rust-analyzer"] = {
						lens = { enable = true },
						-- hover = {
						-- },
						checkOnSave = {
							allFeatures = true,
						},
					}
				}
			},
		})
	end
}
