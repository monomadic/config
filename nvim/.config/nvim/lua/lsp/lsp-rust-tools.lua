local opts = {
  tools = { -- rust-tools options
    -- Automatically set inlay hints (type hints)
    autoSetHints = true,

    -- Whether to show hover actions inside the hover window
    -- This overrides the default hover handler
    hover_with_actions = true,

    -- how to execute terminal commands
    -- options right now: termopen / quickfix
    executor = require("rust-tools/executors").termopen,

    runnables = {
      use_telescope = true,
    },

    debuggables = {
      use_telescope = true,
    },

    -- These apply to the default RustSetInlayHints command
    inlay_hints = {

      -- Only show inlay hints for the current line
      only_current_line = false,

      -- Event which triggers a refersh of the inlay hints.
      -- You can make this "CursorMoved" or "CursorMoved,CursorMovedI" but
      -- not that this may cause  higher CPU usage.
      -- This option is only respected when only_current_line and
      -- autoSetHints both are true.
      only_current_line_autocmd = "CursorHold",

      -- wheter to show parameter hints with the inlay hints or not
      show_parameter_hints = true,

      -- whether to show variable name before type hints with the inlay hints or not
      show_variable_name = false,

      -- prefix for parameter hints
      parameter_hints_prefix = "<- ",

      -- prefix for all the other hints (type, chaining)
      other_hints_prefix = "=> ",

      -- whether to align to the length of the longest line in the file
      max_len_align = false,

      -- padding from the left if max_len_align is true
      max_len_align_padding = 1,

      -- whether to align to the extreme right or not
      right_align = false,

      -- padding from the right if right_align is true
      right_align_padding = 7,

      -- The color of the hints
      highlight = "Comment",
    },

    hover_actions = {
      -- the border that is used for the hover window
      -- see vim.api.nvim_open_win()
      border = {
        { "╭", "FloatBorder" },
        { "─", "FloatBorder" },
        { "╮", "FloatBorder" },
        { "│", "FloatBorder" },
        { "╯", "FloatBorder" },
        { "─", "FloatBorder" },
        { "╰", "FloatBorder" },
        { "│", "FloatBorder" },
      },

      -- whether the hover action window gets automatically focused
      auto_focus = true,
    },
  },

  -- all the opts to send to nvim-lspconfig
  -- these override the defaults set by rust-tools.nvim
  -- see https://github.com/neovim/nvim-lspconfig/blob/master/doc/server_configurations.md#rust_analyzer
  server = {
    -- standalone file support
    -- setting it to false may improve startup time
    standalone = false,
    on_attach = function()
      require("lsp.keymaps")()

      local map = vim.api.nvim_set_keymap
      map("n", "gn", ":RustRunnables<cr>", { silent = true })
    end,

  capabilites = function()
    local capabilities = require'cmp_nvim_lsp'.update_capabilities(vim.lsp.protocol.make_client_capabilities())
    --local capabilities = vim.lsp.protocol.make_client_capabilities()
    capabilities.textDocument.completion.completionItem.snippetSupport = false
    capabilities.textDocument.completion.completionItem.resolveSupport = {
      properties = {
        'documentation',
        'detail',
        'additionalTextEdits',
      }
    }
    --capabilities = vim.tbl_extend('keep', capabilities or {}, lsp_status.capabilities)

    capabilities.experimental = {}
    capabilities.experimental.hoverActions = true

    return capabilities
  end,


    settings = {
      -- to enable rust-analyzer settings visit:
      -- https://github.com/rust-analyzer/rust-analyzer/blob/master/docs/user/generated_config.adoc
      ["rust-analyzer"] = {
        -- enable clippy on save
        checkOnSave = {
          command = "clippy",
        },
      },
    },
  }, -- rust-analyer options

  -- debugging stuff
  dap = {
    adapter = {
      type = "executable",
      command = "lldb-vscode",
      name = "rt_lldb",
    },
  },
}

-- require'lspconfig'.rust_analyzer.setup {
--   on_attach = function()
--     print("attached rust_analyzer lsp")
--     require'lsp.keymaps'()
--
--     local map = vim.api.nvim_set_keymap
--     map('n', 'gn', ':RustRunnables<cr>', { silent = true })
--     --require('rust-tools').setup({})
--   end,
--   capabilites = function()
--     local capabilities = require'cmp_nvim_lsp'.update_capabilities(vim.lsp.protocol.make_client_capabilities())
--     --local capabilities = vim.lsp.protocol.make_client_capabilities()
--     capabilities.textDocument.completion.completionItem.snippetSupport = false
--     capabilities.textDocument.completion.completionItem.resolveSupport = {
--       properties = {
--         'documentation',
--         'detail',
--         'additionalTextEdits',
--       }
--     }
--     --capabilities = vim.tbl_extend('keep', capabilities or {}, lsp_status.capabilities)
--
--     capabilities.experimental = {}
--     capabilities.experimental.hoverActions = true
--
--     return capabilities
--   end,
--   settings = {
--     ["rust-analyzer"] = {
--       assist = {
--         importMergeBehavior = "last",
--         importPrefix = "by_self",
--       },
--       diagnostics = {
--         disabled = { "unresolved-import" }
--       },
--       cargo = {
--         loadOutDirsFromCheck = true
--       },
--       procMacro = {
--         enable = true
--       },
--       checkOnSave = {
--         command = "clippy"
--       },
--     }
--   },
-- }
--
require("rust-tools").setup(opts)