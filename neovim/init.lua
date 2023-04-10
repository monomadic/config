--------------------------------------------------------------------------------
--
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
--  TODO:
--  - https://github.com/ldelossa/litee-symboltree.nvim
--  - https://github.com/MunifTanjim/nui.nvim - menus etc
--  - format-on-save
--  - look for small minimal fuzzy replacement
--
--------------------------------------------------------------------------------

require 'utils'

Utils = require 'utils'
Joshuto = require 'joshuto'
TreeSitter = require 'treesitter'

require 'autocmds'

-- ui
require 'text'
require 'statusline'	-- bottom bar (global)
require 'tabline'			-- top bar (global)
require 'winbar'			-- top bar (buffer)

require 'runfile' -- .runfile support
require 'settings'
require 'keymaps'
require 'colors'

require 'lsp'
-- require 'autoclose'.setup({}) -- automatic close for ()[]"" etc

require 'bootstrap-lazy'

require 'ts.ts-jump'

--------------------------------------------------------------------------------
