-- auto-rotate.lua
local mp = require "mp"

local enabled = false  -- default OFF, change to true if you want always-on

-- apply current desired transform
local function apply_rotation()
    if not enabled then return end

    -- try video-params first (more reliable with hwdec, etc.)
    local w = mp.get_property_number("video-params/w")
    local h = mp.get_property_number("video-params/h")

    if not w or not h then
        -- fallback to plain width/height
        w = mp.get_property_number("width")
        h = mp.get_property_number("height")
    end

    if not w or not h then
        return -- still not ready
    end

    if h > w then
        -- portrait: turn it sideways and fill
        mp.commandv("set", "video-rotate", "90")
        mp.commandv("set", "keepaspect", "no")
        mp.commandv("set", "panscan", "1")
    else
        -- landscape: normal
        mp.commandv("set", "video-rotate", "0")
        mp.commandv("set", "keepaspect", "yes")
        mp.commandv("set", "panscan", "0")
    end
end

-- hard reset to defaults even if disabled
local function reset_view()
    mp.commandv("set", "video-rotate", "0")
    mp.commandv("set", "keepaspect", "yes")
    mp.commandv("set", "panscan", "0")
end

-- toggle from keybind / script-message
local function toggle()
    enabled = not enabled
    mp.osd_message("Auto Rotate: " .. (enabled and "ON" or "OFF"), 1.2)

    if enabled then
        apply_rotation()
    else
        reset_view()
    end
end

-- expose messages
mp.register_script_message("auto-rotate-toggle", toggle)

-- let other scripts query state
mp.register_script_message("auto-rotate-get-state", function()
    mp.commandv("script-message", "auto-rotate-state", enabled and "on" or "off")
end)

-- whenever a file finishes loading
mp.register_event("file-loaded", function()
    if enabled then
        apply_rotation()
    else
        reset_view()
    end
end)

-- whenever mpv detects new video params (like track switch / rotation metadata / hwdec init)
mp.observe_property("video-params", "native", function(_, _)
    if enabled then
        apply_rotation()
    end
end)
