local M = {}

local COLOR_LABEL = "#ffe66d"
local COLOR_CHIP_FG = "#101010"
local MAX_THUMB_ROWS = 25

-- One colour per spec chip, so each value is recognisable by colour alone.
local CHIP_COLORS = {
	resolution = "#66e2ff",
	fps = "#39ff14",
	length = "#ffe66d",
	size = "#ff79c6",
	codec = "#ffa94d",
}

-- Same tag priority as ~/.local/bin/media-open-url, so the preview shows the URL
-- that opener would actually launch.
local URL_TAGS = {
	"source_url",
	"original_url",
	"webpage_url",
	"purl",
	"url",
	"comment",
	"description",
}

-- Read orders mirror tagform's schema (src/model/schema.rs), so the preview
-- agrees with what tagform will show for the same file.
local META_FIELDS = {
	{ label = "title", keys = { "title" }, max_lines = 3 },
	{ label = "channel", keys = { "channel", "album_artist", "album" }, max_lines = 1 },
	{ label = "actors", keys = { "actors", "cast", "artist" }, max_lines = 2 },
	{ label = "url", max_lines = 1 },
	{ label = "date", keys = { "date", "com.apple.quicktime.creationdate", "creation_time" }, max_lines = 1 },
	{ label = "category", keys = { "category" }, max_lines = 1 },
}

local CODEC_NAMES = {
	h264 = "H.264",
	hevc = "HEVC",
	av1 = "AV1",
	vp9 = "VP9",
	vp8 = "VP8",
	prores = "ProRes",
	mpeg4 = "MPEG-4",
	mpeg2video = "MPEG-2",
}

-- Where the last peek drew the thumbnail and the metadata block, in screen
-- cells, so a click in the preview pane (init.lua's Preview:click) can be
-- mapped back to a region.
local set_hit = ya.sync(function(state, hit)
	state.hit = hit
end)

local resolve_click = ya.sync(function(state, x, y)
	local hit = state.hit
	local h = cx.active.current.hovered
	if not hit or not h or tostring(h.url) ~= hit.url then
		return nil
	end

	local function inside(r)
		return r and x >= r.x and x < r.x + r.w and y >= r.y and y < r.y + r.h
	end

	if inside(hit.image) then
		return "image", hit.url
	elseif inside(hit.meta) then
		return "meta", hit.url
	end
end)

local function clean(value)
	if not value or value == "" then
		return nil
	end

	value = tostring(value):gsub("[%c\r\n]", " "):gsub("^%s+", ""):gsub("%s+$", "")
	if #value > 300 then
		value = value:sub(1, 297) .. "..."
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

local function format_size(bytes)
	bytes = tonumber(bytes)
	if not bytes or bytes <= 0 then
		return nil
	end

	local units = { "B", "KB", "MB", "GB", "TB" }
	local size, unit = bytes, 1
	while size >= 1000 and unit < #units do
		size = size / 1000
		unit = unit + 1
	end
	if unit <= 2 then
		return string.format("%d %s", size, units[unit])
	end
	return string.format(size >= 100 and "%.0f %s" or "%.1f %s", size, units[unit])
end

local function format_codec(name)
	if not name then
		return nil
	end
	return CODEC_NAMES[name:lower()] or name:upper()
end

local function format_date(value)
	if not value then
		return nil
	end
	local y, m, d = value:match("^(%d%d%d%d)-?(%d%d)-?(%d%d)")
	if y then
		return string.format("%s-%s-%s", y, m, d)
	end
	return value
end

local function wrap_text(text, width)
	width = math.max(1, width or 1)
	local lines = {}
	text = tostring(text or "")

	while #text > width do
		local cut = width
		for i = width, 1, -1 do
			local c = text:sub(i, i)
			if c == " " or c == "-" or c == "_" or c == "." or c == "," then
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
				info.tags[key:sub(5):lower()] = clean(value)
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
			"stream=width,height,avg_frame_rate,r_frame_rate,codec_name:format=duration:format_tags",
			"-of",
			"default=noprint_wrappers=1",
			tostring(job.file.url),
		})
		:stdout(Command.PIPED)
		:output()

	return (output and output.status and output.status.success) and parse_video_info(output.stdout) or { tags = {} }
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

local function find_url(tags)
	for _, key in ipairs(URL_TAGS) do
		local url = tostring(tags[key] or ""):match("https?://[^%s\"]+")
		if url then
			return url
		end
	end
end

