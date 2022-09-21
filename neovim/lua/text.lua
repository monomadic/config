vim.cmd [[ command -range=% UpperCase '<,'>s/[a-zA-Z]/\U&/g ]]
vim.cmd [[ command -range=% LowerCase '<,'>s/[a-z]/\l&/g ]]
vim.cmd [[ command -range=% CamelCase '<,'>s/[ |-][a-zA-Z]/\U&/g ]]
vim.cmd [[ command -range=% Capitalize '<,'>s/[ |-][a-zA-Z]/\U&/g ]]

-- LOCAL FUNCTION GET_VISUAL_SELECTION()
-- 	-- Yank current visual selection into the 'v' register
-- 	--
-- 	-- Note that this makes no effort to preserve this register
-- 	vim.cmd('noau normal! "vy"')
--
-- 	return vim.fn.getreg('v')
-- end
--
-- local function visual_selection_range()
-- 	local _, csrow, cscol, _ = unpack(vim.fn.getpos "'<")
-- 	local _, cerow, cecol, _ = unpack(vim.fn.getpos "'>")
--
-- 	local start_row, start_col, end_row, end_col
--
-- 	if csrow < cerow or (csrow == cerow and cscol <= cecol) then
-- 		start_row = csrow
-- 		start_col = cscol
-- 		end_row = cerow
-- 		end_col = cecol
-- 	else
-- 		start_row = cerow
-- 		start_col = cecol
-- 		end_row = csrow
-- 		end_col = cscol
-- 	end
--
-- 	return start_row, start_col, end_row, end_col
-- end
--
-- local word_to_title_case = function(str)
-- 	return string.sub(str, 1, 1):upper() .. string.sub(str, 2):lower()
-- end

-- local preserve = function(arguments)
-- 	local arguments = string.format("keepjumps keeppatterns execute %q", arguments)
-- 	-- local original_cursor = vim.fn.winsaveview()
-- 	local line, col = unpack(vim.api.nvim_win_get_cursor(0))
-- 	vim.api.nvim_command(arguments)
-- 	local lastline = vim.fn.line("$")
-- 	-- vim.fn.winrestview(original_cursor)
-- 	if line > lastline then
-- 		line = lastline
-- 	end
-- 	vim.api.nvim_win_set_cursor({ 0 }, { line, col })
-- end

-- camelcase: s/\(_\)\(.\)/\u\2/g

-- local function words_to_title_case(str)
-- 	local words = vim.split(str, ' ')
-- 	return table.concat(utils.map(parts, word_to_title_case), "/")
-- end
--
--vim.api.nvim_create_user_command("TitleCase", [[s/\(_\)\(.\)/\u\2/g]], {})

-- vim.api.nvim_create_user_command("ToTitleCase", function(args)
-- 	print(args.range)
-- 	-- local selected_text = get_visual_selection_range()
-- 	-- local selected_text = get_visual_selection()
-- 	-- print(selected_text)
-- 	--return words_to_title_case(selected_text)
-- end, {})
