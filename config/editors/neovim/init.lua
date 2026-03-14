--------------------------------------------------------------------------------
--
--  MONOMADIC NEOVIM CONFIG
--
--------------------------------------------------------------------------------

-- lua byte-compiler
vim.loader.enable()

require 'debug'
require 'colors'
require 'utils'

Utils = require 'utils'
Joshuto = require 'joshuto'
-- TreeSitter = require 'treesitter'

-- ui
require 'text'       -- text manipulation
require 'statusline' -- bottom bar (global)
require 'tabline'    -- top bar (global)
require 'winbar'     -- top bar (buffer)
require 'gutter'     -- git gutter
require 'menu'       -- right-click menu

require 'runfile'    -- .runfile support
require 'settings'
require 'keymaps'
require 'insert-url' -- insert markdown url
require 'git'

require 'bootstrap-lazy'

require 'lsp'
-- require 'autoclose'.setup({}) -- automatic close for ()[]"" etc

require 'comrak'

-- treesitter
require 'ts.ts-jump'
require 'ts.highlight-matches'

require 'autocmds'
require 'templates'

LoadRunFileKeymaps()

--------------------------------------------------------------------------------
