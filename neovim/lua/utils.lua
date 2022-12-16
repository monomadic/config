-- UTILS
--

local function file_exists(fname)
	local stat = vim.loop.fs_stat(fname)
	return (stat and stat.type) or false
end

function OpenFiles()
	require('telescope.builtin').find_files { path_display = { "truncate" }, prompt_title = "", preview_title = "" }
end

function Format()
	vim.lsp.buf.format { async = true }
end

function ToggleLineNumbers()
	if vim.wo.relativenumber == true then
		vim.wo.relativenumber = false -- turn off line numbers
		vim.wo.number = false
	else
		vim.wo.relativenumber = true -- turn off line numbers
		vim.wo.number = true
	end
end

function Build()
	print("no build command found for this project")
end

-- go to root project file
function GoRoot()
	if file_exists("src/lib.rs") then
		vim.cmd ':edit src/lib.rs'
	elseif file_exists("src/main.rs") then
		vim.cmd ':edit src/main.rs'
	elseif file_exists("index.md") then
		vim.cmd ':edit index.md'
	elseif file_exists("src/index.ts") then
		vim.cmd ':edit src/index.ts'
	elseif file_exists("init.lua") then
		vim.cmd ':edit init.lua'
	else
		print("no root file found.")
	end
end

function GoPackagerFile()
	if file_exists("Cargo.toml") then
		vim.cmd ':edit Cargo.toml'
	elseif file_exists("package.json") then
		vim.cmd ':edit package.json'
	else
		print("no package manifest found.")
	end
end

function LSPDiagnostics(bufnr)
	local count = {}
	local levels = {
		errors = "Error",
		warnings = "Warn",
		info = "Info",
		hints = "Hint",
	}

	for k, level in pairs(levels) do
		count[k] = vim.tbl_count(vim.diagnostic.get(bufnr, { severity = level }))
	end

	local errors = ""
	local warnings = ""
	local hints = ""
	local info = ""

	if count["errors"] ~= 0 then
		errors = "%#LspDiagnosticsSignError# " .. count["errors"] .. " "
	end
	if count["warnings"] ~= 0 then
		warnings = "%#LspDiagnosticsSignWarning# " .. count["warnings"] .. " "
	end
	if count["hints"] ~= 0 then
		hints = "%#LspDiagnosticsSignHint# " .. count["hints"] .. " "
	end
	if count["info"] ~= 0 then
		info = "%#LspDiagnosticsSignInformation# " .. count["info"] .. " "
	end

	return errors .. warnings .. hints .. info .. "%#Normal#"
end
