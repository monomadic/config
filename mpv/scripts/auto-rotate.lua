local mp = require 'mp'

local function auto_rotate()
    -- Get raw video size
    local w = mp.get_property_number("video-params/w")
    local h = mp.get_property_number("video-params/h")
    if not w or not h then return end

    -- Rotation coming from container/codec metadata
    local meta_rotate = mp.get_property_number("video-params/rotate") or 0

    -- Only mess with it if there's no metadata rotation already
    if h > w and meta_rotate == 0 then
        -- Portrait: rotate 90° clockwise
        mp.set_property_number("video-rotate", 90)
        mp.osd_message("Auto-rotated 90° (portrait)")
    else
        -- Force back to normal for non-portrait or already-rotated stuff
        mp.set_property_number("video-rotate", 0)
    end
end

mp.register_event("file-loaded", auto_rotate)
