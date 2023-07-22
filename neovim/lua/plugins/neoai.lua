-- https://github.com/Bryley/neoai
return {
	"Bryley/neoai.nvim",
	dependencies = {
		"MunifTanjim/nui.nvim",
	},
	cmd = {
		"NeoAI",
		"NeoAIOpen",
		"NeoAIClose",
		"NeoAIToggle",
		"NeoAIContext",
		"NeoAIContextOpen",
		"NeoAIContextClose",
		"NeoAIInject",
		"NeoAIInjectCode",
		"NeoAIInjectContext",
		"NeoAIInjectContextCode",
	},
	keys = {
		{ "<leader>As", desc = "summarize text" },
		{ "<leader>Ag", desc = "generate git message" },
		{ "<leader>Am", desc = "format markdown" },
	},
	config = function()
		require("neoai").setup {
			inject = {
				cutoff_width = 4096,
			},
			-- inject = {
			-- 	cutoff_width = nil, -- do not wrap text
			-- },
			shortcuts = {

				{
					name = "format-markdown",
					key = "<leader>Am",
					desc = "format markdown with AI",
					use_context = true,
					prompt = [[
											Please format this markdown text correctly. Fix spelling and
											punctuation errors, remove any page numbers, capitalize headings,
											remove excess whitespace. Ensure that paragraphs do not contain newlines or line breaks.
											Do not attempt to wrap text.
									]],
					modes = { "v" },
					strip_function = nil,
				},
				{
					name = "textify",
					key = "<leader>As",
					desc = "fix text with AI",
					use_context = true,
					prompt = [[
											Please rewrite the text to make it more readable, clear,
											concise, and fix any grammatical, punctuation, or spelling
											errors
									]],
					modes = { "v" },
					strip_function = nil,
				},
				{
					name = "gitcommit",
					key = "<leader>Ag",
					desc = "generate git commit message",
					use_context = false,
					prompt = function()
						return [[
													Using the following git diff generate a consise and
													clear git commit message, with a short title summary
													that is 75 characters or less:
											]] .. vim.fn.system("git diff --cached")
					end,
					modes = { "n" },
					strip_function = nil,
				},
			},
		}
	end,
}
