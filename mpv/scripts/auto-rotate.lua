local mp = require "mp"

local enabled = false  -- default OFF

local function auto_rotate()
    if not enabled then return end

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

    -- Case 1: encoded portrait, display NOT portrait → effectively already rotated.
    -- (e.g. container rotation / exif). Don't touch it.
    if encoded_portrait and not display_portrait then
        mp.set_property_number("video-rotate", 0)
        return
    end

    -- Case 2: still portrait even after mpv's own processing,
    -- and no rotate metadata: this is a "dumb" portrait we want to fix.
    if encoded_portrait and display_portrait and meta_rotate == 0 then
        mp.set_property_number("video-rotate", 90)
    else
        mp.set_property_number("video-rotate", 0)
    end
end

local function toggle_auto_rotate()
    enabled = not enabled
    mp.osd_message(("Auto-rotate: %s"):format(enabled and "ON" or "OFF"), 1.2)

    -- If turning ON mid-file, apply immediately.
    if enabled then
        auto_rotate()
    else
        -- Optional: when turning OFF, return to normal orientation.
        mp.set_property_number("video-rotate", 0)
    end
end

mp.register_event("file-loaded", auto_rotate)

-- Bind a key for toggling (pick whatever you like)
mp.add_key_binding(nil, "toggle-auto-rotate", toggle_auto_rotate)
-- Example binding in input.conf:
-- r script-binding toggle-auto-rotate
