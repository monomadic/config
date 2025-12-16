local mp = require "mp"

local overlay = mp.create_osd_overlay("ass-events")

local keybar_enabled = true  -- what Tab toggles
local osd_ok = true          -- derived from osd-level

local function build_bar()
    local s = [[{\an2}{\fs18}]]

    local key_color  = "{\\1c&H00FF00&}" -- green keys
    local text_color = "{\\1c&HFFFFFF&}" -- white labels

    local function key(label, desc)
        return key_color .. "[" .. label .. "] " .. text_color .. desc .. "    "
    end

    s = s
        .. key("TAB", "Toggle OSD")
        .. key("A",   "Auto-rotate")
        .. key("R",   "Rotate")
        .. key("I",   "Info")

    return s
end

local function render_bar()
    local visible = osd_ok and keybar_enabled
    if not visible then
        overlay:remove()
        return
    end

    local dim = mp.get_property_native("osd-dimensions")
    if not dim or not dim.w or dim.w <= 0 or not dim.h or dim.h <= 0 then
        return
    end

    overlay.data  = build_bar()
    overlay.res_x = dim.w
    overlay.res_y = dim.h
    overlay:update()
end

mp.register_event("file-loaded", render_bar)

mp.observe_property("osd-dimensions", "native", function()
    render_bar()
end)

mp.observe_property("osd-level", "number", function(_, v)
    osd_ok = (v and v > 0) or false
    render_bar()
end)

local function toggle_keybar()
    keybar_enabled = not keybar_enabled
    render_bar()
end

mp.register_script_message("keybar-toggle", toggle_keybar)

-- Tab toggles osd on/off
local OSD_FULL = 3

mp.add_key_binding("TAB", "toggle-osd-full", function()
    local v = mp.get_property_number("osd-level", OSD_FULL)
    if v and v > 0 then
        mp.set_property_number("osd-level", 0)
    else
        mp.set_property_number("osd-level", OSD_FULL)
    end
end)
