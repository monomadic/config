-- support for the comrak cli

-- current file to comrak
function comrak_file_to_html()
	local current_file = vim.fn.expand('%:p')
	local comrak_cmd = "comrak " .. current_file

	if vim.fn.executable('comrak') == 0 then
		print("comrak is not installed")
		return
	end

	local job_id = vim.fn.jobstart(
		comrak_cmd,
		{
			on_stdout = function(_, data, _)
				if data[1] ~= nil then
					local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
					vim.api.nvim_buf_set_lines(0, 0, #lines, false, data)
				end
			end,
			stdout_buffered = true,
		}
	)

	vim.fn.jobwait({ job_id })
end

-- buffer to stdin
function comrak_to_html()
	local input = Utils.get_current_buffer_content()
	local output = vim.fn.system("comrak", input)
	Utils.set_current_buffer_content(output)
end

function comrak_to_md()
	local current_file = vim.fn.expand('%:p')
	local comrak_cmd = "comrak --to commonmark " .. current_file

	if vim.fn.executable('comrak') == 0 then
		print("comrak is not installed")
		return
	end

	local job_id = vim.fn.jobstart(
		comrak_cmd,
		{
			on_stdout = function(_, data, _)
				if data[1] ~= nil then
					local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
					vim.api.nvim_buf_set_lines(0, 0, #lines, false, data)
				end
			end,
			stdout_buffered = true,
		}
	)

	vim.fn.jobwait({ job_id })
end
