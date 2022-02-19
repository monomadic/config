vim.api.nvim_set_keymap("", "<C-t>", ":Telescope<cr>", { noremap = true })

dropdown_theme = require("telescope.themes").get_dropdown({
  previewer = false,
  prompt_title = "",
  results_height = 16,
  width = 0.6,
  borderchars = {
    { "─", "│", "─", "│", "╭", "╮", "╯", "╰" },
    prompt = { "─", "│", " ", "│", "╭", "╮", "│", "│" },
    results = { "─", "│", "─", "│", "├", "┤", "╯", "╰" },
    preview = { "─", "│", "─", "│", "╭", "╮", "╯", "╰" },
  },
})

require("telescope").setup({
  defaults = {
    mappings = {
      i = {
        ["<esc>"] = "close",
      },
    },
  },
})
