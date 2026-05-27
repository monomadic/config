local M = {}

local COLOR_FILENAME = "#66e2ff"
local COLOR_SPECS = "#39ff14"
local COLOR_TAG = "#ffe66d"
local MAX_THUMB_ROWS = 25
local MIN_TEXT_ROWS = 5

local TAG_ORDER = {
	"title",
	"artist",
	"album_artist",
	"album",
	"date",
	"creation_time",
	"genre",
	"comment",
	"description",
	"encoder",
	"major_brand",
	"compatible_brands",
}

local function clean(value)
	if not value or value == "" then
		return nil
	end

	value = tostring(value):gsub("[%c\r\n]", " "):gsub("^%s+", ""):gsub("%s+$", "")
	if #value > 140 then
		value = value:sub(1, 137) .. "..."
	end
	return value ~= "" and value or nil
end

local function parse_fraction(value)
	local numerator, denominator = tostring(value or ""):match("^(%-?%d+)/(%-?%d+)$")
	if not numerator or not denominator then
		return tonumber(value)
	end

	numerator, denominator = tonumber(numerator), tonumber(denominator)
	if not numerator or not denominator or denominator == 0 then
		return nil
	end
	return numerator / denominator
end

local function format_fps(value)
	local fps = parse_fraction(value)
	if not fps or fps <= 0 then
		return nil
	end

	local formatted = string.format("%.2f", fps):gsub("%.?0+$", "")
	return formatted .. " fps"
end

local function format_duration(value)
	local seconds = tonumber(value)
	if not seconds or seconds <= 0 then
		return nil
	end

	local total = math.floor(seconds + 0.5)
	local h = math.floor(total / 3600)
	local m = math.floor((total % 3600) / 60)
	local s = total % 60
	if h > 0 then
		return string.format("%d:%02d:%02d", h, m, s)
	end
	return string.format("%d:%02d", m, s)
end

local function format_bitrate(value)
	local bitrate = tonumber(value)
	if not bitrate or bitrate <= 0 then
		return nil
	end

	local kbps = bitrate / 1000
	if kbps >= 1000 then
		return string.format("%.1f Mb/s", kbps / 1000)
	end
	return string.format("%d kb/s", math.floor(kbps + 0.5))
end

