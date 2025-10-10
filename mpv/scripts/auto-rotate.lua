-- auto-rotate.lua - automatically rotates portrait videos to landscape
-- 
local enabled = true

local function rotate_if_portrait()
    if not enabled then return end

    local width = mp.get_property_number("width")
    local height = mp.get_property_number("height")

    if not width or not height then return end

    if height > width then
        mp.set_property_number("video-rotate", 90)
        mp.set_property_bool("keepaspect", false)
        mp.set_property_number("panscan", 1)
    else
        mp.set_property_number("video-rotate", 0)
        mp.set_property_bool("keepaspect", true)
        mp.set_property_number("panscan", 0)
    end
end

-- Apply on file load
mp.register_event("file-loaded", rotate_if_portrait)

-- Toggle message
mp.register_script_message("toggle", function()
    enabled = not enabled
    mp.osd_message("Auto Rotate: " .. (enabled and "ON" or "OFF"))
    if not enabled then
        -- reset rotation to default
        mp.set_property_number("video-rotate", 0)
        mp.set_property_bool("keepaspect", true)
        mp.set_property_number("panscan", 0)
    else
        rotate_if_portrait()
    end
end)
