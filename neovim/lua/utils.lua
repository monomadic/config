-- UTILS
--

local function file_exists(fname)
	local stat = vim.loop.fs_stat(fname)
	return (stat and stat.type) or false
end

function OpenFiles()
	require('telescope.builtin').find_files { path_display = { "truncate" }, prompt_title = "", preview_title = "" }
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
