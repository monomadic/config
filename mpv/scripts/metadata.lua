local mp = require "mp"

-- Returns a human-friendly description of the file.
local function file_info()
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

    local codec = mp.get_property("video-codec") or "unknown codec"
    codec = codec:match("^(.-)%s*/") or codec

    local height = mp.get_property_number("height", 0)
    local resolution = "unknown resolution"
    if     height >= 2160 then resolution = "4k"
    elseif height >= 1080 then resolution = "1080p"
    elseif height >= 720  then resolution = "720p"
    elseif height > 0      then resolution = tostring(height) .. "p"
    end

    local fps_str = mp.get_property("container-fps") or ""
    local fps = ""
    if fps_str ~= "" then
        local fps_num = tonumber(fps_str)
        if fps_num then
            fps = string.format("%dfps", math.floor(fps_num + 0.5))
        end
    end

    return string.format(" %s    %s    %s%s",
        human_size,
        codec,
        resolution,
        fps ~= "" and (" @ " .. fps) or ""
    )
end

mp.register_event("file-loaded", function()
    local meta = mp.get_property_native("metadata") or {}
    local titlefont = "{\\fnHelvetica Neue}"
    local nerdfont  = "{\\fnHack Nerd Font Mono}"

    local line1 = meta.title  or ""   -- title
    local line2 = meta.artist or ""   -- artist

    -- {\\an7} = top-left corner of screen (no floating box)
    local ass = string.format(
        "%s{\\an7}{\\fs14}{\\b1}%s{\\b0}\\N{\\fs10}%s\\N{\\fs8}%s%s",
        titlefont, line1,
        line2,
        nerdfont, file_info()
    )

    -- show it immediately at (0,0), aligned top-left
    mp.set_osd_ass(0, 0, ass)

    -- then clear after 5 seconds
    mp.add_timeout(5, function()
        mp.set_osd_ass(0, 0, "")
    end)
end)
