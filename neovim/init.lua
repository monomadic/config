--------------------------------------------------------------------------------
--  MONOMADIC NEOVIM CONFIG
--
--------------------------------------------------------------------------------
--
--		- templates:
--		  https://github.com/chr0n1x/neovim-template
--
--		- plugin-index:
--		  https://neoland.dev/
--
--		- references:
--			https://github.com/NormalNvim/NormalNvim
--
--  TODO:
--  - https://github.com/ldelossa/litee-symboltree.nvim
--  - https://github.com/MunifTanjim/nui.nvim - menus etc
--  - format-on-save
--  - https://github.com/mfussenegger/nvim-fzy
--
--------------------------------------------------------------------------------

-- lua byte-compiler
vim.loader.enable()

require 'utils'

Utils = require 'utils'
Joshuto = require 'joshuto'
TreeSitter = require 'treesitter'

require 'autocmds'

-- ui
require 'text'
require 'statusline' -- bottom bar (global)
require 'tabline'    -- top bar (global)
require 'winbar'     -- top bar (buffer)
require 'gutter'
require 'menu'       -- right-click menu

require 'runfile'    -- .runfile support
require 'settings'
require 'keymaps'
require 'colors'
require 'insert-url' -- insert markdown url
require 'git'

require 'lsp'
-- require 'autoclose'.setup({}) -- automatic close for ()[]"" etc

require 'bootstrap-lazy'

-- treesitter
require 'ts.ts-jump'
require 'ts.highlight-matches'

require 'templates'

LoadRunFileKeymaps()

--------------------------------------------------------------------------------
