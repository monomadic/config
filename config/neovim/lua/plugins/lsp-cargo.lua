return {
	'saecki/crates.nvim',
	dependencies = { 'nvim-lua/plenary.nvim' },
	event = { 'BufRead Cargo.toml' },

	config = function()
		local crates = require('crates')

		crates.setup {
			autoload = true,
			null_ls = {
				enabled = true,
				name = 'crates.nvim',
			}
		}

		-- markdown
		vim.api.nvim_create_autocmd("BufRead", { pattern = "Cargo.toml",
			callback = function()
				print("read cargo.toml")
			end })

		local buffer = vim.api. nvim_get_current_buf()
		vim.keymap.set('n', 'K', crates.show_popup, { silent = true, buffer = buffer })
		vim.keymap.set('n', '<Space><Space>u', crates.upgrade_crate, { silent = true, buffer = buffer, desc = "upgrade crate" })
		vim.keymap.set('n', '<Space><Space>U', crates.upgrade_all_crates, { silent = true, buffer = buffer, desc = "upgrade all crates" })

		crates.show()
		--print 'loaded crates.nvim'
	end
}
