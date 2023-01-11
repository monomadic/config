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
--
--------------------------------------------------------------------------------

require 'utils'
require 'autocmds'

require 'text'
require 'statusline'
require 'tabline'
require 'winbar'

require 'settings'
require 'keymaps'
require 'colors'

require 'lsp'
-- require 'autoclose'.setup({}) -- automatic close for ()[]"" etc

require 'bootstrap-lazy'

--------------------------------------------------------------------------------
