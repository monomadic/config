require("sidebar-nvim").setup({
    disable_default_keybindings = 0,
    bindings = nil,
    open = false,
    side = "right",
    initial_width = 50,
    hide_statusline = true,
    update_interval = 1000,
    sections = { "buffers", "git", "diagnostics", "symbols", "todos" },
    section_separator = {"", ""},
    containers = {
        attach_shell = "/bin/sh", show_all = true, interval = 5000,
    },
    todos = { ignored_paths = { "~" }, initially_closed = false, },
    buffers = {
        icon = "",
        ignored_buffers = {"%[(.*)]$"}, -- ignore buffers by regex
        sorting = "name", -- alternatively set it to "name" to sort by buffer name instead of buf id
        show_numbers = true, -- whether to also show the buffer numbers
    }
})

vim.api.nvim_set_keymap("", "<C-m>", ":SidebarNvimToggle<CR>", { noremap = true, silent = true })