local function wrap_text(text, width)
	width = math.max(1, width or 1)
	local lines = {}
	text = tostring(text or "")

	while #text > width do
		local cut = width
		for i = width, 1, -1 do
			local c = text:sub(i, i)
			if c == " " or c == "-" or c == "_" or c == "." then
				cut = i
				break
			end
		end

		lines[#lines + 1] = text:sub(1, cut)
		text = text:sub(cut + 1):gsub("^%s+", "")
	end

	if text ~= "" then
		lines[#lines + 1] = text
	end
	return lines
end

local function parse_video_info(stdout)
	local info = { tags = {} }
	for line in tostring(stdout or ""):gmatch("[^\r\n]+") do
		local key, value = line:match("^([^=]+)=(.*)$")
		if key and value then
			if key:sub(1, 4) == "TAG:" then
				info.tags[key:sub(5)] = clean(value)
			else
				info[key] = clean(value)
			end
		end
	end

	info.width = tonumber(info.width)
	info.height = tonumber(info.height)
	return info
end

local function video_info(job)
	local output = Command("ffprobe")
		:arg({
			"-v",
			"error",
			"-select_streams",
			"v:0",
			"-show_entries",
			"stream=width,height,avg_frame_rate,r_frame_rate,codec_name:format=duration,bit_rate:format_tags",
			"-of",
			"default=noprint_wrappers=1",
			tostring(job.file.url),
		})
		:stdout(Command.PIPED)
		:output()

	if not output or not output.status or not output.status.success then
		return { tags = {} }
	end
	return parse_video_info(output.stdout)
end

local function seek_time(job)
	local skip = math.max(0, math.min(100, tonumber(job.skip) or 0))
	if skip <= 0 then
		return 0
	end

	local output = Command("ffprobe")
		:arg({
			"-v",
			"error",
			"-show_entries",
			"format=duration",
			"-of",
			"default=noprint_wrappers=1:nokey=1",
			tostring(job.file.url),
		})
		:stdout(Command.PIPED)
		:output()

	local duration = output and output.status and output.status.success and tonumber(output.stdout) or nil
	if not duration or duration <= 0 then
		return 0
	end

	return duration * skip / 100
end

local function cache_marker(cache)
	return tostring(cache) .. ".video-thumb-info-first-frame"
end

local function has_marker(cache)
	local status = Command("/usr/bin/test")
		:arg({ "-f", cache_marker(cache) })
		:status()

	return status and status.success
end

local function touch_marker(cache)
	Command("/usr/bin/touch")
		:arg({ cache_marker(cache) })
		:status()
end

local function remove_marker(cache)
	Command("/bin/rm")
		:arg({ "-f", cache_marker(cache) })
		:status()
end

local function write_thumbnail_with_ffmpeg(job, cache, at)
	local status = Command("ffmpeg")
		:arg({
			"-hide_banner",
			"-loglevel",
			"error",
			"-y",
			"-ss",
			string.format("%.3f", at),
			"-i",
			tostring(job.file.url),
			"-map",
			"0:v:0",
			"-frames:v",
			"1",
			"-vf",
			"format=rgba",
			"-f",
			"image2",
			"-vcodec",
			"png",
			tostring(cache),
		})
		:status()

	return status and status.success
end

local function write_thumbnail_with_ffmpegthumbnailer(job, cache, at)
	local status = Command("ffmpegthumbnailer")
		:arg({
			"-i",
			tostring(job.file.url),
			"-o",
			tostring(cache),
			"-s",
			"0",
			"-t",
			string.format("%.3f", at),
			"-q",
			"10",
		})
		:status()

	return status and status.success
end

local function split_preview(area, info)
	local image_h = math.min(MAX_THUMB_ROWS, math.max(1, area.h - MIN_TEXT_ROWS))
	if info.width and info.height and info.width > 0 and info.height > 0 then
		local max_rows = MAX_THUMB_ROWS
		if info.width > info.height then
			max_rows = math.floor(MAX_THUMB_ROWS / 2)
		end
		image_h = math.min(max_rows, math.max(1, area.h - MIN_TEXT_ROWS))
	end

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

local function tag_lines(tags)
	local lines, seen = {}, {}
	for _, key in ipairs(TAG_ORDER) do
		local value = tags[key]
		if value then
			lines[#lines + 1] = { key:gsub("_", " "), value }
			seen[key] = true
		end
		if #lines >= 5 then
			return lines
		end
	end

	for key, value in pairs(tags) do
		if not seen[key] and value then
			lines[#lines + 1] = { key:gsub("_", " "), value }
		end
		if #lines >= 5 then
			break
		end
	end
	return lines
end

local function info_widget(job, area, info)
	local specs = {}
	if info.width and info.height then
		specs[#specs + 1] = string.format("%dx%d", info.width, info.height)
	end
	specs[#specs + 1] = format_fps(info.avg_frame_rate) or format_fps(info.r_frame_rate)
	if info.codec_name then
		specs[#specs + 1] = info.codec_name:upper()
	end
	specs[#specs + 1] = format_duration(info.duration)
	specs[#specs + 1] = format_bitrate(info.bit_rate)

	local compact_specs = {}
	for _, value in ipairs(specs) do
		if value then
			compact_specs[#compact_specs + 1] = value
		end
	end

	local widgets = {}
	local name_lines = wrap_text(job.file.name or tostring(job.file.url), area.w)
	local name_h = math.min(#name_lines, area.h)
	local y = area.y
	if name_h > 0 then
		widgets[#widgets + 1] = ui.Text(table.concat(name_lines, "\n"))
			:area(ui.Rect({ x = area.x, y = y, w = area.w, h = name_h }))
			:fg(COLOR_FILENAME)
		y = y + name_h
	end

	if #compact_specs > 0 and y < area.y + area.h then
		widgets[#widgets + 1] = ui.Text(table.concat(compact_specs, "  "))
			:area(ui.Rect({ x = area.x, y = y, w = area.w, h = 1 }))
			:fg(COLOR_SPECS)
		y = y + 2
	end

	local tags = tag_lines(info.tags or {})
	if #tags > 0 and y < area.y + area.h then
		local label_width = 0
		local rows = {}
		for _, tag in ipairs(tags) do
			label_width = math.max(label_width, #tag[1] + 1)
			rows[#rows + 1] = ui.Row({
				ui.Line(tag[1] .. ":"):fg(COLOR_TAG),
				ui.Text(tag[2]),
			})
		end

		widgets[#widgets + 1] = ui.Table(rows)
			:area(ui.Rect({ x = area.x, y = y, w = area.w, h = area.y + area.h - y }))
			:widths({
				ui.Constraint.Length(label_width + 1),
				ui.Constraint.Fill(1),
			})
	end

	return widgets
end

function M:preload(job)
	local cache = ya.file_cache(job)
	if not cache then
		return false
	end

	local at = seek_time(job)
	if at == 0 and fs.cha(cache) and has_marker(cache) then
		return true
	end

	if cache.parent then
		fs.create("dir_all", cache.parent)
	end

	local ok = write_thumbnail_with_ffmpeg(job, cache, at) or write_thumbnail_with_ffmpegthumbnailer(job, cache, at)
	if ok then
		if at == 0 then
			touch_marker(cache)
		else
			remove_marker(cache)
		end
	end
	return ok
end

function M:peek(job)
	local start, cache = os.clock(), ya.file_cache(job)
	if not cache or not self:preload(job) then
		return 1
	end

	local info = video_info(job)
	local image_area, text_area = split_preview(job.area, info)
	ya.sleep(math.max(0, rt.preview.image_delay / 1000 + start - os.clock()))
	ya.image_show(cache, image_area)
	ya.preview_widget(job, info_widget(job, text_area, info))
end

function M:seek(job)
	local h = cx.active.current.hovered
	if h and h.url == job.file.url then
		local step = math.floor(job.units * job.area.h / 10)
		ya.emit("peek", {
			tostring(math.max(0, cx.active.preview.skip + step)),
			only_if = tostring(job.file.url),
		})
	end
end

return M
