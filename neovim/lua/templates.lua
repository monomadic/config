local function get_project_directory()
    local clients = vim.lsp.buf_get_clients()

    for _, client in pairs(clients) do
        local root_dir = client.config.root_dir
        if root_dir then
            return root_dir
        end
    end

    -- Fall back to the current working directory if no LSP root directory is found
    return vim.fn.getcwd()
end

local function get_files_recursive(directory)
    local files = {}
		local p = io.popen('fd --type symlink . "' .. directory .. '"')
		if p then
			for entry in p:lines() do
					local relative_path = entry
					table.insert(files, relative_path)
			end
		end
    return files
end

local function insert_template(template_path)
    local file = io.open(template_path, "r")
    if file then
        local lines = {}
        for line in file:lines() do
            table.insert(lines, line)
        end
        file:close()

        local bufnr = vim.api.nvim_get_current_buf()
        local cursor = vim.api.nvim_win_get_cursor(0)
        vim.api.nvim_buf_set_lines(bufnr, cursor[1]-1, cursor[1]-1, false, lines)
    else
        print("Could not open template file: " .. template_path)
    end
end

local function create_template_menu(templates_dir)
    local project_dir = get_project_directory()
    local files = get_files_recursive(templates_dir)

    local choices = {}
    for _, file in ipairs(files) do
        table.insert(choices, file)
    end

    vim.ui.select(choices, {
        prompt = 'Select a template:',
        format_item = function(item)
            return item
        end
    }, function(id, item)
        if id then
            local relative_path = files[id]
            local template_path = templates_dir .. "/" .. relative_path
            local filename = vim.fn.fnamemodify(relative_path, ':t')
            local new_file_path = project_dir .. "/" .. filename
            print("Copying from: " .. template_path .. " to " .. new_file_path)
            os.execute('cp "' .. template_path .. '" "' .. new_file_path .. '"')
            vim.cmd('edit ' .. new_file_path)
        end
    end)
end

local function buf_insert_template()
	create_template_menu("~/.config/nvim/templates")
end

_G.create_template_menu = create_template_menu
_G.buf_insert_template = buf_insert_template
