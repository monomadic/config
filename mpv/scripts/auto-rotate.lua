local mp = require 'mp'

local function auto_rotate()
    local w  = mp.get_property_number("video-params/w")
    local h  = mp.get_property_number("video-params/h")
    local dw = mp.get_property_number("dwidth")
    local dh = mp.get_property_number("dheight")
    local meta_rotate = mp.get_property_number("video-params/rotate") or 0

    if not w or not h or not dw or not dh then
        return
    end

    local encoded_portrait  = h > w
    local display_portrait  = dh > dw

    -- Case 1: encoded portrait, display NOT portrait → already --cache=yes --demuxer-max-bytes=50Meffectively rotated
    -- (e.g. Finder / Exif / container rotation). Don't touch it.
    if encoded_portrait and not display_portrait then
        mp.set_property_number("video-rotate", 0)
        return
    end

    -- Case 2: still portrait even after mpv's own processing,
    -- and no rotate metadata: this is a "dumb" portrait we want to fix.
    if encoded_portrait and display_portrait and meta_rotate == 0 then
        mp.set_property_number("video-rotate", 90)
        mp.osd_message("Auto-rotated 90° (portrait)")
    else
        -- Anything else: leave it in normal orientation.
        mp.set_property_number("video-rotate", 0)
    end
end

mp.register_event("file-loaded", auto_rotate)
