local gl = require("galaxyline")
local diagnostic = require("galaxyline.provider_diagnostic")
--local colors = require("galaxyline.theme").default
local condition = require("galaxyline.condition")
local gls = gl.section
gl.short_line_list = { "neo-tree", "vista", "dbui", "packer" }

local colors = {
  bg = "#282c34",
  yellow = "#fabd2f",
  cyan = "#008080",
  darkblue = "#081633",
  green = "#afFF00",
  orange = "#FF8800",
  purple = "#9d2FFa",
  magenta = "#31Fd9e",
  grey = "#c0c0c0",
  blue = "#008AFF",
  red = "#Fc4f97",
}

-- gls.short_line_left[0] = {
--   ProjectDir = {
--     provider = function()
--       return "   " .. vim.fn.fnamemodify(vim.fn.getcwd(), ":t") .. " "
--     end,
--     separator = " ",
--     separator_highlight = { "NONE", colors.bg },
--     highlight = { colors.green, colors.green, "bold" },
--   },
-- }
--
-- gls.short_line_left[1] = {
--   FileIcon = {
--     provider = "FileIcon",
--     condition = condition.buffer_not_empty,
--     highlight = { require("galaxyline.provider_fileinfo").get_file_icon_color, colors.bg },
--   },
-- }
--
-- gls.short_line_left[2] = {
--   FileName = {
--     provider = function()
--       return vim.fn.fnamemodify(vim.fn.expand("%"), ":~:.") .. " "
--     end,
--     condition = condition.buffer_not_empty,
--     highlight = { colors.fg, colors.bg },
--   },
-- }

gls.left[1] = {
  ViMode = {
    provider = function()
      -- auto change color according the vim mode
      local mode_color = {
        n = colors.red,
        i = colors.green,
        v = colors.blue,
        [""] = colors.blue,
        V = colors.blue,
        c = colors.magenta,
        no = colors.red,
        s = colors.orange,
        S = colors.orange,
        [""] = colors.orange,
        ic = colors.yellow,
        R = colors.violet,
        Rv = colors.violet,
        cv = colors.red,
        ce = colors.red,
        r = colors.cyan,
        rm = colors.cyan,
        ["r?"] = colors.cyan,
        ["!"] = colors.red,
        t = colors.red,
      }
      if mode_color[vim.fn.mode()] ~= nil then
        vim.api.nvim_command("hi GalaxyViMode guibg=" .. mode_color[vim.fn.mode()])
      end
      return "  " .. vim.fn.mode() .. " "
    end,
    highlight = { colors.bg, colors.bg, "bold" },
  },
}

-- PWD
gls.left[3] = {
  ProjectDir = {
    provider = function()
      return "   " .. vim.fn.fnamemodify(vim.fn.getcwd(), ":t") .. " "
    end,
    separator = " ",
    separator_highlight = { "NONE", colors.bg },
    highlight = { colors.bg, colors.magenta, "bold" },
  },
}

-- gls.left[2] = {
--   FileSize = {
--     provider = "FileSize",
--     condition = condition.buffer_not_empty,
--     highlight = { colors.fg, colors.bg },
--   },
-- }

-- icon
gls.left[4] = {
  FileIcon = {
    provider = "FileIcon",
    condition = condition.buffer_not_empty,
    highlight = { require("galaxyline.provider_fileinfo").get_file_icon_color, colors.bg },
  },
}

-- filename
gls.left[5] = {
  FileName = {
    provider = function()
      local file = vim.fn.fnamemodify(vim.fn.expand("%"), ":~:.") .. " "
      if vim.bo.modifiable then
        return file .. ""
      else
        return file
      end
    end,
    condition = condition.buffer_not_empty,
    highlight = { colors.fg, colors.bg },
  },
}

gls.left[6] = {
  BufferIcon = {
    provider = "BufferIcon",
    highlight = { colors.fg, colors.bg },
  },
}

-- gls.left[6] = {
--   LineInfo = {
--     provider = "LineColumn",
--     separator = " ",
--     separator_highlight = { "NONE", colors.bg },
--     highlight = { colors.fg, colors.bg },
--   },
-- }

