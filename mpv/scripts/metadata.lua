-- metadata.lua - Toggleable file HUD
local mp = require "mp"
local enabled = true  -- start ON by default

-- Returns a human-friendly description of the file.
local function file_info()
    -- Human-readable file size.
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

    -- Video codec.
    local codec = mp.get_property("video-codec") or "unknown codec"
    codec = codec:match("^(.-)%s*/") or codec

    -- Determine resolution.
    local height = mp.get_property_number("height", 0)
    local resolution = "unknown resolution"
    if height >= 2160 then
        resolution = "4k"
    elseif height >= 1080 then
        resolution = "1080p"
    elseif height >= 720 then
        resolution = "720p"
    elseif height > 0 then
        resolution = tostring(height) .. "p"
    end

    -- Get frame rate.
    local fps_str = mp.get_property("container-fps")
    local fps = ""
    if fps_str and fps_str ~= "" then
        local fps_num = tonumber(fps_str)
        if fps_num then
            fps = string.format("%dfps", math.floor(fps_num + 0.5))
        end
    end

    return string.format(" %s    %s    %s%s",
        human_size, codec, resolution, (fps ~= "" and (" @ " .. fps) or ""))
end

-- Draw HUD (if enabled)
local function draw_hud()
    if not enabled then
        mp.set_osd_ass(0, 0, "") -- clear when disabled
        return
    end

    -- metadata optional; keep minimal output as in your version
    local nerdfont = " "
    local ass_text = string.format("{\\an1}{\\fs6}%s%s", nerdfont, file_info())
    mp.set_osd_ass(0, 0, ass_text)
end

-- Apply on file load and when VO reconfigures
mp.register_event("file-loaded", draw_hud)
mp.observe_property("video-params", "native", function() draw_hud() end)

-- Toggle message (script-message-to metadata toggle)
mp.register_script_message("toggle", function()
    enabled = not enabled
    mp.osd_message("Metadata HUD: " .. (enabled and "ON" or "OFF"), 1.2)
    draw_hud()
end)

-- Get state message (for menu synchronization)
mp.register_script_message("get-state", function()
    mp.commandv("script-message", "metadata-state", enabled and "on" or "off")
end)
