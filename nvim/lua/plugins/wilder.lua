local wilder = require("wilder")

wilder.setup({
  modes = { ":", "/", "?" },
})

wilder.set_option(
  "renderer",
  wilder.popupmenu_renderer({
    pumblend = 20,
    highlighter = wilder.basic_highlighter(),
    left = {
      wilder.popupmenu_devicons(),
    },
    border = "rounded",
  })
)
