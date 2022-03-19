require("nvim-web-devicons").setup({
  override = {
    zsh = { icon = "", color = "#428850", cterm_color = "65", name = "Zsh" },
    sh = { icon = "", color = "#00FF00", cterm_color = "65", name = "Zsh" },
    sol = {
      icon = "ﲹ",
      color = "#5064FF",
      cterm_color = "140",
      name = "Solidity",
    },
    lock = {
      icon = "",
      color = "#43443F",
      cterm_color = "140",
      name = "Lock",
    },
    md = {
      icon = "",
      color = "#44d4fF",
      cterm_color = "140",
      name = "Markdown",
    },
    toml = {
      icon = "",
      color = "#FAC",
      cterm_color = "140",
      name = "TOML",
    },
  },
  default = true,
})
