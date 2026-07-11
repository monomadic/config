-- lut-select.lua - pick / toggle a creative LUT for gpu-next.
--
-- Uses mpv's native `lut` property (libplacebo, GPU-side) instead of the
-- lavfi `lut3d` CPU filter, so it works with hwdec + vo=gpu-next.
--
-- Bindings (see input.conf):
--   script-binding lut_select/menu    -- fuzzy list of .cube LUTs
--   script-binding lut_select/toggle  -- flip last-used LUT on/off
--   script-binding lut_select/clear   -- disable LUT
local mp = require "mp"
local utils = require "mp.utils"
local input = require "mp.input"

-- Folder holding the .cube LUTs (iCloud Drive).
local LUT_DIR = os.getenv("HOME") ..
    "/Library/Mobile Documents/com~apple~CloudDocs/Movies/LUTs"

-- Remembered so `toggle` can restore the last LUT after turning it off.
local last_lut = nil

local function basename(path)
    return path and path:match("([^/\\]+)$") or path
end

local function current_lut()
    local v = mp.get_property("lut", "")
    if v == "" then return nil end
    return v
end

-- The `fast` profile sets dither=no and leaves deband off, which is fine for
-- untouched playback but makes a grading LUT expose 8-bit banding and the
-- source's compression blocks as macroblocking. Turn those quality features on
-- only while a LUT is active, and restore the lean defaults when it is cleared.
local saved_quality = nil

local function set_grade_quality(on)
    if on then
        if not saved_quality then
            saved_quality = {
                dither = mp.get_property("dither"),
                deband = mp.get_property("deband"),
            }
        end
        mp.set_property("dither", "fruit")
        mp.set_property("deband", "yes")
    elseif saved_quality then
        mp.set_property("dither", saved_quality.dither)
        mp.set_property("deband", saved_quality.deband)
        saved_quality = nil
    end
end

local function apply_lut(path)
    mp.set_property("lut", path or "")
    set_grade_quality(path ~= nil)
    if path then
        last_lut = path
        mp.osd_message("LUT: " .. basename(path))
    else
        mp.osd_message("LUT: off")
    end
end

-- Return a sorted list of absolute .cube paths, or nil + message on failure.
local function list_luts()
    local files = utils.readdir(LUT_DIR, "files")
    if not files then
        mp.osd_message("LUT dir not found:\n" .. LUT_DIR, 4)
        return nil
    end
    local luts = {}
    for _, f in ipairs(files) do
        if f:lower():match("%.cube$") then
            luts[#luts + 1] = utils.join_path(LUT_DIR, f)
        end
    end
    table.sort(luts, function(a, b)
        return basename(a):lower() < basename(b):lower()
    end)
    if #luts == 0 then
        mp.osd_message("No .cube LUTs in\n" .. LUT_DIR, 4)
        return nil
    end
    return luts
end

local function show_menu()
    local luts = list_luts()
    if not luts then return end

    local active = current_lut()
    local items = { active and "None  (disable LUT)" or "None  ← active" }
    for _, path in ipairs(luts) do
        local name = basename(path):gsub("%.cube$", "")
        if active == path then
            name = name .. "  ← active"
        end
        items[#items + 1] = name
    end

    input.select({
        prompt = "Select LUT",
        items = items,
        submit = function(index)
            if not index then return end
            if index == 1 then
                apply_lut(nil)
            else
                apply_lut(luts[index - 1])
            end
        end,
    })
end

-- Index of the active LUT within `luts`, or nil if none / not in the folder.
local function active_index(luts)
    local active = current_lut()
    if not active then return nil end
    for i, path in ipairs(luts) do
        if path == active then return i end
    end
    return nil
end

-- Step forward/backward through the folder, applying each LUT (a manual
-- "preview by cycling"). Wraps around. With nothing active, +1 lands on the
-- first LUT and -1 on the last.
local function step(dir)
    local luts = list_luts()
    if not luts then return end
    local idx = active_index(luts)
    local next_idx
    if not idx then
        next_idx = (dir > 0) and 1 or #luts
    else
        next_idx = ((idx - 1 + dir) % #luts) + 1
    end
    apply_lut(luts[next_idx])
end

-- Turn the LUT on: restore the last-used one, or the first in the folder.
local function lut_on()
    if last_lut then
        apply_lut(last_lut)
    else
        local luts = list_luts()
        if luts then apply_lut(luts[1]) end
    end
end

-- Flip the last-used LUT on/off.
local function toggle()
    if current_lut() then
        apply_lut(nil)
    else
        lut_on()
    end
end

mp.add_key_binding(nil, "menu", show_menu)
mp.add_key_binding(nil, "next", function() step(1) end)
mp.add_key_binding(nil, "prev", function() step(-1) end)
mp.add_key_binding(nil, "on", lut_on)
mp.add_key_binding(nil, "off", function() apply_lut(nil) end)
mp.add_key_binding(nil, "toggle", toggle)
mp.add_key_binding(nil, "clear", function() apply_lut(nil) end)
