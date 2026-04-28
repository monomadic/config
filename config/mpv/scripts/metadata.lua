-- osd-formatter.lua - Format OSD status message with custom logic
local mp = require "mp"
local enabled = true
local TITLE_FONT = "Helvetica Neue"
local TITLE_DURATION_MS = 2500
local playlist_overlay = mp.create_osd_overlay("ass-events")
playlist_overlay.z = 12

local function rounded_rect_path(x0, y0, x1, y1, r)
    local c = r * 0.55228475

    return table.concat({
        string.format("m %.1f %.1f", x0 + r, y0),
        string.format("l %.1f %.1f", x1 - r, y0),
        string.format("b %.1f %.1f %.1f %.1f %.1f %.1f", x1 - r + c, y0, x1, y0 + r - c, x1, y0 + r),
        string.format("l %.1f %.1f", x1, y1 - r),
        string.format("b %.1f %.1f %.1f %.1f %.1f %.1f", x1, y1 - r + c, x1 - r + c, y1, x1 - r, y1),
        string.format("l %.1f %.1f", x0 + r, y1),
        string.format("b %.1f %.1f %.1f %.1f %.1f %.1f", x0 + r - c, y1, x0, y1 - r + c, x0, y1 - r),
        string.format("l %.1f %.1f", x0, y0 + r),
        string.format("b %.1f %.1f %.1f %.1f %.1f %.1f", x0, y0 + r - c, x0 + r - c, y0, x0 + r, y0),
    }, " ")
end

local function broadcast_state()
    mp.commandv("script-message", "metadata-state", enabled and "yes" or "no")
end

local function ass_escape(text)
    return tostring(text or "")
        :gsub("\\", "\\\\")
        :gsub("{", "\\{")
        :gsub("}", "\\}")
end

local function trim(text)
    return tostring(text or ""):match("^%s*(.-)%s*$")
end

local function metadata_value_to_text(value)
    if type(value) == "table" then
        local parts = {}

        if #value > 0 then
            for _, item in ipairs(value) do
                local text = trim(item)
                if text ~= "" then
                    table.insert(parts, text)
                end
            end
        else
            for _, item in pairs(value) do
                local text = trim(item)
                if text ~= "" then
                    table.insert(parts, text)
                end
            end
            table.sort(parts)
        end

        return table.concat(parts, ", ")
    end

    return trim(value)
end

local function get_actor_line()
    local preferred_keys = { "actors", "actor", "cast", "starring", "performer" }
    local metadata_sources = {
        mp.get_property_native("metadata") or {},
        mp.get_property_native("filtered-metadata") or {},
    }

    for _, source in ipairs(metadata_sources) do
        local normalized = {}

        for key, value in pairs(source) do
            local normalized_key = tostring(key):lower():gsub("[^%w]", "")
            normalized[normalized_key] = value
        end

        for _, key in ipairs(preferred_keys) do
            local text = metadata_value_to_text(normalized[key])
            if text ~= "" then
                return text
            end
        end
    end

    return nil
end

local function show_title_card()
    local title = trim(mp.get_property("media-title") or mp.get_property("filename") or "Untitled")
    local actors = get_actor_line()
    local lines = {
        string.format("{\\fn%s\\fs28\\b1}%s", TITLE_FONT, ass_escape(title)),
    }

    if actors and actors ~= "" then
        table.insert(lines, string.format("{\\fn%s\\fs18\\b0}%s", TITLE_FONT, ass_escape(actors)))
    end

    mp.commandv("show-text", table.concat(lines, "\n"), tostring(TITLE_DURATION_MS), "1")
end

local function get_orientation()
    -- Prefer display dimensions (already accounts for rotation/aspect)
    local dw = mp.get_property_number("dwidth", 0)
    local dh = mp.get_property_number("dheight", 0)

    -- Fallback: raw width/height + rotate metadata
    if dw <= 0 or dh <= 0 then
        dw = mp.get_property_number("width", 0)
        dh = mp.get_property_number("height", 0)

        local rot = mp.get_property_number("video-params/rotate", 0) or 0
        rot = ((rot % 360) + 360) % 360
        if rot == 90 or rot == 270 then
            dw, dh = dh, dw
        end
    end

    if dw > 0 and dh > 0 then
        if dw > dh then return "landscape"
        elseif dh > dw then return "portrait"
        else return "square"
        end
    end
    return "unknown"
end

