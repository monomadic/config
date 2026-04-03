-- Enhanced f/t motions
-- https://github.com/ggandor/flit.nvim
--
-- multi-line jump with f/F (smart case)
-- press f/F/enter again to jump next
return {
	'ggandor/flit.nvim',
	enabled = false,
	config = function()
		require('flit').setup {
			keys = { f = 'f', F = 'F', t = 't', T = 'T' },
			-- A string like "nv", "nvo", "o", etc.
			labeled_modes = "v",
			multiline = true,
			-- Like `leap`s similar argument (call-specific overrides).
			-- E.g.: opts = { equivalence_classes = {} }
			opts = {}
		}
	end
}
