-- https://github.com/jackMort/ChatGPT.nvim
return {
	"jackMort/ChatGPT.nvim",
	config = function()
		require("chatgpt").setup {
			yank_register = "c",
			api_key_cmd = "op read op://rob/openai/api-key --no-newline",
		}
	end,
	dependencies = {
		"MunifTanjim/nui.nvim",
		"nvim-lua/plenary.nvim",
		"nvim-telescope/telescope.nvim"
	}
}
