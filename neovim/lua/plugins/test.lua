return {
  "klen/nvim-test",
  config = function()
    require('nvim-test').setup()

		vim.keymap.set("n", "<leader>Rt", ':TestSuite<CR>',  { desc = "tests (all)", silent = true })
  end
}
