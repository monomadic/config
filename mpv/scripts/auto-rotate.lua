local mp = require("mp")

local enabled = false
local rotate_applied = false
local ROTATE_LABEL = "@auto_rotate"

local function remove_rotate()
    if not rotate_applied then return end
    mp.commandv("vf", "remove", ROTATE_LABEL)
    rotate_applied = false
end

local function add_rotate()
    if rotate_applied then return end
    mp.commandv("vf", "add", ROTATE_LABEL .. ":rotate=angle=90*PI/180")
    rotate_applied = true
end

local function apply_rotate_if_needed()
    local dw = mp.get_property_number("dwidth")
    local dh = mp.get_property_number("dheight")

    if not dw or not dh or dw <= 0 or dh <= 0 then
        return
    end

    if not enabled then
        remove_rotate()
        return
    end

    -- Portrait => rotate, landscape => no rotate
    if dh > dw then
        add_rotate()
    else
        remove_rotate()
    end
end

local function on_file_loaded()
    -- New file starts clean, but keep the enabled toggle state persistent
    rotate_applied = false
    apply_rotate_if_needed()
end

local function toggle_auto_rotate()
    enabled = not enabled
    mp.osd_message("Auto-rotate: " .. (enabled and "ON" or "OFF"), 1.2)
    apply_rotate_if_needed()
end

mp.register_event("file-loaded", on_file_loaded)
mp.register_event("video-reconfig", apply_rotate_if_needed)
mp.observe_property("dwidth", "number", apply_rotate_if_needed)
mp.observe_property("dheight", "number", apply_rotate_if_needed)

-- input.conf:
-- o script-binding auto-rotate/toggle
mp.add_key_binding(nil, "toggle", toggle_auto_rotate)
