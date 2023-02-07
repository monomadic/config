return {
	"jackMort/ChatGPT.nvim",
	config = function()
		vim.keymap.set("n", "<leader><leader>", "<cmd>ChatGPT<CR>", { desc = " ChatGPT" })

		require("chatgpt").setup({})
	end,
	dependencies = {
		"MunifTanjim/nui.nvim",
		"nvim-lua/plenary.nvim",
		"nvim-telescope/telescope.nvim"
	}
}
