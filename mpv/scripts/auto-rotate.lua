local mp = require "mp"
local enabled = false

local function check_and_rotate()
    if not enabled then return end
    
    local dw = mp.get_property_number("dwidth")
    local dh = mp.get_property_number("dheight")
    
    if not dw or not dh then return end
    
    -- Clear any existing rotation first
    mp.commandv("vf", "remove", "rotate")
    
    -- Apply rotation if portrait
    if dh > dw then
        mp.commandv("vf", "add", "rotate=angle=90*PI/180")
    end
end

local function on_file_loaded()
    -- Clear rotation on new file
    mp.commandv("vf", "remove", "rotate")
end

local function toggle_auto_rotate()
    enabled = not enabled
    mp.osd_message(("Auto-rotate: %s"):format(enabled and "ON" or "OFF"), 1.2)
    
    if enabled then
        check_and_rotate()
    else
        mp.commandv("vf", "remove", "rotate")
    end
end

mp.register_event("file-loaded", on_file_loaded)
mp.observe_property("dwidth", "number", check_and_rotate)
mp.observe_property("dheight", "number", check_and_rotate)
mp.add_key_binding(nil, "toggle-auto-rotate", toggle_auto_rotate)
