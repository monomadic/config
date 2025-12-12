-- osd-formatter.lua - Format OSD status message with custom logic
local mp = require "mp"

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
        hevc = "H.265",
        h264 = "H.264",
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

    -- Get title and pause state
    local title = mp.get_property("media-title") or "Unknown"
    -- local paused = mp.get_property_bool("pause", false)
    
    -- Format file info line
    local file_info = string.format(" %s    %s    %s%s",
        human_size, codec, resolution, (fps ~= "" and (" @ " .. fps) or ""))
    
    -- Format based on OSD level
    -- Note: osd-status-msg only appears at levels 2 and 3 (level 1 only shows seekbar)
    local osd_level = mp.get_property_number("osd-level", 1)
    local status_msg
    
    if osd_level <= 2 then
        -- Level 2 (Minimal): only file info
        status_msg = file_info
    else
        -- Level 3 (Full): title + file info
        local line1 = (paused and "(Paused) " or "") .. title
        status_msg = line1 .. "\n" .. file_info
    end
    
    mp.set_property("osd-status-msg", status_msg)
end

-- Update on file load and when video params change
mp.register_event("file-loaded", format_osd_status)
mp.observe_property("video-params", "native", format_osd_status)
mp.observe_property("osd-level", "number", format_osd_status)  -- Update when OSD level changes
mp.observe_property("pause", "bool", format_osd_status)  -- Update when pause state changes

-- Show OSD briefly when file loads
-- mp.register_event("file-loaded", function()
--     mp.command("show-text ${osd-status-msg} 3000")
-- end)

-- Apply format immediately if file is already loaded (script loaded mid-playback)
if mp.get_property("path") then
    format_osd_status()
end