local function update_playlist_overlay()
    if not enabled then
        playlist_overlay:remove()
        return
    end

    local osd_level = mp.get_property_number("osd-level", 1)
    local pl_count = mp.get_property_number("playlist-count", 0)

    if not osd_level or osd_level <= 2 or not pl_count or pl_count <= 1 then
        playlist_overlay:remove()
        return
    end

    local dim = mp.get_property_native("osd-dimensions")
    if not dim or not dim.w or dim.w <= 0 or not dim.h or dim.h <= 0 then
        return
    end

    local cur = mp.get_property_number("playlist-pos", 0)
    local display_cur = (cur or 0) + 1
    local label = string.format("%d of %d", display_cur, pl_count)
    local w, h = dim.w, dim.h
    local fs = math.floor(math.max(18, math.min(34, h * 0.021)))
    local pill_h = math.floor(fs * 1.55)
    local pill_w = math.floor(#label * fs * 0.54 + fs * 0.85)
    local margin = math.floor(math.max(18, h * 0.026))
    local x1 = w - margin
    local x0 = x1 - pill_w
    local y0 = margin
    local y1 = y0 + pill_h
    local radius = math.floor(pill_h / 2)
    local text_x = math.floor((x0 + x1) / 2)
    local text_y = math.floor(y0 + pill_h / 2 + fs * 0.08)

    local pill = rounded_rect_path(x0, y0, x1, y1, radius)

    playlist_overlay.data = table.concat({
        "{\\an7\\pos(0,0)\\bord0\\shad0\\1c&H000000&\\alpha&H58&\\p1}" .. pill .. "{\\p0}",
        string.format("{\\an5\\pos(%d,%d)\\fn%s\\fs%d\\b1\\bord0\\shad0\\1c&HFFFFFF&}%d {\\1c&H8A8A8A&}of %d",
            text_x, text_y, TITLE_FONT, fs, display_cur, pl_count),
    }, "\n")
    playlist_overlay.res_x = w
    playlist_overlay.res_y = h
    playlist_overlay:update()
end

local function format_osd_status()
    if not enabled then
        mp.set_property("osd-status-msg", "")
        update_playlist_overlay()
        return
    end

    -- Human-readable file size
    local fsize = mp.get_property_number("file-size", 0)
    local human_size = "Unknown size"
    if fsize > 0 then
        local units = { "B", "KB", "MB", "GB", "TB" }
        local unit_index, size = 1, fsize
        while size >= 1024 and unit_index < #units do
            size = size / 1024
            unit_index = unit_index + 1
        end
        human_size = string.format("%.1f%s", size, units[unit_index])
    end

    -- Simplified video codec
    local codec = mp.get_property("video-codec") or "unknown"
    codec = codec:match("^(.-)%s*/") or codec

    -- Convert to common names
    local codec_map = {
        hevc = "h265",
        h264 = "h264",
        avc1 = "AVC1",
        av1 = "AV1",
        vp9 = "VP9",
        vp8 = "VP8"
    }
    codec = codec_map[codec:lower()] or codec:upper()

    -- Resolution shorthand
    local height = mp.get_property_number("height", 0)
    local resolution = "unknown"
    if height >= 2160 then
        resolution = "4k"
    elseif height >= 1080 then
        resolution = "1080p"
    elseif height >= 720 then
        resolution = "720p"
    elseif height > 0 then
        resolution = tostring(height) .. "p"
    end

    -- Rounded frame rate
    local fps_str = mp.get_property("container-fps")
    local fps = ""
    if fps_str and fps_str ~= "" then
        local fps_num = tonumber(fps_str)
        if fps_num then
            fps = string.format("%dfps", math.floor(fps_num + 0.5))
        end
    end

    local title = mp.get_property("media-title") or "Untitled"
    local artist = mp.get_property("artist") or ""

    -- Orientation (rotation-aware)
    local orient = get_orientation()

    -- Format file info line
    local file_info = string.format(" %s%s   %s   %s   %s",
        resolution, (fps ~= "" and ("@" .. fps) or ""), codec, human_size, orient
    )

    -- Format based on OSD level
    local osd_level = mp.get_property_number("osd-level", 1)
    local status_msg

    if osd_level <= 2 then
        status_msg = file_info
    else
        local line1 = artist .. title
        status_msg = line1 .. "\n" .. file_info
    end

    mp.set_property("osd-status-msg", status_msg)
    update_playlist_overlay()
end

mp.register_script_message("toggle", function()
    enabled = not enabled
    broadcast_state()
    format_osd_status()
end)

mp.register_script_message("metadata-query", broadcast_state)

mp.register_event("file-loaded", function()
    format_osd_status()
    mp.add_timeout(0.05, show_title_card)
end)
mp.observe_property("video-params", "native", format_osd_status)
mp.observe_property("osd-level", "number", format_osd_status)
mp.observe_property("pause", "bool", format_osd_status)
mp.observe_property("playlist-pos", "number", format_osd_status)
mp.observe_property("playlist-count", "number", format_osd_status)
mp.observe_property("osd-dimensions", "native", format_osd_status)

if mp.get_property("path") then
    format_osd_status()
end

mp.add_timeout(0, broadcast_state)
