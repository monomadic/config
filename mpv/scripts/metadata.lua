-- osd-formatter.lua - Format OSD status message with custom logic
local mp = require "mp"

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

local function format_osd_status()
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
    local paused = mp.get_property_bool("pause", false)

    local cur = mp.get_property_number("playlist-pos", 0)
    local pl_count = mp.get_property_number("playlist-count", 0)

    -- Orientation (rotation-aware)
    local orient = get_orientation()

    -- Format file info line
    local file_info = string.format(" %s    %s    %s%s    %s",
        human_size, codec, resolution, (fps ~= "" and (" @ " .. fps) or ""), orient
    )

    -- Format based on OSD level
    local osd_level = mp.get_property_number("osd-level", 1)
    local status_msg

    if osd_level <= 2 then
        status_msg = file_info
    else
        local line1 = (paused and "PAUSED " or "") .. cur .. '/' .. pl_count .. ' ' .. artist .. title
        status_msg = line1 .. "\n" .. file_info
    end

    mp.set_property("osd-status-msg", status_msg)
end

mp.register_event("file-loaded", format_osd_status)
mp.observe_property("video-params", "native", format_osd_status)
mp.observe_property("osd-level", "number", format_osd_status)
mp.observe_property("pause", "bool", format_osd_status)

if mp.get_property("path") then
    format_osd_status()
end
