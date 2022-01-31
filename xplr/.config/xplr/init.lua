version = "0.17.0"

local home = os.getenv("HOME")

xplr.config.general.show_hidden = false
xplr.config.general.prompt.format = "❯ "
xplr.config.general.panel_ui.default.borders = {}
xplr.config.general.panel_ui.help_menu.borders = { "Top", "Bottom", "Left", "Right", }

package.path = home
.. "/.config/xplr/plugins/?/init.lua;"
.. home
.. "/.config/xplr/plugins/?.lua;"
.. home
.. '/.config/xplr/plugins/?/src/init.lua;'
.. package.path

require("zentable").setup()
require("icons").setup()

require("dragon").setup{
  mode = "selection_ops",
  key = "D",
  drag_args = "",
  drop_args = "",
  keep_selection = false,
}