-- The old media-write-tags stored channel/actors as JSON inside `comment`.
local function legacy_comment(tags)
	local comment = tags.comment
	if not comment or comment:sub(1, 1) ~= "{" then
		return {}
	end

	local legacy = { channel = comment:match('"channel"%s*:%s*"([^"]*)"') }
	local actors = comment:match('"actors"%s*:%s*%[([^%]]*)%]')
	if actors then
		local names = {}
		for name in actors:gmatch('"([^"]*)"') do
			names[#names + 1] = name
		end
		legacy.actors = #names > 0 and table.concat(names, ", ") or nil
	end
	return legacy
end

local function metadata(info)
	local tags = info.tags or {}
	local legacy = legacy_comment(tags)
	local rows = {}

	for _, field in ipairs(META_FIELDS) do
		local value
		if field.label == "url" then
			value = find_url(tags)
		else
			for _, key in ipairs(field.keys) do
				value = tags[key]
				if value then
					break
				end
			end
			value = value or clean(legacy[field.label])
		end

		if field.label == "date" then
			value = format_date(value)
		end
		if value then
			rows[#rows + 1] = { field = field, value = value }
		end
	end
	return rows
end

local function spec_chips(job, info)
	local chips = {}
	local function push(kind, value)
		if value then
			chips[#chips + 1] = { text = " " .. value .. " ", color = CHIP_COLORS[kind] }
		end
	end

	if info.width and info.height then
		push("resolution", string.format("%d×%d", info.width, info.height))
	end
	push("fps", format_fps(info.avg_frame_rate) or format_fps(info.r_frame_rate))
	push("length", format_duration(info.duration))
	push("size", format_size(job.file.cha and job.file.cha.len))
	push("codec", format_codec(info.codec_name))
	return chips
end

-- Lay the chips out left to right, starting a new line when one would overflow.
local function chip_lines(chips, width)
	local lines, spans, used = {}, {}, 0
	for _, chip in ipairs(chips) do
		local w = ui.Line(chip.text):width()
		if used > 0 and used + 1 + w > width then
			lines[#lines + 1] = ui.Line(spans)
			spans, used = {}, 0
		end
		if used > 0 then
			spans[#spans + 1] = ui.Span(" ")
			used = used + 1
		end
		spans[#spans + 1] = ui.Span(chip.text):fg(COLOR_CHIP_FG):bg(chip.color):bold()
		used = used + w
	end
	if #spans > 0 then
		lines[#lines + 1] = ui.Line(spans)
	end
	return lines
end

-- label + value with a hanging indent, clipped to the field's line budget.
local function meta_lines(rows, width)
	local label_w = 0
	for _, row in ipairs(rows) do
		label_w = math.max(label_w, #row.field.label + 1)
	end
	local indent = label_w + 1
	local value_w = math.max(1, width - indent)

	local lines = {}
	for _, row in ipairs(rows) do
		local wrapped = wrap_text(row.value, value_w)
		local max = row.field.max_lines
		if #wrapped > max then
			local last = wrapped[max]
			wrapped[max] = last:sub(1, math.max(1, value_w - 1)) .. "…"
			for i = #wrapped, max + 1, -1 do
				wrapped[i] = nil
			end
		end

		for i, text in ipairs(wrapped) do
			local head = i == 1 and (row.field.label .. ":") or ""
			lines[#lines + 1] = ui.Line({
				ui.Span(head .. string.rep(" ", indent - #head)):fg(COLOR_LABEL),
				ui.Span(text),
			})
		end
	end
	return lines
end

local function thumb_area(area, info, text_rows)
	local max_rows = MAX_THUMB_ROWS
	if info.width and info.height and info.width > 0 and info.height > 0 and info.width > info.height then
		max_rows = math.floor(MAX_THUMB_ROWS / 2)
	end

	local image_h = math.min(max_rows, math.max(1, area.h - text_rows))
	return ui.Rect({ x = area.x, y = area.y, w = area.w, h = image_h })
end

local function plain_rect(r)
	return r and { x = r.x, y = r.y, w = r.w, h = r.h }
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

	local area = job.area
	local info = video_info(job)
	local chips = chip_lines(spec_chips(job, info), area.w)
	local meta = meta_lines(metadata(info), area.w)
	local text_rows = 1 + #chips + (#meta > 0 and 1 + #meta or 0)

	local image_area = thumb_area(area, info, text_rows)
	ya.sleep(math.max(0, rt.preview.image_delay / 1000 + start - os.clock()))
	local shown = ya.image_show(cache, image_area)
	if type(shown) ~= "userdata" and type(shown) ~= "table" then
		shown = image_area
	end

	-- Text sits one row under the drawn image, not under the box it was fitted into.
	local y = math.min(shown.y + shown.h + 1, area.y + area.h)
	local bottom = area.y + area.h
	local widgets = {}

	local chip_h = math.min(#chips, bottom - y)
	if chip_h > 0 then
		widgets[#widgets + 1] = ui.Text(chips):area(ui.Rect({ x = area.x, y = y, w = area.w, h = chip_h }))
		y = y + chip_h + 1
	end

	local meta_rect
	local meta_h = math.min(#meta, bottom - y)
	if meta_h > 0 then
		meta_rect = ui.Rect({ x = area.x, y = y, w = area.w, h = meta_h })
		widgets[#widgets + 1] = ui.Text(meta):area(meta_rect)
	end

	set_hit({
		url = tostring(job.file.url),
		image = plain_rect(shown),
		meta = plain_rect(meta_rect),
	})
	ya.preview_widget(job, widgets)
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

-- `plugin video-thumb-info -- click X Y`, sent by Preview:click in init.lua.
-- Thumbnail plays the file with the default opener; metadata opens tagform.
function M:entry(job)
	local args = job.args or {}
	if args[1] ~= "click" then
		return
	end

	local region, path = resolve_click(tonumber(args[2]) or -1, tonumber(args[3]) or -1)
	if region == "image" then
		ya.emit("open", { hovered = true })
	elseif region == "meta" then
		local url = Url(path)
		ya.emit("shell", {
			"/Users/nom/.local/bin/kitty-launch --tab --cwd "
				.. ya.quote(tostring(url.parent or "."))
				.. " -- /Users/nom/.local/bin/tagform "
				.. ya.quote(path),
			orphan = true,
		})
	end
end

return M
