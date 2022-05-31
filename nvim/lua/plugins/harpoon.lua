local opts = { noremap = true, silent = true }
local keymap = vim.api.nvim_set_keymap

keymap("n", "gm", ":lua require('harpoon.ui').toggle_quick_menu()<CR>", opts)
keymap("n", "ml", ":lua require('harpoon.ui').toggle_quick_menu()<CR>", opts)
keymap("n", "mm", ":lua require('harpoon.mark').add_file()<CR>", opts)
keymap("n", "mt", ":Telescope harpoon marks<CR>", opts)

keymap("n", "m]", ":lua require('harpoon.ui').nav_next()<CR>", opts)
--keymap("n", "<C-]>", ":lua require('harpoon.ui').nav_next()<CR>", opts)
keymap("n", "m[", ":lua require('harpoon.ui').nav_prev()<CR>", opts)
--keymap("n", "<C-[>", ":lua require('harpoon.ui').nav_prev()<CR>", opts)

keymap("n", "M1", ":lua require('harpoon.mark').set_current_at(1)<CR>", opts)
keymap("n", "M2", ":lua require('harpoon.mark').set_current_at(2)<CR>", opts)
keymap("n", "M3", ":lua require('harpoon.mark').set_current_at(3)<CR>", opts)
keymap("n", "M4", ":lua require('harpoon.mark').set_current_at(4)<CR>", opts)
keymap("n", "M5", ":lua require('harpoon.mark').set_current_at(5)<CR>", opts)
keymap("n", "M6", ":lua require('harpoon.mark').set_current_at(6)<CR>", opts)

keymap("n", "m1", ":lua require('harpoon.ui').nav_file(1)<CR>", opts)
keymap("n", "m2", ":lua require('harpoon.ui').nav_file(2)<CR>", opts)
keymap("n", "m3", ":lua require('harpoon.ui').nav_file(3)<CR>", opts)
keymap("n", "m4", ":lua require('harpoon.ui').nav_file(4)<CR>", opts)
keymap("n", "m5", ":lua require('harpoon.ui').nav_file(5)<CR>", opts)
keymap("n", "m6", ":lua require('harpoon.ui').nav_file(6)<CR>", opts)
keymap("n", "m7", ":lua require('harpoon.ui').nav_file(7)<CR>", opts)
keymap("n", "m8", ":lua require('harpoon.ui').nav_file(8)<CR>", opts)
keymap("n", "m9", ":lua require('harpoon.ui').nav_file(9)<CR>", opts)

keymap("n", "t1", ":lua require('harpoon.term').gotoTerminal(1)<CR>", opts)
keymap("n", "t2", ":lua require('harpoon.term').gotoTerminal(2)<CR>", opts)
keymap("n", "t3", ":lua require('harpoon.term').gotoTerminal(3)<CR>", opts)
keymap("n", "t4", ":lua require('harpoon.term').gotoTerminal(4)<CR>", opts)
keymap("n", "t5", ":lua require('harpoon.term').gotoTerminal(5)<CR>", opts)
keymap("n", "t6", ":lua require('harpoon.term').gotoTerminal(6)<CR>", opts)
