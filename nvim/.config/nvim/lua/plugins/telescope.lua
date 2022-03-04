vim.api.nvim_set_keymap("", "<C-t>", ":Telescope<cr>", { noremap = true })
local action_layout = require("telescope.actions.layout")
local previewers = require("telescope.previewers")

vim.cmd([[
  hi TelescopePromptPrefix guifg=None
]])

local small_file_preview_only = function(filepath, bufnr, opts)
  opts = opts or {}

  filepath = vim.fn.expand(filepath)
  vim.loop.fs_stat(filepath, function(_, stat)
    if not stat then
      return
    end
    if stat.size > 100000 then
      return
    else
      previewers.buffer_previewer_maker(filepath, bufnr, opts)
    end
  end)
end
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
    prompt_prefix = " ",
    buffer_previewer_maker = small_file_preview_only,
    mappings = {
      i = {
        ["<esc>"] = "close",
        ["<C-l>"] = action_layout.toggle_preview,
        ["<C-u>"] = false,
      },
    },
  },
  pickers = {
    find_files = {
      find_command = { "fd", "--type", "f", "--strip-cwd-prefix" },
    },
  },
})
