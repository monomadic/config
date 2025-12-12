local mp = require "mp"

local overlay = mp.create_osd_overlay("ass-events")
local visible = true

local function build_bar()
    -- bottom-center, size tweak as you like
    local s = [[{\an2}{\fs18}]]

    -- ASS colors: \1c&HBBGGRR& (BGR)
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
    overlay.z     = 10
    overlay:update()
end

-- redraw on new file
mp.register_event("file-loaded", render_bar)

-- redraw on resize/fullscreen changes
mp.observe_property("osd-dimensions", "native", function()
    render_bar()
end)

-- keep keybar visibility in sync with osd-level
mp.observe_property("osd-level", "number", function(_, v)
    -- osd-level 0 => hide bar, >0 => show bar
    if v and v > 0 then
        visible = true
    else
        visible = false
    end
    render_bar()
end)

-- optional manual toggle if you still want it:
--   F12 script-message keybar-toggle
mp.register_script_message("keybar-toggle", function()
    visible = not visible
    render_bar()
end)
