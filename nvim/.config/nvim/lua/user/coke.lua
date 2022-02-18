local get_hex = require('cokeline/utils').get_hex

local map = vim.api.nvim_set_keymap
map('n', 'H', '<Plug>(cokeline-focus-prev)', { silent = true })
map('n', 'L', '<Plug>(cokeline-focus-next)', { silent = true })
-- map('n', '<Leader>p', '<Plug>(cokeline-switch-prev)', { silent = true })
-- map('n', '<Leader>n', '<Plug>(cokeline-switch-next)', { silent = true })

-- for i = 1,9 do
--   map('n', ('<F%s>'):format(i),      ('<Plug>(cokeline-focus-%s)'):format(i),  { silent = true })
--   map('n', ('<Leader>%s'):format(i), ('<Plug>(cokeline-switch-%s)'):format(i), { silent = true })
-- end

require('cokeline').setup({
  default_hl = {
    focused = {
      -- fg = get_hex("Normal", "fg"),
      -- bg = get_hex("Normal", "bg"),
      -- style = "none",
      fg = "#0F111A",
      bg = "#c8fc0c",
    },
    unfocused = {
      fg = get_hex("Normal", "fg"),
      bg = get_hex("Normal", "bg"),
    },
  },

  rendering = {
    left_sidebar = {
      filetype = 'neo-tree',
      components = {
        {
          text = '  ' .. vim.fn.fnamemodify(vim.fn.getcwd(), ":t"),
          hl = {
            --fg = "white",
            bg = get_hex('NeoTreeNormal', 'bg'),
            style = 'bold'
          }
        },
      }
    },
  },

  components = {
    {
      text = ' ',
      hl = {
        fg = function(buffer)
          return
            buffer.is_modified and yellow or green
        end
      },
    },
    {
      text = function(buffer) return ' ' .. buffer.devicon.icon end,
      -- hl = {
      --   fg = function(buffer) return buffer.devicon.color end,
      -- },
    },
    -- {
    --   text = function(buffer) return buffer.index .. ': ' end,
    -- },
    -- {
    --   text = function(buffer) return buffer.unique_prefix end,
    --   hl = {
    --     fg = get_hex('Comment', 'fg'),
    --     --style = 'italic',
    --   },
    -- },
    {
      text = function(buffer) return buffer.filename .. ' ' end,
      hl = {
        style = function(buffer) return buffer.is_focused and 'bold' or nil end,
      },
    },
    {
      text = ' ',
    },
  },
})
