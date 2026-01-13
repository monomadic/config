local mp = require "mp"

local overlay = mp.create_osd_overlay("ass-events")
overlay.z = 10

local keybar_enabled = true  -- what Tab toggles
local osd_ok = true          -- derived from osd-level

-- PAN & SCAN state
local panscan = 0.0
local panscan_on = false
local last_panscan = 1.0
local PANSCAN_DEFAULT = 1.0

local function update_panscan_state(v)
    panscan = (type(v) == "number") and v or mp.get_property_number("panscan", 0.0)
    panscan_on = (panscan or 0.0) > 0.001
end
update_panscan_state()

-- RANDOM JUMP state (from the other script)
local rj_autojump_on = false
local rj_autoseek_on = false
local rj_delay = nil

local function build_bar(dim)
    dim = dim or mp.get_property_native("osd-dimensions")
    local w = (dim and dim.w) or 1280
    local h = (dim and dim.h) or 720

    local fs = math.floor(math.max(22, math.min(60, h * 0.02)))
    local bar_h = math.floor(math.max(fs * 2.2, h * 0.06))
    local bar_h_max = 220
    if bar_h > bar_h_max then bar_h = bar_h_max end

    local y0 = h - bar_h
    local bottom_pad = math.floor(math.max(6, fs * 0.35))

    local function dlg(txt)
        return ("Dialogue: 0,0:00:00.00,0:00:10.00,Default,,0,0,0,,%s"):format(txt)
    end

    local bg = ("{\\an7\\pos(0,%d)\\bord0\\shad0\\1c&H000000&\\alpha&H20&\\p1}"
        .. "m 0 0 l %d 0 l %d %d l 0 %d"
        .. "{\\p0}"):format(y0, w, w, bar_h, bar_h)

    local key_color  = "{\\1c&H00FF00&}"
    local text_color = "{\\1c&HFFFFFF&}"

    local function badge(on)
        local c = on and "{\\1c&H00FFFF&}" or "{\\1c&H777777&}"
        return c .. (on and "ON" or "OFF") .. text_color
    end

    local function fmt_delay()
        if not rj_delay then return "" end
        local d = tonumber(rj_delay)
        if not d or d <= 0 then return "" end
        if d >= 60 then
            return (" %dm"):format(math.floor(d / 60 + 0.5))
        end
        return (" %ds"):format(math.floor(d + 0.5))
    end

    local function key(label, desc)
        return key_color .. label .. text_color .. desc .. "    "
    end

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
        .. key("D",   "ir-Open")
        .. key("I",   "nfo")
        .. key("󰘶 J",   "ump " .. badge(rj_autojump_on) .. fmt_delay())
        .. key("J",   "seek " .. badge(rj_autoseek_on))
        .. key("N",   "ext")
        .. key("P",   "an/scan " .. badge(panscan_on))
        .. key("󰘶 P", "rogress Bar ")
        .. key("Q",   "uit")
        .. key("R",   "otate")
        .. key("S",   "huffle")

    return bg .. "\n" .. s
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

-- receive random-jump state updates
mp.register_script_message("keybar-randjump-state", function(a, f, d)
    rj_autojump_on = (a == "1" or a == "true")
    rj_autoseek_on = (f == "1" or f == "true")
    rj_delay = d
    render_bar()
end)

mp.register_event("file-loaded", function()
    update_panscan_state()
    render_bar()
end)

mp.observe_property("osd-dimensions", "native", function() render_bar() end)

mp.observe_property("osd-level", "number", function(_, v)
    osd_ok = (v and v > 0) or false
    render_bar()
end)

mp.observe_property("panscan", "number", function(_, v)
    update_panscan_state(v)
    render_bar()
end)

-- ask the other script for current state (best-effort)
mp.add_timeout(0, function()
    mp.commandv("script-message", "randjump-query")
end)

mp.register_script_message("keybar-toggle", function()
    keybar_enabled = not keybar_enabled
    render_bar()
end)

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

local OSD_FULL = 3
mp.add_key_binding("TAB", "toggle-osd-full", function()
    local v = mp.get_property_number("osd-level", OSD_FULL)
    if v and v > 0 then
        mp.set_property_number("osd-level", 0)
    else
        mp.set_property_number("osd-level", OSD_FULL)
    end
end)
