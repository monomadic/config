return {
  "klen/nvim-test",
  config = function()
    require('nvim-test').setup()

		-- vim.keymap.set("n", "\\", ':TestSuite<CR>',  { desc = "tests (all)", silent = true })
		vim.keymap.set("n", "|", ':TestSuite<CR>',  { desc = "tests (all)", silent = true })
  end
}
