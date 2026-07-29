-- Preview macOS .icns icon files.
--
-- mime-ext has no mapping for .icns, so these arrive as application/octet-stream
-- and would otherwise fall through to the `code` previewer as binary garbage.
-- Convert the largest representation to a cached PNG with sips(1) — always
-- present on macOS, no Homebrew dependency — and render that.

local M = {}

local COLOR_FILENAME = "#66e2ff"
local COLOR_SPECS = "#39ff14"
local TEXT_ROWS = 2

local function png_size(cache)
	local output = Command("sips")
		:arg({ "-g", "pixelWidth", "-g", "pixelHeight", tostring(cache) })
		:stdout(Command.PIPED)
		:output()

	if not output or not output.status or not output.status.success then
		return nil, nil
	end

	local w = tostring(output.stdout):match("pixelWidth:%s*(%d+)")
	local h = tostring(output.stdout):match("pixelHeight:%s*(%d+)")
	return tonumber(w), tonumber(h)
end

local function format_size(bytes)
	if not bytes or bytes <= 0 then
		return nil
	end

	local units = { "B", "KB", "MB", "GB" }
	local value, i = bytes, 1
	while value >= 1024 and i < #units do
		value, i = value / 1024, i + 1
	end

	if i == 1 then
		return string.format("%d %s", value, units[i])
	end
	return (string.format("%.1f", value):gsub("%.0$", "")) .. " " .. units[i]
end

local function split_preview(area)
	local image_h = math.max(1, area.h - TEXT_ROWS)
	return ui.Rect({
		x = area.x,
		y = area.y,
		w = area.w,
		h = image_h,
	}), ui.Rect({
		x = area.x,
		y = area.y + image_h,
		w = area.w,
		h = area.h - image_h,
	})
end

local function info_widget(job, area, cache)
	local specs = {}
	local w, h = png_size(cache)
	if w and h then
		specs[#specs + 1] = string.format("%dx%d", w, h)
	end

	local cha = fs.cha(job.file.url)
	local size = format_size(cha and cha.len)
	if size then
		specs[#specs + 1] = size
	end
	specs[#specs + 1] = "ICNS"

	local widgets = {}
	local y = area.y
	if area.h > 0 then
		widgets[#widgets + 1] = ui.Text(job.file.name or tostring(job.file.url))
			:area(ui.Rect({ x = area.x, y = y, w = area.w, h = 1 }))
			:fg(COLOR_FILENAME)
		y = y + 1
	end

	if y < area.y + area.h then
		widgets[#widgets + 1] = ui.Text(table.concat(specs, "  "))
			:area(ui.Rect({ x = area.x, y = y, w = area.w, h = 1 }))
			:fg(COLOR_SPECS)
	end

	return widgets
end

function M:preload(job)
	local cache = ya.file_cache(job)
	if not cache then
		return false
	end
	if fs.cha(cache) then
		return true
	end

	if cache.parent then
		fs.create("dir_all", cache.parent)
	end

	local status = Command("sips")
		:arg({
			"-s",
			"format",
			"png",
			tostring(job.file.url),
			"--out",
			tostring(cache),
		})
		:status()

	return status and status.success or false
end

function M:peek(job)
	local start, cache = os.clock(), ya.file_cache(job)
	if not cache or not self:preload(job) then
		return 1
	end

	local image_area, text_area = split_preview(job.area)
	ya.sleep(math.max(0, rt.preview.image_delay / 1000 + start - os.clock()))
	ya.image_show(cache, image_area)
	ya.preview_widget(job, info_widget(job, text_area, cache))
end

function M:seek() end

return M
