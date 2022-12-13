-- UTILS
--

local function file_exists(fname)
	local stat = vim.loop.fs_stat(fname)
	return (stat and stat.type) or false
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
