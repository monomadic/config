-- you can configure Hop the way you like here; see :h hop-config
require("hop").setup({ keys = "etovxqpdygfblzhckisuran" })
local opts = { noremap = true, silent = true }

vim.api.nvim_set_keymap("n", "/", ":HopPattern<CR>", {})
vim.api.nvim_set_keymap("n", "f", ":HopChar1<CR>", opts)
vim.api.nvim_set_keymap("n", "F", ":HopChar2<CR>", opts)
vim.api.nvim_set_keymap("n", "s", ":HopWord<CR>", opts)
vim.api.nvim_set_keymap("i", "<C-f>", "<cmd>HopWord<CR>", opts)

-- vim.api.nvim_set_keymap(
--   "n",
--   "f",
--   "<cmd>lua require'hop'.hint_char1({ direction = require'hop.hint'.HintDirection.AFTER_CURSOR, current_line_only = true })<cr>",
--   opts
-- )
-- vim.api.nvim_set_keymap(
--   "n",
--   "F",
--   "<cmd>lua require'hop'.hint_char1({ direction = require'hop.hint'.HintDirection.BEFORE_CURSOR, current_line_only = true })<cr>",
--   opts
-- )
-- vim.api.nvim_set_keymap(
--   "o",
--   "f",
--   "<cmd>lua require'hop'.hint_char1({ direction = require'hop.hint'.HintDirection.AFTER_CURSOR, current_line_only = true, inclusive_jump = true })<cr>",
--   opts
-- )
-- vim.api.nvim_set_keymap(
--   "o",
--   "F",
--   "<cmd>lua require'hop'.hint_char1({ direction = require'hop.hint'.HintDirection.BEFORE_CURSOR, current_line_only = true, inclusive_jump = true })<cr>",
--   opts
-- )
vim.api.nvim_set_keymap(
  "",
  "t",
  "<cmd>lua require'hop'.hint_char1({ direction = require'hop.hint'.HintDirection.AFTER_CURSOR, current_line_only = true })<cr>",
  opts
)
vim.api.nvim_set_keymap(
  "",
  "T",
  "<cmd>lua require'hop'.hint_char1({ direction = require'hop.hint'.HintDirection.BEFORE_CURSOR, current_line_only = true })<cr>",
  opts
)
