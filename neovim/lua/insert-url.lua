-- prompt for a markdown url and fetch its title, inserting it as a markdown link

local function prompt_url()
  local input_url = vim.fn.input('Enter URL: ')
  return input_url
end

local function fetch_title(url)
  local cmd = "curl -s " .. url .. " | awk -v IGNORECASE=1 -v RS='</title>' '/<title>/{gsub(/.*<title>|<[^>]*>/,\"\");print;exit}'"
  local handle = io.popen(cmd)
	if handle then
		local title = handle:read("*a")
		handle:close()
		return title:gsub("%s+", " "):gsub("^%s*(.-)%s*$", "%1")
	else
		print("Error: fetch title failed")
	end
end

local function create_markdown_link(url, title)
  return string.format("[%s](%s)", title, url)
end

local function insert_markdown_link(link)
  local current_line = vim.api.nvim_win_get_cursor(0)[1]
  vim.api.nvim_buf_set_lines(0, current_line, current_line, false, {link})
end

function InsertMarkdownLink()
  local url = prompt_url()
  local title = fetch_title(url)
  if not title or title == "" then
    print("Failed to find the title.")
    return
  end

  local markdown_link = create_markdown_link(url, title)
  insert_markdown_link(markdown_link)
end
