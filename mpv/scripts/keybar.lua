local mp = require "mp"

local overlay = mp.create_osd_overlay("ass-events")
overlay.z = 10

local keybar_enabled = true  -- what Tab toggles
local osd_ok = true          -- derived from osd-level

-- PAN & SCAN state (mpv "panscan" property)
local panscan = 0.0
local panscan_on = false
local last_panscan = 1.0      -- restore value when turning back on
local PANSCAN_DEFAULT = 1.0   -- tweak to taste (0..1 is typical)

local function update_panscan_state(v)
    panscan = (type(v) == "number") and v or mp.get_property_number("panscan", 0.0)
    panscan_on = (panscan or 0.0) > 0.001
end
update_panscan_state()

local function build_bar(dim)
    dim = dim or mp.get_property_native("osd-dimensions")
    local w = (dim and dim.w) or 1280
    local h = (dim and dim.h) or 720

    -- font scale (unchanged behavior)
    local fs = math.floor(math.max(22, math.min(60, h * 0.02)))

    -- make the background bar taller without changing font size
    local bar_h = math.floor(math.max(fs * 2.2, h * 0.06)) -- tweak
    local bar_h_max = 220
    if bar_h > bar_h_max then bar_h = bar_h_max end

    local y0 = h - bar_h
    local bottom_pad = math.floor(math.max(6, fs * 0.35)) -- text from bottom

    local function dlg(txt)
        return ("Dialogue: 0,0:00:00.00,0:00:10.00,Default,,0,0,0,,%s"):format(txt)
    end

    -- background bar (vector draw). \alpha sets transparency (00=opaque, FF=invisible)
    local bg = ("{\\an7\\pos(0,%d)\\bord0\\shad0\\1c&H000000&\\alpha&H20&\\p1}"
        .. "m 0 0 l %d 0 l %d %d l 0 %d"
        .. "{\\p0}"):format(y0, w, w, bar_h, bar_h)

    local key_color  = "{\\1c&H00FF00&}" -- green keys
    local text_color = "{\\1c&HFFFFFF&}" -- white labels

    local function badge(on)
        local c = on and "{\\1c&H00FFFF&}" or "{\\1c&H777777&}" -- cyan / grey
        return c .. (on and "ON" or "OFF") .. text_color
    end

    local function key(label, desc)
        return key_color .. label .. text_color .. desc .. "    "
    end

    -- text line (keep same fs; just position it inside the bar)
    local s = ("{\\an2\\pos(%d,%d)\\bord0\\shad0\\fs%d}"):format(
        math.floor(w / 2),
        h - bottom_pad,
        fs
    )

    s = s
        .. key("󱊷 ", " Menu")
        .. key(" ", " OSD")
        .. key("󰌑 ", " Next")
        .. key("A",   "uto-rotate")
        .. key("I",   "nfo")
        .. key("J",   "ump in playlist")
        .. key("N",   "ext")
        .. key("P",   "an/scan " .. badge(panscan_on))
        .. key("󰘶P", "rogress Bar ")
        .. key("Q",   "uit")
        .. key("R",   "otate")
        .. key("S",   "huffle")

    return dlg(bg) .. "\n" .. dlg(s)
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

    overlay.data  = build_bar(dim)
    overlay.res_x = dim.w
    overlay.res_y = dim.h
    overlay:update()
end

mp.register_event("file-loaded", function()
    update_panscan_state()
    render_bar()
end)

mp.observe_property("osd-dimensions", "native", function()
    render_bar()
end)

mp.observe_property("osd-level", "number", function(_, v)
    osd_ok = (v and v > 0) or false
    render_bar()
end)

mp.observe_property("panscan", "number", function(_, v)
    update_panscan_state(v)
    render_bar()
end)

local function toggle_keybar()
    keybar_enabled = not keybar_enabled
    render_bar()
end

mp.register_script_message("keybar-toggle", toggle_keybar)

-- Toggle PAN & SCAN (panscan 0 <-> last/default)
local function toggle_panscan()
    update_panscan_state()
    if panscan_on then
        last_panscan = panscan
        mp.set_property_number("panscan", 0.0)
    else
        local v = (last_panscan and last_panscan > 0.001) and last_panscan or PANSCAN_DEFAULT
        mp.set_property_number("panscan", v)
    end
end

mp.register_script_message("pan-scan-toggle", toggle_panscan)
mp.add_key_binding("p", "toggle-pan-scan", toggle_panscan)

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
