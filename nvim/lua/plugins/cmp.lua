local cmp = require'cmp'
local compare = require('cmp.config.compare')
--local luasnip = require('luasnip')

--require("luasnip/loaders/from_vscode").lazy_load()

-- local check_backspace = function()
--   local col = vim.fn.col "." - 1
--   return col == 0 or vim.fn.getline("."):sub(col, col):match "%s"
-- end

--   פּ ﯟ   some other good icons
-- find more here: https://www.nerdfonts.com/cheat-sheet
local kind_icons = {
  Text = "",
  Method = "m",
  Function = "",
  Constructor = "",
  Field = "",
  Variable = "",
  Class = "",
  Interface = "",
  Module = "",
  Property = "",
  Unit = "",
  Value = "",
  Enum = "",
  Keyword = "",
  Snippet = "",
  Color = "",
  File = "",
  Reference = "",
  Folder = "",
  EnumMember = "",
  Constant = "",
  Struct = "",
  Event = "",
  Operator = "",
  TypeParameter = "",
}

cmp.setup({
  sources = {
    { name = "nvim_lsp" },
    { name = "buffer" },
    { name = "path" },
    { name = "vsnip" },
    { name = "crates" },
  },
  preselect = cmp.PreselectMode.None,
  snippet = {
    expand = function(args)
      --luasnip.lsp_expand(args.body) -- For `luasnip` users.
      vim.fn["vsnip#anonymous"](args.body) -- For `vsnip` users.
    end,
  },
  mapping = {
    ['<CR>'] = cmp.mapping.confirm({
      behavior = cmp.ConfirmBehavior.Replace,
      select = false, -- false = only complete if an item is actually selected
    }),
    -- ['<CR>'] = function(fallback)
    --   -- if cmp.visible() then
    --     cmp.mapping.confirm({
    --       behavior = cmp.ConfirmBehavior.Insert,
    --       select = true,
    --     })
    --   -- else
    --   --     fallback()
    --   -- end
    -- end,
    ['<Tab>'] = function(fallback)
        if cmp.visible() then
            cmp.select_next_item()
        else
            fallback()
        end
    end,
    ['<S-Tab>'] = function(fallback)
        if cmp.visible() then
            cmp.select_prev_item()
        else
            fallback()
        end
    end,
  },
})

-- cmp.setup {
--   sources = {
--     { name = "nvim_lsp" },
--     --{ name = "luasnip" },
--     { name = "buffer" },
--     { name = "path" },
--   },
--
--   formatting = {
--     fields = { "abbr", "kind", "menu" },
--     format = function(entry, vim_item)
--       -- Kind icons
--       --vim_item.kind = string.format("%s", kind_icons[vim_item.kind])
--       vim_item.kind = string.format('%s %s', kind_icons[vim_item.kind], vim_item.kind) -- This concatonates the icons with the name of the item kind
--       vim_item.menu = ({
--         nvim_lsp = "LSP",
--         --luasnip = "Snippet",
--         buffer = "Buffer",
--         path = "Path",
--       })[entry.source.name]
--       return vim_item
--     end,
--   },
--   documentation = {
--     border = { "╭", "─", "╮", "│", "╯", "─", "╰", "│" },
--     -- border = { '', '', '', ' ', '', '', '', ' ' },
--   },
--
--   snippet = {
--     expand = function(args)
--       --luasnip.lsp_expand(args.body) -- For `luasnip` users.
--       vim.fn["vsnip#anonymous"](args.body) -- For `vsnip` users.
--     end,
--   },
--
--
--   sorting = {
--       --priority_weight = 2,
--       comparators = {
--         -- compare.offset,
--         -- compare.exact,
--         compare.score,
--         compare.recently_used,
--         -- compare.kind,
--         -- compare.sort_text,
--         -- compare.length,
--         -- compare.order,
--       },
--     },
--
--
--     mapping = {
--         ['<C-p>'] = cmp.mapping.select_prev_item(),
--         ['<C-n>'] = cmp.mapping.select_next_item(),
--         ['<C-d>'] = cmp.mapping.scroll_docs(-4),
--         ['<C-f>'] = cmp.mapping.scroll_docs(4),
--         ['<C-Space>'] = cmp.mapping.complete(),
--         ['<C-e>'] = cmp.mapping.close(),
--         ['<CR>'] = cmp.mapping({
--           i = cmp.mapping.confirm({ select = true }),
--           c = cmp.mapping.confirm({ select = false }),
--         }),
--         ['<Tab>'] = function(fallback)
--             if cmp.visible() then
--                 cmp.select_next_item()
--             else
--                 fallback()
--             end
--         end,
--         ['<S-Tab>'] = function(fallback)
--             if cmp.visible() then
--                 cmp.select_prev_item()
--             else
--                 fallback()
--             end
--         end,
--     },
--  
--   --
--   -- confirm_opts = {
--   --   behavior = cmp.ConfirmBehavior.Replace,
--   --   select = false,
--   -- },
--
--   -- experimental = {
--   --   ghost_text = false,
--   --   native_menu = false,
--   -- },
-- }
--
-- -- -- Set configuration for specific filetype.
-- -- cmp.setup.filetype('gitcommit', {
-- --   sources = cmp.config.sources({
-- --     { name = 'cmp_git' }, -- You can specify the `cmp_git` source if you were installed it. 
-- --   }, {
-- --     { name = 'buffer' },
-- --   })
-- -- })
-- --
-- -- -- Use buffer source for `/` (if you enabled `native_menu`, this won't work anymore).
-- -- cmp.setup.cmdline('/', {
-- --   sources = {
-- --     { name = 'buffer' }
-- --   }
-- -- })
-- --
-- -- -- Use cmdline & path source for ':' (if you enabled `native_menu`, this won't work anymore).
-- -- cmp.setup.cmdline(':', {
-- --   sources = cmp.config.sources({
-- --     { name = 'path' }
-- --   }, {
-- --     { name = 'cmdline' }
-- --   })
-- -- })
-- --
-- -- Setup lspconfig.
-- --local capabilities = require'cmp'.update_capabilities(vim.lsp.protocol.make_client_capabilities())
-- -- require('lspconfig')['rust_analyzer'].setup {
-- --   capabilities = capabilities
-- -- }
--