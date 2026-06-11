-- interpolation-toggle.lua - toggle interpolation with a transient OSD notice.
-- Uses an ASS overlay so the notice shows even at osd-level=0 (the default
-- here), where show-text/osd_message are suppressed.
local mp = require "mp"

local overlay = mp.create_osd_overlay("ass-events")
overlay.z = 40

local NOTICE_SECS = 2.5
local FONT = "Helvetica Neue"

local hide_timer = nil

local function show_notice(text)
    local dim = mp.get_property_native("osd-dimensions")
    if not dim or not dim.w or dim.w <= 0 or not dim.h or dim.h <= 0 then
        return
    end

    local fs = math.floor(math.max(16, math.min(30, dim.h * 0.022)))
    overlay.data = string.format(
        "{\\an8\\pos(%d,%d)\\fn%s\\fs%d\\b1\\bord1\\3c&H000000&\\3a&H40&\\shad0\\1c&HF2F2F2&}%s",
        math.floor(dim.w / 2), math.floor(dim.h * 0.06), FONT, fs, text
    )
    overlay.res_x = dim.w
    overlay.res_y = dim.h
    overlay:update()

    if hide_timer then
        hide_timer:kill()
    end
    hide_timer = mp.add_timeout(NOTICE_SECS, function()
        overlay:remove()
    end)
end

local function toggle()
    local on = not mp.get_property_bool("interpolation", false)
    mp.set_property_bool("interpolation", on)
    show_notice("interpolation: " .. (on and "on" or "off"))
end

mp.add_key_binding(nil, "toggle-interpolation", toggle)
mp.register_script_message("toggle-interpolation", toggle)
