vim.cmd([[
  hi link NeoTreeDirectoryName Normal
  hi link NeoTreeDirectoryIcon Comment
  hi link NeoTreeIndentMarket Comment
  hi NeoTreeGitModified guifg=#02f9d0
  hi NeoTreeGitAdded guifg=#2bf902
  hi NeoTreeGitUntracked guifg=#02f954
]])

vim.api.nvim_set_keymap("", "<C-p>", ":NeoTreeFloat<CR>", { noremap = true, silent = true })
vim.api.nvim_set_keymap("", "<C-b>", ":NeoTreeShowToggle<CR>", { noremap = true, silent = true })

require("neo-tree").setup({
  close_if_last_window = false, -- Close Neo-tree if it is the last window left in the tab
  popup_border_style = "rounded",
  enable_git_status = true,
  enable_diagnostics = true,
  default_component_configs = {
    indent = {
      with_markers = false,
      -- indent_size = 2,
      -- padding = 1, -- extra padding on left hand side
      -- with_markers = true,
      -- indent_marker = "│",
      -- last_indent_marker = "└",
      -- highlight = "NeoTreeIndentMarker",
    },
    icon = {
      folder_closed = "",
      folder_open = "",
      folder_empty = "ﰊ",
      default = "*",
    },
    name = {
      trailing_slash = false,
      use_git_status_colors = true,
    },
    git_status = {
      highlight = "NeoTreeDimText", -- if you remove this the status will be colorful
    },
  },

  filesystem = {
    filters = { --These filters are applied to both browsing and searching
      show_hidden = false,
      respect_gitignore = true,
    },
    follow_current_file = true, -- This will find and focus the file in the active buffer every
    -- time the current file is changed while the tree is open.
    use_libuv_file_watcher = false, -- This will use the OS level file watchers
    -- to detect changes instead of relying on nvim autocmd events.
    hijack_netrw_behavior = "open_default", -- netrw disabled, opening a directory opens neo-tree
    -- in whatever position is specified in window.position
    -- "open_split",  -- netrw disabled, opening a directory opens within the
    -- window like netrw would, regardless of window.position
    -- "disabled",    -- netrw left alone, neo-tree does not handle opening dirs

    window = {
      position = "left",
      width = 40,
      mappings = {
        ["<1-LeftMouse>"] = "open",
        ["<cr>"] = "open",
        ["l"] = "open",
        ["S"] = "open_split",
        ["s"] = "open_vsplit",
        ["C"] = "close_node",
        ["<bs>"] = "navigate_up",
        ["."] = "set_root",
        ["H"] = "toggle_hidden",
        ["I"] = "toggle_gitignore",
        ["R"] = "refresh",
        ["/"] = "fuzzy_finder",
        --["/"] = "filter_as_you_type", -- this was the default until v1.28
        --["/"] = "none" -- Assigning a key to "none" will remove the default mapping
        ["f"] = "filter_on_submit",
        ["<c-x>"] = "clear_filter",
        ["a"] = "add",
        ["d"] = "delete",
        ["r"] = "rename",
        ["c"] = "copy_to_clipboard",
        ["x"] = "cut_to_clipboard",
        ["p"] = "paste_from_clipboard",
        ["q"] = "close_window",
      },
    },
  },

  buffers = {
    show_unloaded = true,
    window = {
      position = "left",
      mappings = {
        ["<1-LeftMouse>"] = "open",
        ["<cr>"] = "open",
        ["l"] = "open",
        ["S"] = "open_split",
        ["s"] = "open_vsplit",
        ["<bs>"] = "navigate_up",
        ["."] = "set_root",
        ["R"] = "refresh",
        ["a"] = "add",
        ["d"] = "delete",
        ["r"] = "rename",
        ["c"] = "copy_to_clipboard",
        ["x"] = "cut_to_clipboard",
        ["p"] = "paste_from_clipboard",
        ["bd"] = "buffer_delete",
      },
    },
  },

  git_status = {
    window = {
      position = "float",
      mappings = {
        ["<1-LeftMouse>"] = "open",
        ["<CR>"] = "open",
        ["l"] = "open",
        ["S"] = "open_split",
        ["s"] = "open_vsplit",
        ["C"] = "close_node",
        ["R"] = "refresh",
        ["d"] = "delete",
        ["r"] = "rename",
        ["c"] = "copy_to_clipboard",
        ["x"] = "cut_to_clipboard",
        ["p"] = "paste_from_clipboard",
        ["A"] = "git_add_all",
        ["gu"] = "git_unstage_file",
        ["ga"] = "git_add_file",
        ["gr"] = "git_revert_file",
        ["gc"] = "git_commit",
        ["gp"] = "git_push",
        ["gg"] = "git_commit_and_push",
      },
    },
  },
})
