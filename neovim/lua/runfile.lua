function _G.RunFile()
  -- Load commands from a text file in the project root directory
  local function load_commands_from_file(filename)
    local commands = {}
    local file = io.open(filename, "r")
    if file then
      for line in file:lines() do
        table.insert(commands, line)
      end
      file:close()
    else
      return nil
    end
    return commands
  end

  local function open_floating_terminal_with_command(command)
    local width = vim.api.nvim_get_option("columns")
    local height = vim.api.nvim_get_option("lines")

    local buf = vim.api.nvim_create_buf(false, true)
    vim.api.nvim_buf_set_option(buf, "bufhidden", "wipe")

    local win = vim.api.nvim_open_win(buf, true, {
      relative = "editor",
      width = math.ceil(width * 0.8),
      height = math.ceil(height * 0.8),
      col = math.ceil(width * 0.1),
      row = math.ceil(height * 0.1),
      style = "minimal",
      border = "single"
    })

    vim.api.nvim_win_set_option(win, "winhl", "Normal:NormalFloat")
    vim.cmd("terminal " .. command)
  end

  local commands_file_path = vim.loop.cwd() .. "/.runfile"
  local commands = load_commands_from_file(commands_file_path)

  if commands then
    -- Add numbers before each command
    local numbered_commands = {}
    for i, command in ipairs(commands) do
      table.insert(numbered_commands, string.format("%d. %s", i, command))
    end

    -- Display the commands using vim.ui.select and store the user's choice
    vim.ui.select(numbered_commands, {
      prompt = "RunFile:",
      default = 1
    }, function(selected_item)
      -- Find the index of the selected item in the numbered_commands table
      local choice = nil
      for i, item in ipairs(numbered_commands) do
        if item == selected_item then
          choice = i
          break
        end
      end

      -- Check if the user made a valid choice and execute the corresponding command
      if choice and choice >= 1 and choice <= #commands then
        open_floating_terminal_with_command(commands[choice])
      end
    end)
  else
    print("Error: Could not open the file " .. commands_file_path)
  end
end