-- gls.left[7] = {
--   PerCent = {
--     provider = "LinePercent",
--     separator = " ",
--     separator_highlight = { "NONE", colors.bg },
--     highlight = { colors.fg, colors.bg, "bold" },
--   },
-- }

gls.left[8] = {
  DiagnosticError,
}

-- gls.left[9] = {
--   DiagnosticWarn = {
--     provider = 'DiagnosticWarn',
--     icon = '  ',
--     highlight = {colors.yellow,colors.bg},
--   }
-- }
--
-- gls.left[10] = {
--   DiagnosticHint = {
--     provider = 'DiagnosticHint',
--     icon = '  ',
--     highlight = {colors.cyan,colors.bg},
--   }
-- }
--
-- gls.left[11] = {
--   DiagnosticInfo = {
--     provider = 'DiagnosticInfo',
--     icon = '  ',
--     highlight = {colors.blue,colors.bg},
--   }
-- }

gls.right[1] = {
  ShowLspClient = {
    provider = function(msg)
      msg = msg or ""
      local buf_ft = vim.api.nvim_buf_get_option(0, "filetype")
      local clients = vim.lsp.get_active_clients()
      if next(clients) == nil then
        return msg
      end
      for _, client in ipairs(clients) do
        local filetypes = client.config.filetypes
        if filetypes and vim.fn.index(filetypes, buf_ft) ~= -1 then
          return client.name
        end
      end
      return msg
    end,
    condition = function()
      local tbl = { ["dashboard"] = true, [""] = true }
      if tbl[vim.bo.filetype] then
        return false
      end
      return true
    end,
    highlight = { require("galaxyline.provider_fileinfo").get_file_icon_color, colors.bg, "bold" },
    separator = " ",
    separator_highlight = { "NONE", colors.bg },
  },
}

-- gls.right[9] = {
--   FileEncode = {
--     provider = 'FileEncode',
--     condition = condition.hide_in_width,
--     separator = ' ',
--     separator_highlight = {'NONE',colors.bg},
--     highlight = {colors.green,colors.bg,'bold'}
--   }
-- }

-- gls.right[2] = {
--   FileFormat = {
--     provider = 'FileFormat',
--     condition = condition.hide_in_width,
--     separator = ' ',
--     separator_highlight = {'NONE',colors.bg},
--     highlight = {colors.green,colors.bg,'bold'}
--   }
-- }

gls.right[3] = {
  GitIcon = {
    provider = function()
      return "  "
    end,
    condition = condition.check_git_workspace and condition.hide_in_width,
    highlight = { colors.violet, colors.bg, "bold" },
    separator = " ",
    separator_highlight = { "NONE", colors.bg },
  },
}

gls.right[4] = {
  GitBranch = {
    provider = "GitBranch",
    condition = condition.check_git_workspace and condition.hide_in_width,
    highlight = { colors.violet, colors.bg, "bold" },
  },
}

gls.right[5] = {
  RainbowBlue = {
    provider = function()
      return "   "
    end,
    highlight = { colors.blue, colors.bg },
  },
}

gls.right[6] = {
  DiffAdd = {
    provider = "DiffAdd",
    condition = condition.hide_in_width,
    icon = " ",
    highlight = { colors.green, colors.bg },
  },
}
gls.right[7] = {
  DiffModified = {
    provider = "DiffModified",
    condition = condition.hide_in_width,
    icon = " ",
    highlight = { colors.orange, colors.bg },
  },
}

gls.right[8] = {
  DiffRemove = {
    provider = "DiffRemove",
    condition = condition.hide_in_width,
    icon = " ",
    highlight = { colors.red, colors.bg },
  },
}

gls.short_line_left[1] = {
  BufferType = {
    --provider = 'FileTypeName',
    provider = function()
      return "  "
    end,
    separator = " ",
    separator_highlight = { "NONE", "#111111" },
    highlight = { colors.blue, "#111111", "bold" },
  },
}

gls.short_line_left[2] = {
  SFileName = {
    provider = "SFileName",
    condition = condition.buffer_not_empty,
    highlight = { colors.fg, "#111111", "bold" },
  },
}

gls.short_line_right[3] = {
  BufferIcon = {
    provider = "BufferIcon",
    highlight = { colors.fg, "#111111" },
  },
}