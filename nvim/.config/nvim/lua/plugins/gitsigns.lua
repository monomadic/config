require("gitsigns").setup({
  signs = {
    -- add          = {hl = 'GitSignsAdd'   , text = '│', numhl='GitSignsAddNr'   , linehl='GitSignsAddLn'},
    add = { hl = "GitSignsAdd", text = "", numhl = "GitSignsAddNr", linehl = "GitSignsAddLn" },
    change = { hl = "GitSignsChange", text = "", numhl = "GitSignsChangeNr", linehl = "GitSignsChangeLn" },
    delete = { hl = "GitSignsDelete", text = "", numhl = "GitSignsDeleteNr", linehl = "GitSignsDeleteLn" },
    topdelete = { hl = "GitSignsDelete", text = "", numhl = "GitSignsDeleteNr", linehl = "GitSignsDeleteLn" },
    changedelete = { hl = "GitSignsChange", text = "~", numhl = "GitSignsChangeNr", linehl = "GitSignsChangeLn" },
  },
  signcolumn = true, -- Toggle with `:Gitsigns toggle_signs`
  numhl = false, -- Toggle with `:Gitsigns toggle_numhl`
  linehl = false, -- Toggle with `:Gitsigns toggle_linehl`
  word_diff = false, -- Toggle with `:Gitsigns toggle_word_diff`
  watch_gitdir = {
    interval = 1000,
    follow_files = true,
  },
  attach_to_untracked = true,
  current_line_blame = false, -- Toggle with `:Gitsigns toggle_current_line_blame`
  current_line_blame_opts = {
    virt_text = true,
    virt_text_pos = "eol", -- 'eol' | 'overlay' | 'right_align'
    delay = 1000,
    ignore_whitespace = false,
  },
  current_line_blame_formatter = "<author>, <author_time:%Y-%m-%d> - <summary>",
  sign_priority = 6,
  update_debounce = 100,
  status_formatter = nil, -- Use default
  max_file_length = 40000,
  preview_config = {
    -- Options passed to nvim_open_win
    border = "single",
    style = "minimal",
    relative = "cursor",
    row = 0,
    col = 1,
  },
  yadm = {
    enable = false,
  },
  on_attach = function(bufnr)
    local gs = package.loaded.gitsigns
    local map = vim.api.nvim_set_keymap
    local opts = { noremap = true, silent = true }

    -- Colors
    vim.cmd([[
      hi GitSignsAdd guifg=#30F951
      hi GitSignsChange guifg=#FF8800
      hi GitSignsDelete guifg=#Fc4f97
    ]])

    -- Navigation
    map("n", "]c", "&diff ? ']c' : '<cmd>Gitsigns next_hunk<CR>'", { expr = true })
    map("n", "[c", "&diff ? '[c' : '<cmd>Gitsigns prev_hunk<CR>'", { expr = true })

    -- Actions
    map("n", "<leader>hs", ":Gitsigns stage_hunk<CR>", opts)
    map("n", "<leader>hr", ":Gitsigns reset_hunk<CR>", opts)
    map("n", "<leader>hS", ":Gitsigns stage_buffer<CR>", opts)
    map("n", "<leader>hu", ":Gitsigns undo_stage_hunk<CR>", opts)
    map("n", "<leader>hR", ":Gitsigns reset_buffer<CR>", opts)
    map("n", "<leader>hp", ":Gitsigns preview_hunk<CR>", opts)
    map("n", "gb", ":Gitsigns blame_line<CR>", opts)

    -- map('n', '<leader>hb', function() gs.blame_line{full=true} end, opts)
    -- map('n', '<leader>tb', gs.toggle_current_line_blame, opts)
    -- map('n', '<leader>hd', gs.diffthis, opts)
    -- map('n', '<leader>hD', function() gs.diffthis('~') end, opts)
    -- map('n', '<leader>td', gs.toggle_deleted, opts)

    -- Text object
    --map({'o', 'x'}, 'ih', ':<C-U>Gitsigns select_hunk<CR>')
  end,
})
