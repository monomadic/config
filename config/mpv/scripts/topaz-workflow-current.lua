local utils = require "mp.utils"

local kitty_launch = "/Users/nom/.zsh/bin/kitty-launch"
local topaz_workflow = "/Users/nom/.zsh/bin/topaz-workflow"
local topaz_run = "/Users/nom/.zsh/bin/topaz-encode"
local topaz_preview_frame = "/Users/nom/.zsh/bin/topaz-preview-frame"
local preset_catalog = "/Users/nom/config/config/zsh/bin/lib/topaz-preset-catalog.zsh"

-- Active render-menu session (nil when closed). One sheet with three tabs
-- (Enhance / Interpolate / Output); every choice lives in this table and the
-- encode launches directly from it.
local menu = nil
local pending_restore = nil

local list_badge = mp.create_osd_overlay("ass-events")
list_badge.z = 30
local params_badge = mp.create_osd_overlay("ass-events")
params_badge.z = 30
local detail_badge = mp.create_osd_overlay("ass-events")
detail_badge.z = 30
-- Preset-details companion sheet visibility (toggled with `d`, on by default);
-- deliberately a script-local so the choice survives closing and reopening the menu.
local details_visible = true
local encode_timer = nil
local encode_start = nil

-- Enhancement categories shown in the Enhance tab, in display order.
local CATEGORY_ORDER = {
    { key = "detail", label = "Detail" },
    { key = "repair", label = "Repair" },
    { key = "sharpen", label = "Sharpen & Texture" },
    { key = "focus-fix", label = "Focus Fix" },
}

local TABS = { "Enhance", "Interpolate", "Output" }

-- Forward declarations (so the menu functions can reference each other).
local draw_menu, show_original, render_or_show, move_cursor, select_number
local render_cursor, start_encode_now, close_menu, enable_menu_keys, remove_menu_keys
local space_toggle, show_render, show_orig, toggle_ui, menu_seek
local draw_details, toggle_details, set_tab, interp_select, output_select
local save_job_file, copy_encode_command

local function shell_quote(value)
    return "'" .. tostring(value):gsub("'", "'\\''") .. "'"
end

local function trim(value)
    return (value or ""):gsub("^%s+", ""):gsub("%s+$", "")
end

local function basename(path)
    return path and path:match("([^/\\]+)$") or path
end

local function format_time(seconds)
    local total = tonumber(seconds) or 0
    local hours = math.floor(total / 3600)
    local minutes = math.floor((total % 3600) / 60)
    local secs = total % 60

    if hours > 0 then
        return string.format("%d:%02d:%06.3f", hours, minutes, secs)
    end

    return string.format("%d:%06.3f", minutes, secs)
end

-- Compact render-duration label (e.g. "1.4s", "1:05") for the per-preset readout.
local function short_secs(seconds)
    local s = tonumber(seconds)
    if not s then
        return ""
    end
    if s >= 60 then
        return string.format("%d:%02d", math.floor(s / 60), math.floor(s % 60))
    end
    return string.format("%.1fs", s)
end

-- Whole-second clock (M:SS or H:MM:SS) for the clip-length caption.
local function format_clock(seconds)
    local total = math.floor(tonumber(seconds) or 0)
    local h = math.floor(total / 3600)
    local m = math.floor((total % 3600) / 60)
    local s = total % 60
    if h > 0 then
        return string.format("%d:%02d:%02d", h, m, s)
    end
    return string.format("%d:%02d", m, s)
end

-- Split a preset display name into (model, variant) for the two-tone row title:
-- "Proteus — Detail Max" -> "Proteus", "Detail Max"; "Nyx Heavy Denoise" -> "Nyx",
-- "Heavy Denoise". Resolution tokens are dropped if present.
local TWO_WORD_MODELS = { "Starlight Mini", "Gaia HQ", "Focus Fix", "Proteus Deblur" }

local function split_display(display)
    local d = trim((display or ""):gsub("%s+2x", ""):gsub("%s+4K", ""))
    local model, variant = d:match("^(.-)%s+—%s+(.+)$")
    if model then
        return model, variant
    end
    for _, m in ipairs(TWO_WORD_MODELS) do
        if d == m then
            return d, nil
        end
        local rest = d:match("^" .. m .. "%s+(.+)$")
        if rest then
            return m, rest
        end
    end
    model, variant = d:match("^(%S+)%s+(.+)$")
    if model then
        return model, variant
    end
    return d, nil
end

-- Escape text for use inside an ASS event (literal braces/backslashes).
local function ass_escape(value)
    return (tostring(value or ""))
        :gsub("\\", "\\\\")
        :gsub("{", "\\{")
        :gsub("}", "\\}")
end

-- UTF-8-aware character count (bytes that are not continuation bytes), so multi-byte
-- key glyphs like ⇧ don't over-inflate keycap widths.
local function disp_len(value)
    local _, n = tostring(value or ""):gsub("[^\128-\191]", "")
    return n
end

-- ===== ASS drawing helpers / shared menu layout =====
-- Coordinates are in the 1280x720 virtual space used by every overlay here.

local FONT = "Helvetica Neue"  -- system UI font for the whole panel

-- Floating "sheet" geometry (iOS grouped-list feel): the panel sits inset from the
-- top-left and grows only as tall as its content.
local MARGIN_L = 14     -- gap from the left screen edge (so the sheet floats)
local MARGIN_T = 12     -- gap from the top screen edge
local PANEL_W = 340     -- floating panel width
local PAD = 14          -- panel edge -> content inset
local LIST_X = MARGIN_L + PAD        -- group / row left edge (also mouse hit-test left)
local ROW_W = PANEL_W - 2 * PAD      -- group + row width (also mouse hit-test right)
local RH = 30           -- preset / option row height (two text lines)
local HH = 22           -- section-header row advance
local TAB_Y = MARGIN_T + 36          -- segmented tab bar top
local TAB_H = 24                     -- segmented tab bar height
local LIST_TOP = TAB_Y + TAB_H + 14  -- y where the first group starts
local CAP_W = 16        -- index keycap width
local CAP_H = 16        -- index keycap height
local KEY_X = LIST_X + 10             -- keycap left, inset inside the group card
local TEXT_X = KEY_X + CAP_W + 10     -- row-text left (fixed; digit-count independent)
local PANEL_X = MARGIN_L

-- One accent colour only (iOS blue #0A84FF); green marks a cached still. ASS is BGR.
local ACCENT = "&HFF840A&"
local GREEN = "&H58D130&"

-- Rectangle drawing path (origin at the event \pos).
local function ass_rect(w, h)
    return string.format("m 0 0 l %d 0 l %d %d l 0 %d", w, w, h, h)
end

-- Rounded rectangle path (origin at the event \pos), corner radius r.
local function ass_round_rect(w, h, r)
    return table.concat({
        "m", r, 0,
        "l", w - r, 0,
        "b", w, 0, w, 0, w, r,
        "l", w, h - r,
        "b", w, h, w, h, w - r, h,
        "l", r, h,
        "b", 0, h, 0, h, 0, h - r,
        "l", 0, r,
        "b", 0, 0, 0, 0, r, 0,
    }, " ")
end

-- Rounded rect with independent top / bottom corner radii, so a full-width row
-- highlight can round to match the group card's ends yet stay square where rows meet.
local function ass_round_rect2(w, h, rt, rb)
    local p = { string.format("m %g 0", rt), string.format("l %g 0", w - rt) }
    p[#p + 1] = rt > 0 and string.format("b %g 0 %g 0 %g %g", w, w, w, rt)
        or string.format("l %g 0", w)
    p[#p + 1] = string.format("l %g %g", w, h - rb)
    p[#p + 1] = rb > 0 and string.format("b %g %g %g %g %g %g", w, h, w, h, w - rb, h)
        or string.format("l %g %g", w, h)
    p[#p + 1] = string.format("l %g %g", rb, h)
    p[#p + 1] = rb > 0 and string.format("b 0 %g 0 %g 0 %g", h, h, h - rb)
        or string.format("l 0 %g", h)
    p[#p + 1] = string.format("l 0 %g", rt)
    p[#p + 1] = rt > 0 and string.format("b 0 0 0 0 %g 0", rt) or "l 0 0"
    return table.concat(p, " ")
end

-- The index "keycap": a low-contrast rounded block with a white number on top, so it
-- reads as a quiet keyboard hint rather than shouting. `on_accent` lifts the fill a
-- little for legibility on the accent-filled selected row. Centred on `center_y`.
-- Returns two ASS events (cap, number); row text starts at the fixed TEXT_X.
local function keycap_events(center_y, number, on_accent)
    local top = center_y - CAP_H / 2
    local alpha = on_accent and "&HC4&" or "&HDA&"
    local cap = string.format(
        "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c&HFFFFFF&\\1a%s\\p1}%s{\\p0}",
        KEY_X, top, alpha, ass_round_rect(CAP_W, CAP_H, 6))
    local num = string.format(
        "{\\an5\\pos(%d,%d)\\bord0\\shad1\\4c&H000000&\\4a&HC0&\\fn%s\\fs11\\b1\\1c&HFFFFFF&}%s",
        KEY_X + math.floor(CAP_W / 2), center_y, FONT, tostring(number))
    return cap, num
end

-- A full-width row fill, flush to the group-card edges. rt / rb are the top / bottom
-- corner radii so the highlight rounds to match the card's ends but stays square where
-- rows meet. `alpha` is an ASS alpha string (&H00& opaque .. &HFF& transparent). Used
-- for the blue selection highlight, the white rendering throb, and cursor / hover focus.
local function row_pill(y, color, alpha, rt, rb)
    return string.format(
        "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c%s\\1a%s\\p1}%s{\\p0}",
        LIST_X, y, color, alpha, ass_round_rect2(ROW_W, RH, rt or 0, rb or 0))
end

-- Blue accent pill marking the active selection (shown render / chosen option).
local function selection_pill_event(y, rt, rb)
    return row_pill(y, ACCENT, "&H0E&", rt, rb)
end

-- Small outlined eye glyph (lens + pupil), centred on (cx, cy). Marks the shown row.
local function eye_events(cx, cy, color)
    local lens = string.format(
        "{\\an7\\pos(%d,%d)\\bord1.1\\shad0\\1a&HFF&\\3c%s\\3a&H00&\\p1}"
            .. "m -5 0 b -2.2 -3.6 2.2 -3.6 5 0 b 2.2 3.6 -2.2 3.6 -5 0{\\p0}", cx, cy, color)
    local pupil = string.format(
        "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c%s\\p1}"
            .. "m 0 -1.4 b 0.77 -1.4 1.4 -0.77 1.4 0 b 1.4 0.77 0.77 1.4 0 1.4"
            .. " b -0.77 1.4 -1.4 0.77 -1.4 0 b -1.4 -0.77 -0.77 -1.4 0 -1.4{\\p0}",
        cx, cy, color)
    return lens, pupil
end

-- Subtle rounded "card" behind one section's rows (iOS grouped-list look). Returns the
-- card fill plus faint inset hairline separators between its `count` rows.
local function group_card_events(y0, count)
    local ev = { string.format(
        "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c&HFFFFFF&\\1a&HF2&\\p1}%s{\\p0}",
        LIST_X, y0, ass_round_rect(ROW_W, count * RH, 12)) }
    local sep_w = ROW_W - (TEXT_X - LIST_X) - 12
    for i = 1, count - 1 do
        ev[#ev + 1] = string.format(
            "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c&HFFFFFF&\\1a&HE8&\\p1}%s{\\p0}",
            TEXT_X, y0 + i * RH, ass_rect(sep_w, 1))
    end
    return ev
end

-- Dark, translucent floating "sheet" backing the menu / details panel. Rounded on
-- every corner with a faint light hairline — an iOS grouped-list card.
local function panel_bg_event(top, bottom, x, w)
    return string.format(
        "{\\an7\\pos(%d,%d)\\bord1\\shad0\\1c&H14100F&\\1a&H1E&\\3c&HFFFFFF&\\3a&HDA&\\p1}%s{\\p0}",
        x or PANEL_X, top, ass_round_rect(w or PANEL_W, bottom - top, 20))
end

-- ===== keyboard-hint bar (macOS-style keycaps) =====

-- Draw one keycap centred vertically on `cy`, starting at `x`. Returns the next x.
local function draw_hint_key(ev, x, cy, label)
    local w = math.max(16, 8 + disp_len(label) * 7)
    local top = cy - 8
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(%d,%d)\\bord0\\shad1\\4c&H000000&\\4a&HC4&\\1c&HFFFFFF&\\1a&HD2&\\p1}%s{\\p0}",
        x, top, ass_round_rect(w, 16, 7))
    ev[#ev + 1] = string.format(
        "{\\an5\\pos(%d,%d)\\bord0\\shad1\\4c&H000000&\\4a&HC0&\\fn%s\\fs10\\b1\\1c&HFFFFFF&}%s",
        x + math.floor(w / 2), cy, FONT, ass_escape(label))
    return x + w + 4
end

-- Draw a dim label after a keycap group. Returns an approximate next x.
local function draw_hint_label(ev, x, cy, text)
    ev[#ev + 1] = string.format(
        "{\\an4\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs10\\1c&HA6A6A6&}%s",
        x, cy, FONT, ass_escape(text))
    return x + disp_len(text) * 6 + 12
end

-- Render a sequence of { keys = {...}, label = "..." } segments as a hint bar.
local function draw_hint_segments(ev, x, cy, segments)
    for _, seg in ipairs(segments) do
        for _, key in ipairs(seg.keys) do
            x = draw_hint_key(ev, x, cy, key)
        end
        if seg.label and seg.label ~= "" then
            x = draw_hint_label(ev, x + 3, cy, seg.label)
        end
        x = x + 8
    end
    return x
end

-- Shortcut legends for the full-width bottom bar (one line, per tab).
local ENHANCE_HINTS = {
    { keys = { "0–9" }, label = "render" },
    { keys = { "J", "K" }, label = "move" },
    { keys = { "Space" }, label = "A/B" },
    { keys = { "⇧←", "⇧→" }, label = "±10s" },
    { keys = { "D" }, label = "details" },
    { keys = { "Tab" }, label = "tabs" },
    { keys = { "F" }, label = "hide UI" },
    { keys = { "Esc" }, label = "close" },
}
local OPTION_HINTS = {
    { keys = { "1–9" }, label = "select" },
    { keys = { "J", "K" }, label = "move" },
    { keys = { "Enter" }, label = "select" },
    { keys = { "Space" }, label = "A/B" },
    { keys = { "Tab" }, label = "tabs" },
    { keys = { "Esc" }, label = "back" },
}
-- Finalize actions, right-aligned in the bar on every tab. Each closes the menu.
local FINALIZE_HINTS = {
    { keys = { "⌘S" }, label = "save to file" },
    { keys = { "⌘↩" }, label = "start encoding now" },
    { keys = { "⌘C" }, label = "copy CLI command" },
}

-- Approximate rendered width of a hint-segment run (mirrors draw_hint_segments),
-- so a run can be right-aligned against the bar's right padding.
local function measure_hint_segments(segments)
    local x = 0
    for _, seg in ipairs(segments) do
        for _, key in ipairs(seg.keys) do
            x = x + math.max(16, 8 + disp_len(key) * 7) + 4
        end
        if seg.label and seg.label ~= "" then
            x = x + 3 + disp_len(seg.label) * 6 + 12
        end
        x = x + 8
    end
    return x
end

-- A single full-width bar pinned to the bottom of the screen: shortcut keys on the
-- left, and on the right either the on-screen resolution (Enhance tab) or the
-- pending-encode summary (Interpolate / Output tabs).
local function draw_bottom(opts)
    opts = opts or {}
    local hints = opts.hints
    local right_hints = opts.right_hints
    local right_value = opts.right_value
    local right_label = opts.right_label or ""
    local right_text = trim(opts.right_text or "")
    local has_res = right_value and right_value ~= ""

    if not hints and not right_hints and not has_res and right_text == "" then
        params_badge:remove()
        return
    end

    params_badge.res_x = 1280
    params_badge.res_y = 720

    local pad = 18
    local bar_h = 32
    local left = MARGIN_L
    local width = 1280 - 2 * MARGIN_L
    local top = 720 - MARGIN_T - bar_h
    local cy = top + math.floor(bar_h / 2)
    local rx = left + width - pad

    local ev = { string.format(
        "{\\an7\\pos(%d,%d)\\bord1\\shad0\\1c&H14100F&\\1a&H1E&\\3c&HFFFFFF&\\3a&HDA&\\p1}%s{\\p0}",
        left, top, ass_round_rect(width, bar_h, 15)) }

    if hints then
        draw_hint_segments(ev, left + pad, cy, hints)
    end

    if right_hints then
        -- +8: the trailing segment gap doesn't render, so reclaim it when aligning.
        draw_hint_segments(ev, rx - measure_hint_segments(right_hints) + 8, cy, right_hints)
    elseif has_res then
        ev[#ev + 1] = string.format(
            "{\\an6\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs13\\b1\\1c&HFFFFFF&}%s",
            rx, cy, FONT, ass_escape(right_value))
        if right_label ~= "" then
            ev[#ev + 1] = string.format(
                "{\\an6\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs11\\1c&H9A9A9A&}%s",
                rx - disp_len(right_value) * 8 - 12, cy, FONT, ass_escape(right_label))
        end
    elseif right_text ~= "" then
        ev[#ev + 1] = string.format(
            "{\\an6\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs10\\1c&HA6A6A6&}%s",
            rx, cy, FONT, ass_escape(right_text:sub(1, 130)))
    end

    params_badge.data = table.concat(ev, "\n")
    params_badge:update()
end

-- Detected source video properties.
local function source_profile()
    local w = mp.get_property_number("width")
    local h = mp.get_property_number("height")
    local fps = mp.get_property_number("container-fps")
        or mp.get_property_number("estimated-vf-fps")

    if not (w and h and w > 0 and h > 0) then
        return { show_all = true }
    end

    local long_edge = math.max(w, h)
    return {
        width = w,
        height = h,
        fps = fps,
        long_edge = long_edge,
        portrait = h > w,
        -- long-edge band: <=2560 treated as "<4K" (upscale offered), >2560 as "4K"
        is_4k = long_edge > 2560,
    }
end

-- Orientation-aware 4K target: long edge -> 3840, aspect preserved, even dims.
local function target_4k_dims(w, h)
    if not (w and h and w > 0 and h > 0) then
        return 3840, 2160
    end

    local function even(value)
        return math.max(2, math.floor(value / 2 + 0.5) * 2)
    end

    if w >= h then
        return 3840, even(h * 3840 / w)
    end
    return even(w * 3840 / h), 3840
end

local function absolute_media_path()
    local media_path = mp.get_property("path")

    if not media_path or media_path == "" then
        return nil, "No media loaded"
    end

    local proto = mp.get_property("protocol") or ""
    if proto ~= "" and proto ~= "file" then
        return nil, "Not a local file (" .. proto .. ")"
    end

    if media_path:sub(1, 1) == "/" then
        return media_path
    end

    return utils.join_path(mp.get_property("working-directory") or "", media_path)
end

local function check_current_file()
    local abs, err = absolute_media_path()
    if not abs then
        return nil, err
    end

    local info = utils.file_info(abs)
    if not info then
        return nil, "File no longer exists:\n" .. abs
    end

    if info.is_dir then
        return nil, "Topaz needs a media file"
    end

    return abs
end

local function split_tsv(line)
    local fields = {}
    local start = 1

    while true do
        local tab = line:find("\t", start, true)
        if not tab then
            table.insert(fields, line:sub(start))
            break
        end
        table.insert(fields, line:sub(start, tab - 1))
        start = tab + 1
    end

    return fields
end

local function catalog_stdout(function_name)
    local result = utils.subprocess({
        args = {
            "/bin/zsh",
            "-lc",
            "source " .. shell_quote(preset_catalog) .. "; " .. function_name,
        },
        cancellable = false,
    })

    if not result or result.status ~= 0 then
        local stderr = result and trim(result.stderr) or ""
        return nil, stderr ~= "" and stderr or "Could not read Topaz preset catalog"
    end

    return result.stdout or ""
end

local function parse_preview_paths(stdout)
    local paths = {}

    for line in (stdout or ""):gmatch("[^\r\n]+") do
        local key, value = line:match("^TOPAZ_PREVIEW_([A-Z]+)=(.+)$")
        if key and value then
            paths[key:lower()] = value
        end
    end

    return paths
end

-- ===== output resolutions (decoupled from the enhancement presets) =====
-- Options are input-aware: portrait sources keep portrait dims; on exactly-1080p
-- sources 2x IS the 4K target so the 2x row is omitted; 4K sources only offer
-- Original. Each enhancement preset declares which scales it supports (`scales`
-- catalog column) and the Output tab greys out what the chosen preset can't do.

-- Fallback order when the user's resolution choice isn't supported by a preset.
local RES_PRIORITY = {
    orig = { "orig", "2x", "4k" },
    ["2x"] = { "2x", "4k", "orig" },
    ["4k"] = { "4k", "2x", "orig" },
}

local function build_res_options(profile)
    local sw, sh = profile.width, profile.height
    if not sw then
        return { { key = "orig", label = "Original", short = "source" } }, 1, false
    end

    local tw, th = target_4k_dims(sw, sh)
    local two_x_is_4k = (sw * 2 == tw and sh * 2 == th)

    local opts = {
        { key = "orig", label = "Original", short = string.format("%d×%d", sw, sh),
          w = sw, h = sh, blurb = string.format("%d×%d — source resolution", sw, sh) },
    }
    if not profile.is_4k then
        if not two_x_is_4k then
            opts[#opts + 1] = { key = "2x", label = "Upscale 2x", short = "2x",
                w = sw * 2, h = sh * 2,
                blurb = string.format("%d×%d", sw * 2, sh * 2) }
        end
        opts[#opts + 1] = { key = "4k", label = "Upscale to 4K", short = "4K",
            w = tw, h = th, blurb = string.format("%d×%d", tw, th) }
    end

    -- Default: 4K whenever the source is below 4K, else Original.
    local default = 1
    for i, o in ipairs(opts) do
        if o.key == "4k" then
            default = i
        end
    end
    return opts, default, two_x_is_4k
end

-- Can `preset` render at resolution option `opt`? (`two_x` = 2x reaches the 4K
-- target on this source, so scale=2-only models can still hit "4K".)
local function res_available(preset, opt, two_x)
    local s = preset.scales
    if opt.key == "orig" then
        return s["1"] == true
    end
    if opt.key == "2x" then
        return s["2"] == true
    end
    return s["4k"] == true or (two_x and s["2"] == true)
end

-- The resolution actually used for `preset`: the user's choice when supported,
-- else the nearest supported fallback (Focus Fix -> Original, Starlight -> 2x...).
local function effective_res(preset)
    local sel = menu.res_options[menu.res_sel]
    if res_available(preset, sel, menu.two_x_is_4k) then
        return sel
    end
    for _, key in ipairs(RES_PRIORITY[sel.key]) do
        for _, o in ipairs(menu.res_options) do
            if o.key == key and res_available(preset, o, menu.two_x_is_4k) then
                return o
            end
        end
    end
    return menu.res_options[1]
end

-- Enhancement filter for `preset` at resolution `res`: returns (body, tail), both
-- comma-less parts. The @SCALE@ token becomes scale=1 / scale=2 / scale=0:w:h (the
-- forced-4K form gains a lanczos tail). "Original (no enhancement)" upscales — when
-- asked to — with a plain lanczos scale and no AI pass.
local function enh_filter_for(preset, res)
    if preset.is_original then
        if res.key == "orig" then
            return "", ""
        end
        return "", string.format("scale=w=%d:h=%d:flags=lanczos:threads=0", res.w, res.h)
    end

    local clause, tail = "scale=1", ""
    if res.key == "2x" then
        clause = "scale=2"
    elseif res.key == "4k" then
        if preset.scales["4k"] then
            clause = string.format("scale=0:w=%d:h=%d", res.w, res.h)
            tail = string.format("scale=w=%d:h=%d:flags=lanczos:threads=0", res.w, res.h)
        else
            clause = "scale=2"  -- 2x == 4K target on this source
        end
    end
    return (preset.filter_body:gsub("@SCALE@", clause)), tail
end

-- Preview-cache key: one still per preset per resolution actually used.
local function key_for(preset)
    return preset.slug .. "|" .. effective_res(preset).key
end

-- The enhancement the encode will use: the shown render, else Original.
local function chosen_enh()
    if menu.shown_slug then
        for _, p in ipairs(menu.presets) do
            if p.slug == menu.shown_slug then
                return p
            end
        end
    end
    return menu.presets[1]  -- the Original pseudo-preset is always first
end

-- ===== catalog loading =====

-- Read enhancement presets, group by category, and hide presets that can't reach
-- any offered resolution (e.g. 2x-only models on a 4K source).
-- Returns { items = {header|preset...}, presets = {selectable in order} }.
local function load_enhancement_presets(profile, res_options, two_x_is_4k)
    local stdout, err = catalog_stdout("topaz_enhancement_preset_rows")
    if not stdout then
        return nil, err
    end

    local by_cat = {}
    for line in stdout:gmatch("[^\r\n]+") do
        local f = split_tsv(line)
        -- category, display, slug, scales, filter_body, blurb, metadata
        if #f >= 5 then
            local scales = {}
            for tok in (f[4] or ""):gmatch("[^,%s]+") do
                scales[tok] = true
            end
            local cat = f[1]
            by_cat[cat] = by_cat[cat] or {}
            table.insert(by_cat[cat], {
                category = cat,
                display = f[2],
                slug = f[3],
                scales = scales,
                filter_body = f[5] or "",
                blurb = f[6] or "",
                metadata = f[7] or "",
            })
        end
    end

    local items = {}
    local presets = {}
    local number = 0

    -- "Do nothing" option: skip enhancement (upscales via plain lanczos if the
    -- Output tab asks for resolution). Numbered 0; always first.
    do
        local original = {
            is_original = true,
            display = "Original (no enhancement)",
            blurb = "Skip enhance — re-encode / plain scale only",
            slug = "__original__",
            cat_label = "Original",
            scales = { ["1"] = true, ["2"] = true, ["4k"] = true },
            filter_body = "",
            number = 0,
            metadata = "videoai=Original (no enhancement)",
        }
        table.insert(presets, original)
        table.insert(items, { kind = "preset", preset = original })
    end

    for _, cat in ipairs(CATEGORY_ORDER) do
        local rows = by_cat[cat.key]
        if rows and #rows > 0 then
            local visible = {}
            for _, p in ipairs(rows) do
                local ok = false
                for _, opt in ipairs(res_options) do
                    if res_available(p, opt, two_x_is_4k) then
                        ok = true
                        break
                    end
                end
                if ok then
                    visible[#visible + 1] = p
                end
            end
            if #visible > 0 then
                table.insert(items, { kind = "header", label = cat.label })
                for _, p in ipairs(visible) do
                    p.cat_label = cat.label
                    number = number + 1
                    p.number = number
                    table.insert(presets, p)
                    table.insert(items, { kind = "preset", preset = p })
                end
            end
        end
    end

    if #presets <= 1 then
        return nil, "No enhancement presets matched this source"
    end

    -- index within presets[] (for cursor) and a number -> preset map (for 0-9 keys)
    local by_number = {}
    for i, p in ipairs(presets) do
        p.index = i
        by_number[p.number] = p
    end

    return { items = items, presets = presets, by_number = by_number }
end

-- Interpolation rows for the Interpolate tab; row 1 is always "Off" (the default).
local function load_interp_rows()
    local rows = { {
        display = "Off",
        title = "Off",
        blurb = "Keep the source frame rate",
        fi = nil,
        detail = "Encode without motion interpolation.",
    } }
    local stdout = catalog_stdout("topaz_interpolation_preset_rows")
    if stdout then
        for line in stdout:gmatch("[^\r\n]+") do
            local f = split_tsv(line)
            -- display, slug, fi_filter, metadata
            if #f >= 3 then
                local display = f[1]
                local title, blurb = display:match("^(.-)%s+—%s+(.+)$")
                table.insert(rows, {
                    display = display,
                    title = title or display,
                    blurb = blurb or "",
                    slug = f[2],
                    fi = f[3],
                    detail = (f[4] or ""):gsub("^videoai=", ""),
                })
            end
        end
    end
    return rows
end

-- Output format rows for the Output tab.
local function load_output_rows()
    local stdout, err = catalog_stdout("topaz_output_profile_rows")
    if not stdout then
        return nil, err
    end

    local rows = {}
    for line in stdout:gmatch("[^\r\n]+") do
        local f = split_tsv(line)
        if #f >= 4 then
            local args = f[4] or ""
            local codec = args:match("%-c:v%s+(%S+)") or ""
            local rate = args:match("%-b:v%s+(%S+)")
            local blurb = (f[3] or "")
            if codec ~= "" then
                blurb = blurb .. " · " .. codec
            end
            if rate then
                blurb = blurb .. " · " .. rate
            end
            table.insert(rows, {
                display = f[1],
                slug = f[2] or "",
                output_ext = f[3] or "",
                video_args = args,
                blurb = blurb,
            })
        end
    end

    if #rows == 0 then
        return nil, "No Topaz output profiles found"
    end

    return rows
end

-- Advanced insight text for the details sheet, keyed by preset slug. Row kinds:
-- strategy (why the recipe works), watch (failure mode), vs (neighbour slug + when
-- to pick which). Failure is tolerated — the sheet just renders without prose.
local function load_preset_insights()
    local stdout = catalog_stdout("topaz_preset_insights")
    if not stdout then
        return {}
    end

    local by_slug = {}
    for line in stdout:gmatch("[^\r\n]+") do
        local f = split_tsv(line)
        local slug, kind = f[1], f[2]
        if slug and kind then
            local ins = by_slug[slug]
            if not ins then
                ins = { notes = {} }
                by_slug[slug] = ins
            end
            if kind == "strategy" then
                ins.strategy = f[3]
            elseif kind == "note" and f[3] then
                ins.notes[f[3]] = f[4]
            elseif kind == "watch" then
                ins.watch = f[3]
            elseif kind == "vs" then
                ins.vs_slug, ins.vs = f[3], f[4]
            end
        end
    end

    return by_slug
end

-- ===== rendering timer =====
-- While a preview renders we keep a periodic timer alive purely to re-draw the menu,
-- so the rendering row's throb animates and its elapsed-seconds readout ticks up.

local function start_render_timer()
    encode_start = mp.get_time()
    if encode_timer then
        encode_timer:kill()
    end
    encode_timer = mp.add_periodic_timer(0.2, function()
        draw_menu()
    end)
end

local function stop_render_timer()
    if encode_timer then
        encode_timer:kill()
        encode_timer = nil
    end
    encode_start = nil
end

-- ===== menu drawing =====

-- Paint one selectable option row (Interpolate / Output tabs) into the buckets.
local function paint_option_row(pills, fg, o)
    if o.selected then
        pills[#pills + 1] = selection_pill_event(o.y, o.rt, o.rb)
    elseif (o.is_cursor or o.is_hover) and not o.greyed then
        pills[#pills + 1] = row_pill(o.y, "&HFFFFFF&", "&HD2&", o.rt, o.rb)
    end

    local center = o.y + 15
    local cap, num = keycap_events(center, o.number, o.selected)
    fg[#fg + 1] = cap
    fg[#fg + 1] = num

    local lifted = o.selected or o.is_cursor or o.is_hover
    local tcolor = o.greyed and "&H6E6E6E&" or "&HFFFFFF&"
    local scolor = o.greyed and "&H565656&" or (lifted and "&HDCDCDC&" or "&HA6A6A6&")
    if o.blurb and o.blurb ~= "" then
        fg[#fg + 1] = string.format(
            "{\\an4\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs13\\b1\\1c%s}%s",
            TEXT_X, o.y + 10, FONT, tcolor, ass_escape(o.title))
        fg[#fg + 1] = string.format(
            "{\\an4\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs10\\b0\\1c%s}%s",
            TEXT_X, o.y + 22, FONT, scolor, ass_escape(o.blurb))
    else
        fg[#fg + 1] = string.format(
            "{\\an4\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs13\\b1\\1c%s}%s",
            TEXT_X, center, FONT, tcolor, ass_escape(o.title))
    end

    if o.selected then
        fg[#fg + 1] = string.format(
            "{\\an6\\pos(%d,%d)\\bord0\\shad0\\fs12\\1c&HFFFFFF&}✓",
            LIST_X + ROW_W - 12, center, FONT)
    end
end

-- The segmented Enhance / Interpolate / Output control under the title.
local function draw_tab_bar(ev)
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c&HFFFFFF&\\1a&HE8&\\p1}%s{\\p0}",
        LIST_X, TAB_Y, ass_round_rect(ROW_W, TAB_H, 9))

    menu.tab_hitboxes = {}
    local seg_w = math.floor((ROW_W - 6) / #TABS)
    for i, name in ipairs(TABS) do
        local x = LIST_X + 3 + (i - 1) * seg_w
        local cy = TAB_Y + math.floor(TAB_H / 2)
        if i == menu.tab then
            ev[#ev + 1] = string.format(
                "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c&HFFFFFF&\\1a&H1E&\\p1}%s{\\p0}",
                x, TAB_Y + 3, ass_round_rect(seg_w, TAB_H - 6, 7))
            ev[#ev + 1] = string.format(
                "{\\an5\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs11\\b1\\1c&H1A1A1A&}%s",
                x + math.floor(seg_w / 2), cy, FONT, name)
        else
            ev[#ev + 1] = string.format(
                "{\\an5\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs11\\b0\\1c&HB4B4B4&}%s",
                x + math.floor(seg_w / 2), cy, FONT, name)
        end
        menu.tab_hitboxes[#menu.tab_hitboxes + 1] =
            { x0 = x, x1 = x + seg_w, y0 = TAB_Y, y1 = TAB_Y + TAB_H, tab = i }
    end
end

-- Enhance tab body: grouped preset list with preview status. Fills the buckets and
-- hitboxes; returns the content-end y.
local function draw_enhance_body(cards, pills, fg, labels)
    local cursor_preset = menu.presets[menu.cursor]
    local y = LIST_TOP
    local grp_y0, grp_count = nil, 0
    local function flush_group()
        if grp_y0 and grp_count > 0 then
            for _, e in ipairs(group_card_events(grp_y0, grp_count)) do
                cards[#cards + 1] = e
            end
        end
        grp_y0, grp_count = nil, 0
    end

    for idx, item in ipairs(menu.items) do
        if item.kind == "header" then
            flush_group()
            labels[#labels + 1] = string.format(
                "{\\an4\\pos(%d,%d)\\bord0\\shad0\\fn%s\\b0\\fs10\\1c&H8C8C8C&}%s",
                LIST_X + 2, y + HH - 8, FONT, ass_escape(item.label:upper()))
            y = y + HH
        else
            local p = item.preset
            if not grp_y0 then
                grp_y0 = y
            end

            local is_cursor = (p == cursor_preset)
            local is_hover = (menu.hover == p.slug)
            local entry = (not p.is_original) and menu.cache.stills[key_for(p)] or nil
            local cached = entry ~= nil
            local is_shown = (p.is_original and menu.shown_slug == nil)
                or (menu.shown_slug == p.slug)
            local is_rendering = (menu.rendering_slug == p.slug)

            local center = y + 15
            table.insert(menu.hitboxes, { y0 = y, y1 = y + RH, preset = p })

            -- Corner radii so a full-width row highlight matches the group card's
            -- rounded ends but stays square where rows meet.
            local nxt = menu.items[idx + 1]
            local rt = (grp_count == 0) and 12 or 0
            local rb = (not nxt or nxt.kind == "header") and 12 or 0

            -- Row background priority: a rendering row throbs white (obvious work in
            -- progress), the shown render is a solid blue pill, and mere keyboard /
            -- mouse focus is a light static white wash. Only the shown row goes blue.
            if is_rendering then
                local t = (mp.get_time() % 1.1) / 1.1
                local tri = t < 0.5 and t * 2 or 2 - t * 2
                pills[#pills + 1] = row_pill(y, "&HFFFFFF&",
                    string.format("&H%02X&", 0xD0 - math.floor(tri * 0x48)), rt, rb)
            elseif is_shown then
                pills[#pills + 1] = selection_pill_event(y, rt, rb)
            elseif is_cursor or is_hover then
                pills[#pills + 1] = row_pill(y, "&HFFFFFF&", "&HD2&", rt, rb)
            end

            local cap, num = keycap_events(center, p.number, is_shown or is_rendering)
            fg[#fg + 1] = cap
            fg[#fg + 1] = num

            -- Two-tone title: bold model name, then a smaller bold variant name.
            local model, variant = split_display(p.display)
            local lifted = is_cursor or is_hover or is_shown or is_rendering
            local vcolor = lifted and "&HE8E8E8&" or "&H8F8F8F&"
            local vtail = variant
                and string.format("  {\\b1\\fs11\\1c%s}%s", vcolor, ass_escape(variant)) or ""
            fg[#fg + 1] = string.format(
                "{\\an4\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs13\\b1\\1c&HFFFFFF&}%s%s",
                TEXT_X, y + 10, FONT, ass_escape(model), vtail)
            local subcolor = lifted and "&HDCDCDC&" or "&HA6A6A6&"
            fg[#fg + 1] = string.format(
                "{\\an4\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs10\\b0\\1c%s}%s",
                TEXT_X, y + 22, FONT, subcolor, ass_escape(p.blurb))

            -- Right status slot, vertically centred: while rendering, the elapsed time
            -- ticks up here; otherwise an eye marks the shown render (a check marks any
            -- other cached preset), with the still's render time just left of it.
            local rx = LIST_X + ROW_W - 12
            if is_rendering then
                local elapsed = encode_start and (mp.get_time() - encode_start) or 0
                fg[#fg + 1] = string.format(
                    "{\\an6\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs11\\b1\\1c&HFFFFFF&}%.0fs",
                    rx, center, FONT, elapsed)
            else
                local time_x = rx
                if is_shown then
                    local lens, pupil = eye_events(rx - 6, center, "&HFFFFFF&")
                    fg[#fg + 1] = lens
                    fg[#fg + 1] = pupil
                    time_x = rx - 15
                elseif cached then
                    fg[#fg + 1] = string.format(
                        "{\\an6\\pos(%d,%d)\\bord0\\shad0\\fs12\\1c%s}✓",
                        rx, center, GREEN)
                    time_x = rx - 15
                end
                if entry and entry.secs then
                    fg[#fg + 1] = string.format(
                        "{\\an6\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs10\\1c%s}%s",
                        time_x, center, FONT,
                        lifted and "&HD9D9D9&" or "&H8A8A8A&",
                        short_secs(entry.secs))
                end
            end

            y = y + RH
            grp_count = grp_count + 1
        end
    end
    flush_group()
    return y
end

-- Interpolate tab body: one grouped card of frame-rate options (radio-style).
local function draw_interp_body(cards, pills, fg, labels)
    local rows = menu.interp_rows
    local y = LIST_TOP

    labels[#labels + 1] = string.format(
        "{\\an4\\pos(%d,%d)\\bord0\\shad0\\fn%s\\b0\\fs10\\1c&H8C8C8C&}FRAME RATE",
        LIST_X + 2, y + HH - 8, FONT)
    y = y + HH

    local grp_y0 = y
    for i, row in ipairs(rows) do
        table.insert(menu.hitboxes, { y0 = y, y1 = y + RH, index = i })
        paint_option_row(pills, fg, {
            y = y,
            rt = (i == 1) and 12 or 0,
            rb = (i == #rows) and 12 or 0,
            number = i,
            title = row.title,
            blurb = row.blurb,
            selected = (i == menu.interp_sel),
            is_cursor = (i == menu.interp_cursor),
            is_hover = (menu.hover == i),
        })
        y = y + RH
    end
    for _, e in ipairs(group_card_events(grp_y0, #rows)) do
        cards[#cards + 1] = e
    end
    return y
end

-- Output tab body: resolution group (gated by the chosen enhancement) + format group.
local function draw_output_body(cards, pills, fg, labels)
    local enh = chosen_enh()
    local res_opts = menu.res_options
    local formats = menu.out_formats or {}
    local y = LIST_TOP

    -- Resolution section. Rows the chosen enhancement can't reach are greyed and
    -- the blue pill sits on the resolution actually in effect.
    local eff = effective_res(enh)
    local limited = false
    for _, opt in ipairs(res_opts) do
        if not res_available(enh, opt, menu.two_x_is_4k) then
            limited = true
            break
        end
    end
    local res_label = "RESOLUTION"
    if limited then
        local name = enh.is_original and "ORIGINAL" or (split_display(enh.display)):upper()
        res_label = res_label .. "  ·  LIMITED BY " .. name
    end
    labels[#labels + 1] = string.format(
        "{\\an4\\pos(%d,%d)\\bord0\\shad0\\fn%s\\b0\\fs10\\1c&H8C8C8C&}%s",
        LIST_X + 2, y + HH - 8, FONT, ass_escape(res_label))
    y = y + HH

    local grp_y0 = y
    for i, opt in ipairs(res_opts) do
        table.insert(menu.hitboxes, { y0 = y, y1 = y + RH, index = i })
        paint_option_row(pills, fg, {
            y = y,
            rt = (i == 1) and 12 or 0,
            rb = (i == #res_opts) and 12 or 0,
            number = i,
            title = opt.label,
            blurb = opt.blurb,
            selected = (opt == eff),
            is_cursor = (i == menu.out_cursor),
            is_hover = (menu.hover == i),
            greyed = not res_available(enh, opt, menu.two_x_is_4k),
        })
        y = y + RH
    end
    for _, e in ipairs(group_card_events(grp_y0, #res_opts)) do
        cards[#cards + 1] = e
    end
    y = y + 8

    -- Format section.
    labels[#labels + 1] = string.format(
        "{\\an4\\pos(%d,%d)\\bord0\\shad0\\fn%s\\b0\\fs10\\1c&H8C8C8C&}FORMAT",
        LIST_X + 2, y + HH - 8, FONT)
    y = y + HH

    grp_y0 = y
    local k = #res_opts
    for i, fmt in ipairs(formats) do
        local n = k + i
        table.insert(menu.hitboxes, { y0 = y, y1 = y + RH, index = n })
        paint_option_row(pills, fg, {
            y = y,
            rt = (i == 1) and 12 or 0,
            rb = (i == #formats) and 12 or 0,
            number = n,
            title = fmt.display,
            blurb = fmt.blurb,
            selected = (i == menu.fmt_sel),
            is_cursor = (n == menu.out_cursor),
            is_hover = (menu.hover == n),
        })
        y = y + RH
    end
    if #formats > 0 then
        for _, e in ipairs(group_card_events(grp_y0, #formats)) do
            cards[#cards + 1] = e
        end
    end
    return y
end

function draw_menu()
    if not menu or menu.ui_hidden then
        list_badge:remove()
        params_badge:remove()
        detail_badge:remove()
        return
    end

    list_badge.res_x = 1280
    list_badge.res_y = 720

    -- Compact caption: resolution · fps · clip length · preview time.
    local prof = menu.profile
    local meta
    if prof and prof.width then
        local fps = prof.fps and string.format(" · %.0ffps", prof.fps) or ""
        local len = menu.duration and (" · " .. format_clock(menu.duration)) or ""
        meta = string.format("%d×%d%s%s · @ %s",
            prof.width, prof.height, fps, len, format_time(menu.time_pos))
    else
        meta = "@ " .. format_time(menu.time_pos)
    end

    -- Per-tab body into z-layer buckets (cards behind pills behind text).
    local cards, pills, fg, labels = {}, {}, {}, {}
    menu.hitboxes = {}
    local content_end
    if menu.tab == 1 then
        content_end = draw_enhance_body(cards, pills, fg, labels)
    elseif menu.tab == 2 then
        content_end = draw_interp_body(cards, pills, fg, labels)
    else
        content_end = draw_output_body(cards, pills, fg, labels)
    end

    local panel_bottom = math.min(content_end + 10, 712)

    -- Sheet backing + header (title left, source caption right) + tab bar + body.
    local ev = {}
    ev[#ev + 1] = panel_bg_event(MARGIN_T, panel_bottom)
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(%d,%d)\\bord0\\shad0\\fn%s\\b1\\fs18\\1c&HFFFFFF&}Topaz",
        LIST_X, MARGIN_T + 10, FONT)
    ev[#ev + 1] = string.format(
        "{\\an6\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs10\\1c&H9A9A9A&}%s",
        LIST_X + ROW_W, MARGIN_T + 21, FONT, ass_escape(meta))
    draw_tab_bar(ev)

    for _, e in ipairs(cards) do ev[#ev + 1] = e end
    for _, e in ipairs(pills) do ev[#ev + 1] = e end
    for _, e in ipairs(fg) do ev[#ev + 1] = e end
    for _, e in ipairs(labels) do ev[#ev + 1] = e end

    list_badge.data = table.concat(ev, "\n")
    list_badge:update()

    -- Bottom bar: per-tab shortcuts on the left, the finalize actions far right.
    draw_bottom({
        hints = menu.tab == 1 and ENHANCE_HINTS or OPTION_HINTS,
        right_hints = FINALIZE_HINTS,
    })

    draw_details()
end

-- ===== preset details companion sheet (`d`, Enhance tab) =====
-- A right-docked sheet in the same visual language as the menu. Follows the
-- cursor preset live: decoded tvai_up sliders as bars, plus the catalog's
-- insight prose (strategy / failure mode / vs-neighbour).

local DETAIL_W = 340
local DETAIL_X = 1280 - MARGIN_L - DETAIL_W
local DETAIL_PAD = 14
local DET_TX = DETAIL_X + DETAIL_PAD       -- content left edge
local DET_TW = DETAIL_W - 2 * DETAIL_PAD   -- content width

local AMBER = "&H40B3FF&"

-- Friendly names for the model codes appearing in enhancement filters.
local MODEL_NAMES = {
    ["prob-4"] = "Proteus v4", ["slm-1"] = "Starlight Mini",
    ["ghq-5"] = "Gaia HQ", ["iris-3"] = "Iris v3",
    ["nyx-3"] = "Nyx v3", ["thd-3"] = "Theia Detail",
}

-- Bar display order, grouped like the Topaz UI. `axis` rescales the bar for
-- sliders whose usable range is much smaller than 0-1 (grain), so light grain
-- still reads as a bar; the printed value stays the raw number.
local PARAM_GROUPS = {
    { label = "DETAIL & STRUCTURE", params = {
        { key = "details", label = "Recover detail" },
        { key = "blur", label = "Sharpen" },
        { key = "preblur", label = "Anti-alias / deblur" },
    } },
    { label = "CLEANUP", params = {
        { key = "compression", label = "Revert compression" },
        { key = "noise", label = "Reduce noise" },
        { key = "halo", label = "Dehalo" },
    } },
    { label = "TEXTURE", params = {
        { key = "grain", label = "Add grain", axis = 0.25 },
    } },
}

-- key=value map of the first tvai_up segment of a filter (nil if none).
local function parse_tvai_params(filter)
    local seg = (filter or ""):match("tvai_up=([^,]*)")
    if not seg then
        return nil
    end
    local params = {}
    for k, v in seg:gmatch("([%w_]+)=([^:]*)") do
        params[k] = v
    end
    return params
end

-- Greedy word wrap to a display-character budget (ASS has no width-bounded
-- wrapping for positioned events, so prose is broken into lines here).
local function wrap_text(text, max_chars)
    local lines, line = {}, ""
    for word in tostring(text or ""):gmatch("%S+") do
        local cand = (line == "") and word or (line .. " " .. word)
        if line ~= "" and disp_len(cand) > max_chars then
            lines[#lines + 1] = line
            line = word
        else
            line = cand
        end
    end
    if line ~= "" then
        lines[#lines + 1] = line
    end
    return lines
end

function draw_details()
    if not menu or menu.ui_hidden or menu.tab ~= 1 or not details_visible then
        detail_badge:remove()
        if menu then
            menu.det_close_hb = nil
        end
        return
    end

    local p = menu.presets[menu.cursor]
    if not p then
        detail_badge:remove()
        return
    end

    detail_badge.res_x = 1280
    detail_badge.res_y = 720

    local ins = (menu.insights or {})[p.slug] or { notes = {} }
    local params = p.is_original and nil or parse_tvai_params(p.filter_body)
    local has_sliders = params ~= nil and params.details ~= nil

    local ev = {}
    local y = MARGIN_T + DETAIL_PAD

    -- Emit `text` word-wrapped at (x, y..), advancing y one line-height per line.
    local function put_lines(text, x, width, fs, color, lh)
        local max_chars = math.floor(width / (fs * 0.55))
        for _, line in ipairs(wrap_text(text, max_chars)) do
            ev[#ev + 1] = string.format(
                "{\\an7\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs%g\\b0\\1c%s}%s",
                x, y, FONT, fs, color, ass_escape(line))
            y = y + lh
        end
    end

    -- Title: two-tone model / variant, matching the menu rows.
    local model, variant = split_display(p.display)
    local vtail = variant
        and string.format("  {\\b1\\fs11\\1c&HC9C9C9&}%s", ass_escape(variant)) or ""
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs15\\b1\\1c&HFFFFFF&}%s%s",
        DET_TX, y, FONT, ass_escape(model), vtail)
    y = y + 19

    -- Mode line: model code plus slider semantics (the glossary layer).
    local mode_line
    if p.is_original then
        mode_line = "source frame — no processing"
    elseif not params then
        mode_line = "no tvai_up stage in this filter"
    else
        local mname = MODEL_NAMES[params.model] or params.model or "?"
        if not has_sliders then
            mode_line = mname .. " — runs on model defaults, no manual sliders"
        elseif (tonumber(params.estimate) or 0) > 0 then
            mode_line = mname .. " — relative: sliders offset a per-clip auto estimate"
        else
            mode_line = mname .. " — manual: sliders are absolute, not per-clip offsets"
        end
    end
    put_lines(mode_line, DET_TX, DET_TW, 9, "&H9A9A9A&", 12)
    y = y + 6

    -- Strategy callout on a tinted accent card.
    if ins.strategy then
        local fs, lh, pad = 10.5, 14, 10
        local lines = wrap_text(ins.strategy,
            math.floor((DET_TW - 2 * pad) / (fs * 0.55)))
        local card_h = #lines * lh + 2 * pad - 3
        ev[#ev + 1] = string.format(
            "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c%s\\1a&HD4&\\p1}%s{\\p0}",
            DET_TX, y, ACCENT, ass_round_rect(DET_TW, card_h, 9))
        local ty = y + pad - 1
        for _, line in ipairs(lines) do
            ev[#ev + 1] = string.format(
                "{\\an7\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs%g\\b0\\1c&HE0E0E0&}%s",
                DET_TX + pad, ty, FONT, fs, ass_escape(line))
            ty = ty + lh
        end
        y = y + card_h + 10
    end

    -- Parameter bars (manual Proteus rows only).
    if has_sliders then
        for _, group in ipairs(PARAM_GROUPS) do
            local rows = {}
            for _, prm in ipairs(group.params) do
                local v = tonumber(params[prm.key])
                if v then
                    rows[#rows + 1] = { def = prm, value = v }
                end
            end
            if #rows > 0 then
                ev[#ev + 1] = string.format(
                    "{\\an7\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs9\\b0\\1c&H8C8C8C&}%s",
                    DET_TX, y, FONT, group.label)
                y = y + 14
                for _, row in ipairs(rows) do
                    local prm, v = row.def, row.value
                    local suffix = ""
                    if prm.key == "grain" and params.gsize then
                        suffix = string.format("  {\\b0\\fs9\\1c&H8A8A8A&}gsize %s",
                            ass_escape(params.gsize))
                    end
                    ev[#ev + 1] = string.format(
                        "{\\an7\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs11\\b1\\1c&HFFFFFF&}%s%s",
                        DET_TX, y, FONT, ass_escape(prm.label), suffix)
                    ev[#ev + 1] = string.format(
                        "{\\an9\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs10\\b1\\1c&HE6E6E6&}%.2f",
                        DET_TX + DET_TW, y + 1, FONT, v)
                    y = y + 14

                    -- Track, then fill: left-anchored for positive values,
                    -- centre-anchored amber for negative (relative-mode) ones.
                    ev[#ev + 1] = string.format(
                        "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c&HFFFFFF&\\1a&HE2&\\p1}%s{\\p0}",
                        DET_TX, y, ass_round_rect(DET_TW, 4, 2))
                    local frac = math.max(-1, math.min(1, v / (prm.axis or 1)))
                    if frac > 0 then
                        local w = math.max(3, math.floor(DET_TW * frac + 0.5))
                        ev[#ev + 1] = string.format(
                            "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c%s\\p1}%s{\\p0}",
                            DET_TX, y, ACCENT, ass_round_rect(w, 4, 2))
                    elseif frac < 0 then
                        local mid = DET_TX + math.floor(DET_TW / 2)
                        local w = math.max(3, math.floor(DET_TW / 2 * -frac + 0.5))
                        ev[#ev + 1] = string.format(
                            "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c%s\\p1}%s{\\p0}",
                            mid - w, y, AMBER, ass_round_rect(w, 4, 2))
                        ev[#ev + 1] = string.format(
                            "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c&HFFFFFF&\\1a&H90&\\p1}%s{\\p0}",
                            mid, y - 2, ass_rect(1, 8))
                    end
                    y = y + 9

                    local note = ins.notes and ins.notes[prm.key]
                    if note then
                        put_lines(note, DET_TX, DET_TW, 9.5, "&H9D9D9D&", 12)
                        y = y + 2
                    end
                    y = y + 5
                end
                y = y + 4
            end
        end
    end

    -- Footer: failure mode + nearest-neighbour pick, under a hairline.
    if ins.watch or ins.vs then
        ev[#ev + 1] = string.format(
            "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c&HFFFFFF&\\1a&HE6&\\p1}%s{\\p0}",
            DET_TX, y, ass_rect(DET_TW, 1))
        y = y + 9
        local label_w = 58
        local function footer(label, color, text)
            ev[#ev + 1] = string.format(
                "{\\an7\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs9\\b1\\1c%s}%s",
                DET_TX, y, FONT, color, label)
            put_lines(text, DET_TX + label_w, DET_TW - label_w, 9.5, "&H9D9D9D&", 12)
            y = y + 3
        end
        if ins.watch then
            footer("WATCH FOR", AMBER, ins.watch)
        end
        if ins.vs then
            -- Resolve the neighbour's current menu number so "VS 3" is
            -- actionable; falls back to a bare "VS" when it is hidden.
            local label = "VS"
            if ins.vs_slug then
                for _, other in ipairs(menu.presets) do
                    if other.slug == ins.vs_slug then
                        label = "VS " .. tostring(other.number)
                        break
                    end
                end
            end
            footer(label, ACCENT, ins.vs)
        end
    end

    local bottom = math.min(y + DETAIL_PAD - 4, 712)
    table.insert(ev, 1, panel_bg_event(MARGIN_T, bottom, DETAIL_X, DETAIL_W))

    -- Close button (×) in the sheet's top-right corner; `d` toggles too.
    local bx = DETAIL_X + DETAIL_W - DETAIL_PAD - 17
    local by = MARGIN_T + DETAIL_PAD - 2
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c&HFFFFFF&\\1a&HDA&\\p1}%s{\\p0}",
        bx, by, ass_round_rect(17, 17, 8.5))
    ev[#ev + 1] = string.format(
        "{\\an5\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs11\\b1\\1c&HFFFFFF&}×",
        bx + 8, by + 8, FONT)
    menu.det_close_hb = { x0 = bx - 5, y0 = by - 5, x1 = bx + 22, y1 = by + 22 }

    detail_badge.data = table.concat(ev, "\n")
    detail_badge:update()
end

function toggle_details()
    if not menu then
        return
    end
    details_visible = not details_visible
    draw_details()
end

-- ===== frame display / playlist management =====

-- Show the original frame: a cached original still if we have one, else the source
-- video paused at the captured timestamp.
function show_original()
    if not menu then
        return
    end

    menu.shown_slug = nil
    menu.shown_key = nil

    if menu.cache.original then
        if not menu.original_plindex then
            mp.commandv("loadfile", menu.cache.original, "append")
            menu.original_plindex = mp.get_property_number("playlist-count", 1) - 1
            table.insert(menu.appended, menu.original_plindex)
            mp.add_timeout(0.03, function()
                if menu and menu.shown_slug == nil and menu.original_plindex then
                    mp.commandv("playlist-play-index", tostring(menu.original_plindex))
                end
            end)
        else
            mp.commandv("playlist-play-index", tostring(menu.original_plindex))
        end
    elseif menu.original_pos then
        mp.commandv("playlist-play-index", tostring(menu.original_pos))
        pending_restore = { time_pos = menu.time_pos, pause = true }
    end
end

function render_or_show(preset)
    if not menu or not preset then
        return
    end

    -- The "original" pseudo-preset has nothing to render — just show the source.
    if preset.is_original then
        menu.cursor = preset.index
        menu.last_render_key = nil
        show_original()
        draw_menu()
        return
    end

    local res = effective_res(preset)
    local key = preset.slug .. "|" .. res.key
    local entry = menu.cache.stills[key]
    if entry then
        menu.shown_slug, menu.shown_key = preset.slug, key
        menu.last_render_key = key
        mp.commandv("playlist-play-index", tostring(entry.plindex))
        draw_menu()
        return
    end

    if menu.rendering_slug then
        mp.osd_message("Already rendering — please wait…", 1.5)
        return
    end

    menu.rendering_slug = preset.slug
    show_original()
    draw_menu()
    start_render_timer()

    local body, tail = enh_filter_for(preset, res)
    local parts = {}
    if body ~= "" then parts[#parts + 1] = body end
    if tail ~= "" then parts[#parts + 1] = tail end
    local preview_filter = table.concat(parts, ",")

    local render_started = mp.get_time()
    local time_arg = string.format("%.3f", menu.time_pos)
    mp.command_native_async({
        name = "subprocess",
        args = {
            topaz_preview_frame,
            "--input", menu.source,
            "--preset-name", preset.display .. " " .. res.label,
            "--preset-flag", "--filter_complex",
            "--filter", preview_filter,
            "--time", time_arg,
            "--no-open",
            "--print-paths",
        },
        playback_only = false,
        capture_stdout = true,
        capture_stderr = true,
    }, function(success, result, error)
        stop_render_timer()
        if not menu then
            return
        end
        menu.rendering_slug = nil

        if not success or not result or result.status ~= 0 then
            local stderr = result and trim(result.stderr) or ""
            local detail = stderr ~= "" and stderr or tostring(error or "unknown error")
            mp.osd_message("Topaz preview failed", 3)
            mp.msg.error("Topaz preview failed: " .. detail)
            draw_menu()
            return
        end

        local paths = parse_preview_paths(result.stdout)
        local topaz_image = paths.topaz or paths.compare
        if not topaz_image or topaz_image == "" then
            mp.osd_message("Topaz preview returned no image", 3)
            mp.msg.error("Topaz preview stdout:\n" .. (result.stdout or ""))
            draw_menu()
            return
        end

        if (not menu.cache.original) and paths.original and paths.original ~= "" then
            menu.cache.original = paths.original
        end

        mp.commandv("loadfile", topaz_image, "append")
        local plindex = mp.get_property_number("playlist-count", 1) - 1
        table.insert(menu.appended, plindex)
        menu.cache.stills[key] = {
            image = topaz_image,
            plindex = plindex,
            secs = mp.get_time() - render_started,
        }
        menu.shown_slug, menu.shown_key = preset.slug, key
        menu.last_render_key = key

        mp.add_timeout(0.03, function()
            if menu and menu.shown_key == key then
                mp.commandv("playlist-play-index", tostring(plindex))
            end
            draw_menu()
        end)
    end)
end

-- ===== navigation / key actions =====

-- Mouse position mapped into the 1280x720 virtual overlay space (nil if unknown).
local function mouse_virtual_pos()
    local pos = mp.get_property_native("mouse-pos")
    local ow = mp.get_property_number("osd-width")
    local oh = mp.get_property_number("osd-height")
    if not (pos and pos.x and ow and oh and ow > 0 and oh > 0) then
        return nil
    end
    return pos.x * 1280 / ow, pos.y * 720 / oh
end

-- Mouse-move observer: tracks which row the pointer is over so it can get a subtle
-- focus wash, and redraws only when the hovered row changes. No-op when closed.
local function update_hover()
    if not menu or menu.ui_hidden then
        return
    end

    local mx, my = mouse_virtual_pos()
    local hit = nil
    if mx and mx >= LIST_X and mx <= LIST_X + ROW_W then
        for _, hb in ipairs(menu.hitboxes or {}) do
            if my >= hb.y0 and my < hb.y1 then
                hit = hb.preset and hb.preset.slug or hb.index
                break
            end
        end
    end

    if menu.hover ~= hit then
        menu.hover = hit
        draw_menu()
    end
end

function set_tab(i)
    if not menu or menu.tab == i then
        return
    end
    menu.tab = i
    menu.hover = nil
    draw_menu()
end

function interp_select(i)
    if not menu or not menu.interp_rows[i] then
        return
    end
    menu.interp_sel = i
    menu.interp_cursor = i
    draw_menu()
end

-- After a resolution change, prefer the shown preset's still at the new resolution
-- if it is already cached; otherwise fall back to the original frame.
local function reshow_for_res()
    if not menu.shown_slug then
        draw_menu()
        return
    end
    local sp = chosen_enh()
    local key = (not sp.is_original) and key_for(sp) or nil
    local entry = key and menu.cache.stills[key]
    if entry then
        menu.shown_key = key
        menu.last_render_key = key
        mp.commandv("playlist-play-index", tostring(entry.plindex))
        draw_menu()
    else
        show_original()
        draw_menu()
    end
end

-- Select the i-th row of the Output tab (resolutions first, then formats).
function output_select(i)
    if not menu then
        return
    end
    local k = #menu.res_options
    if i <= k then
        local opt = menu.res_options[i]
        if not opt then
            return
        end
        if not res_available(chosen_enh(), opt, menu.two_x_is_4k) then
            mp.osd_message("Resolution fixed by the chosen enhancement", 1.5)
            return
        end
        menu.res_sel = i
        menu.out_cursor = i
        reshow_for_res()
    else
        local f = i - k
        if menu.out_formats and menu.out_formats[f] then
            menu.fmt_sel = f
            menu.out_cursor = i
            draw_menu()
        end
    end
end

-- Click dispatch: tab bar, details close button, then the active tab's rows.
local function menu_click()
    if not menu or menu.ui_hidden then
        return
    end
    local mx, my = mouse_virtual_pos()
    if not mx then
        return
    end

    for _, hb in ipairs(menu.tab_hitboxes or {}) do
        if mx >= hb.x0 and mx < hb.x1 and my >= hb.y0 and my < hb.y1 then
            set_tab(hb.tab)
            return
        end
    end

    local dhb = menu.det_close_hb
    if details_visible and dhb and mx >= dhb.x0 and mx <= dhb.x1
        and my >= dhb.y0 and my <= dhb.y1 then
        details_visible = false
        draw_details()
        return
    end

    if mx < LIST_X or mx > LIST_X + ROW_W then
        return
    end
    for _, hb in ipairs(menu.hitboxes or {}) do
        if my >= hb.y0 and my < hb.y1 then
            if menu.tab == 1 then
                menu.cursor = hb.preset.index
                render_or_show(hb.preset)
            elseif menu.tab == 2 then
                interp_select(hb.index)
            else
                output_select(hb.index)
            end
            return
        end
    end
end

function move_cursor(delta)
    if not menu then
        return
    end

    if menu.tab == 2 then
        local n = #menu.interp_rows
        if n > 0 then
            menu.interp_cursor = ((menu.interp_cursor - 1 + delta) % n) + 1
            draw_menu()
        end
        return
    elseif menu.tab == 3 then
        local n = #menu.res_options + #(menu.out_formats or {})
        if n > 0 then
            menu.out_cursor = ((menu.out_cursor - 1 + delta) % n) + 1
            draw_menu()
        end
        return
    end

    local n = #menu.presets
    if n == 0 then
        return
    end
    menu.cursor = ((menu.cursor - 1 + delta) % n) + 1

    -- Free hover-preview: original reverts to source; cached presets switch instantly.
    local preset = menu.presets[menu.cursor]
    if preset and not menu.rendering_slug then
        if preset.is_original then
            if menu.shown_slug ~= nil then
                show_original()
            end
        else
            local key = key_for(preset)
            local entry = menu.cache.stills[key]
            if entry and menu.shown_key ~= key then
                menu.shown_slug, menu.shown_key = preset.slug, key
                menu.last_render_key = key
                mp.commandv("playlist-play-index", tostring(entry.plindex))
            end
        end
    end

    draw_menu()
end

function select_number(num)
    if not menu then
        return
    end
    if menu.tab == 2 then
        if num >= 1 then
            interp_select(num)
        end
        return
    elseif menu.tab == 3 then
        if num >= 1 then
            output_select(num)
        end
        return
    end
    local preset = menu.by_number and menu.by_number[num]
    if not preset then
        return
    end
    menu.cursor = preset.index
    render_or_show(preset)
end

function render_cursor()
    if not menu then
        return
    end
    if menu.tab == 2 then
        interp_select(menu.interp_cursor)
        return
    elseif menu.tab == 3 then
        output_select(menu.out_cursor)
        return
    end
    render_or_show(menu.presets[menu.cursor])
end

-- The render currently being compared: the cursor preset if it is cached, else the
-- most recently displayed render.
local function current_render_key()
    if not menu then
        return nil
    end
    local cp = menu.presets[menu.cursor]
    if menu.tab == 1 and cp and not cp.is_original then
        local key = key_for(cp)
        if menu.cache.stills[key] then
            return key
        end
    end
    if menu.last_render_key and menu.cache.stills[menu.last_render_key] then
        return menu.last_render_key
    end
    return nil
end

-- Show the active render (l / RIGHT). No-op if nothing is rendered yet.
function show_render()
    if not menu then
        return
    end
    local key = current_render_key()
    if not key then
        return
    end
    menu.shown_key = key
    menu.shown_slug = key:match("^(.-)|")
    menu.last_render_key = key
    mp.commandv("playlist-play-index", tostring(menu.cache.stills[key].plindex))
    draw_menu()
end

-- Show the original (h / LEFT). No-op if the original is already showing.
function show_orig()
    if not menu or not menu.shown_slug then
        return
    end
    show_original()
    draw_menu()
end

-- Space: flip between the active render and the original.
function space_toggle()
    if not menu then
        return
    end
    if menu.shown_slug then
        show_original()
        draw_menu()
    else
        show_render()
    end
end

-- F: hide/show the whole menu UI for a clean fullscreen preview. Navigation and
-- A/B keys still work while hidden, so you can flick presets at full size.
function toggle_ui()
    if not menu then
        return
    end
    menu.ui_hidden = not menu.ui_hidden
    if menu.ui_hidden then
        list_badge:remove()
        params_badge:remove()
        detail_badge:remove()
    else
        draw_menu()
    end
end

-- ===== finalize: encode now / save job / copy command =====

-- Everything needed to run the encode outside mpv, built from the current tab
-- selections: the topaz-encode argv, a human preset name, and the source
-- location. Returns nil + message when the catalog gave us nothing to encode with.
local function build_encode_plan()
    if not menu.out_formats or #menu.out_formats == 0 then
        return nil, "No output formats available (catalog error)"
    end

    local enh = chosen_enh()
    local res = effective_res(enh)
    local interp = menu.interp_rows[menu.interp_sel]
    local output = menu.out_formats[menu.fmt_sel]

    local body, tail = enh_filter_for(enh, res)
    local parts = {}
    if body ~= "" then parts[#parts + 1] = body end
    if interp and interp.fi and interp.fi ~= "" then parts[#parts + 1] = interp.fi end
    if tail ~= "" then parts[#parts + 1] = tail end
    local final_filter = table.concat(parts, ",")
    if final_filter == "" then
        -- Original + no interpolation + source size: re-encode only.
        final_filter = "null"
    end

    local name_parts = { enh.is_original and "Original" or enh.display }
    if res.key ~= "orig" then
        name_parts[#name_parts + 1] = res.label
    end
    if interp and interp.fi then
        name_parts[#name_parts + 1] = interp.title
    end
    name_parts[#name_parts + 1] = output.display
    local preset_name = table.concat(name_parts, " + ")

    local metadata = enh.metadata
    if not metadata or metadata == "" then
        metadata = "videoai=" .. enh.display
    end

    local source = menu.source
    return {
        source = source,
        directory = utils.split_path(source),
        preset_name = preset_name,
        argv = {
            topaz_run,
            "--preset_name", preset_name,
            "--filter_complex", final_filter,
            "--output_ext", output.output_ext,
            "--video_args", output.video_args,
            "--metadata", metadata,
            "--",
            source,
        },
    }
end

-- The plan's argv as one copy-pasteable zsh command line.
local function shell_command(argv)
    local quoted = {}
    for _, a in ipairs(argv) do
        quoted[#quoted + 1] = shell_quote(a)
    end
    return table.concat(quoted, " ")
end

-- ⌘↩ (also c): launch the encode in a kitty tab and close the menu.
function start_encode_now()
    if not menu then
        return
    end
    if menu.rendering_slug then
        mp.osd_message("Wait for the preview render to finish", 1.5)
        return
    end
    local plan, err = build_encode_plan()
    if not plan then
        mp.osd_message(err, 2)
        return
    end

    local args = {
        kitty_launch,
        "--tab",
        "--hold",
        "--cwd", plan.directory,
        "--title", " topaz encode ",
        "--",
    }
    for _, a in ipairs(plan.argv) do
        args[#args + 1] = a
    end

    local result = utils.subprocess_detached({ args = args })
    if result == false then
        mp.osd_message("Topaz encode launch failed", 2)
        mp.msg.error("Failed to launch Topaz encode for: " .. plan.source)
        return
    end

    close_menu("Topaz encode started")
end

-- ⌘S: write a runnable .job script (the encode command) next to the video and
-- close the menu. Run it later with `zsh <file>.job` or straight from a shell.
function save_job_file()
    if not menu then
        return
    end
    local plan, err = build_encode_plan()
    if not plan then
        mp.osd_message(err, 2)
        return
    end

    local job_path = plan.source .. ".job"
    local f, ferr = io.open(job_path, "w")
    if not f then
        mp.osd_message("Could not write job file", 2)
        mp.msg.error("Job write failed: " .. tostring(ferr))
        return
    end
    f:write("#!/bin/zsh\n")
    f:write("# Topaz encode job — generated by the mpv Topaz workflow\n")
    f:write("# " .. plan.preset_name .. "\n")
    f:write("cd " .. shell_quote(plan.directory) .. "\n")
    f:write("exec " .. shell_command(plan.argv) .. "\n")
    f:close()
    utils.subprocess({ args = { "chmod", "+x", job_path }, cancellable = false })

    close_menu("Job saved: " .. basename(job_path))
end

-- ⌘C: copy the encode command line to the clipboard and close the menu.
function copy_encode_command()
    if not menu then
        return
    end
    local plan, err = build_encode_plan()
    if not plan then
        mp.osd_message(err, 2)
        return
    end

    local result = mp.command_native({
        name = "subprocess",
        args = { "pbcopy" },
        stdin_data = shell_command(plan.argv),
        playback_only = false,
    })
    if not result or result.status ~= 0 then
        mp.osd_message("Clipboard copy failed", 2)
        return
    end

    close_menu("Encode command copied")
end

-- Skip the preview point ±`delta` seconds without leaving the renderer. Every
-- cached still was rendered for the old timestamp, so they are dropped and the
-- source is re-shown at the new time; presets re-render on demand as before.
function menu_seek(delta)
    if not menu or menu.rendering_slug then
        return
    end

    local duration = mp.get_property_number("duration")
    local newt = menu.time_pos + delta
    if newt < 0 then
        newt = 0
    end
    if duration and newt > duration - 0.05 then
        newt = math.max(0, duration - 0.05)
    end
    menu.time_pos = newt

    -- Invalidate every cached preview (they belong to the previous timestamp).
    local stale = menu.appended
    menu.appended = {}
    menu.cache = { original = nil, stills = {} }
    menu.shown_slug = nil
    menu.shown_key = nil
    menu.original_plindex = nil
    menu.last_render_key = nil

    -- Return to the source video and seek to the new time.
    mp.commandv("playlist-play-index", tostring(menu.original_pos))
    pending_restore = { time_pos = newt, pause = true }
    mp.add_timeout(0.05, function()
        if menu then
            mp.commandv("seek", tostring(newt), "absolute", "exact")
            mp.set_property_bool("pause", true)
        end
    end)

    -- Drop the now-stale preview stills from the playlist (after the switch, so we
    -- never remove the item we just started playing). Appended stills always sit
    -- after original_pos, so their removal does not shift it.
    if #stale > 0 then
        mp.add_timeout(0.12, function()
            local count = mp.get_property_number("playlist-count", 0)
            table.sort(stale, function(a, b) return a > b end)
            for _, idx in ipairs(stale) do
                if idx >= 0 and idx ~= menu.original_pos and count > idx then
                    mp.commandv("playlist-remove", tostring(idx))
                end
            end
        end)
    end

    draw_menu()
end

-- ===== key bindings =====

local MENU_KEY_NAMES = {
    "topaz_menu_down", "topaz_menu_down2",
    "topaz_menu_up", "topaz_menu_up2",
    "topaz_menu_render_r", "topaz_menu_render_R",
    "topaz_menu_render_enter", "topaz_menu_render_kp_enter",
    "topaz_menu_space", "topaz_menu_tab", "topaz_menu_tab_back",
    "topaz_menu_hide_ui",
    "topaz_menu_seek_fwd", "topaz_menu_seek_back",
    "topaz_menu_details",
    "topaz_menu_show_render_l", "topaz_menu_show_render_right",
    "topaz_menu_show_orig_h", "topaz_menu_show_orig_left",
    "topaz_menu_proceed_c",
    "topaz_menu_save_job", "topaz_menu_encode_meta", "topaz_menu_copy_cmd",
    "topaz_menu_block_n", "topaz_menu_block_N",
    "topaz_menu_click",
    "topaz_menu_esc", "topaz_menu_bs",
}

function enable_menu_keys()
    mp.add_forced_key_binding("j", "topaz_menu_down", function() move_cursor(1) end)
    mp.add_forced_key_binding("DOWN", "topaz_menu_down2", function() move_cursor(1) end)
    mp.add_forced_key_binding("k", "topaz_menu_up", function() move_cursor(-1) end)
    mp.add_forced_key_binding("UP", "topaz_menu_up2", function() move_cursor(-1) end)
    for i = 0, 9 do
        mp.add_forced_key_binding(tostring(i), "topaz_menu_num_" .. i, function()
            select_number(i)
        end)
    end
    -- Render / select the cursor row.
    mp.add_forced_key_binding("r", "topaz_menu_render_r", render_cursor)
    mp.add_forced_key_binding("R", "topaz_menu_render_R", render_cursor)
    mp.add_forced_key_binding("ENTER", "topaz_menu_render_enter", render_cursor)
    mp.add_forced_key_binding("KP_ENTER", "topaz_menu_render_kp_enter", render_cursor)
    -- A/B: Space toggles render vs original; l/→ show render, h/← show original.
    mp.add_forced_key_binding("SPACE", "topaz_menu_space", space_toggle)
    mp.add_forced_key_binding("l", "topaz_menu_show_render_l", show_render)
    mp.add_forced_key_binding("RIGHT", "topaz_menu_show_render_right", show_render)
    mp.add_forced_key_binding("h", "topaz_menu_show_orig_h", show_orig)
    mp.add_forced_key_binding("LEFT", "topaz_menu_show_orig_left", show_orig)
    -- Tab / Shift+Tab cycle the Enhance / Interpolate / Output tabs.
    mp.add_forced_key_binding("TAB", "topaz_menu_tab", function()
        if menu then
            set_tab(menu.tab % #TABS + 1)
        end
    end)
    mp.add_forced_key_binding("Shift+TAB", "topaz_menu_tab_back", function()
        if menu then
            set_tab((menu.tab - 2) % #TABS + 1)
        end
    end)
    -- F hides/shows the UI for a clean fullscreen preview.
    mp.add_forced_key_binding("f", "topaz_menu_hide_ui", toggle_ui)
    -- d toggles the preset-details companion sheet.
    mp.add_forced_key_binding("d", "topaz_menu_details", toggle_details)
    -- Skip the preview point ±10s without leaving the renderer.
    mp.add_forced_key_binding("Shift+RIGHT", "topaz_menu_seek_fwd", function() menu_seek(10) end)
    mp.add_forced_key_binding("Shift+LEFT", "topaz_menu_seek_back", function() menu_seek(-10) end)
    -- Finalize: ⌘S saves a .job file, ⌘↩ launches the encode, ⌘C copies the
    -- command line. Each closes the menu. `c` stays as an encode alias.
    mp.add_forced_key_binding("Meta+s", "topaz_menu_save_job", save_job_file)
    mp.add_forced_key_binding("Meta+ENTER", "topaz_menu_encode_meta", start_encode_now)
    mp.add_forced_key_binding("Meta+c", "topaz_menu_copy_cmd", copy_encode_command)
    mp.add_forced_key_binding("c", "topaz_menu_proceed_c", start_encode_now)
    -- Swallow playlist-next/prev so preview stills don't get navigated away.
    mp.add_forced_key_binding("n", "topaz_menu_block_n", function() end)
    mp.add_forced_key_binding("N", "topaz_menu_block_N", function() end)
    -- Click: tab segments, details close, preset / option rows.
    mp.add_forced_key_binding("MBTN_LEFT", "topaz_menu_click", menu_click)
    -- Esc backs out of the option tabs, closes from the Enhance tab.
    mp.add_forced_key_binding("ESC", "topaz_menu_esc", function()
        if menu and menu.tab ~= 1 then
            set_tab(1)
        else
            close_menu("Topaz menu closed")
        end
    end)
    mp.add_forced_key_binding("BS", "topaz_menu_bs", function()
        if menu and menu.tab ~= 1 then
            set_tab(1)
        else
            close_menu("Topaz menu closed")
        end
    end)
end

function remove_menu_keys()
    for _, name in ipairs(MENU_KEY_NAMES) do
        mp.remove_key_binding(name)
    end
    for i = 0, 9 do
        mp.remove_key_binding("topaz_menu_num_" .. i)
    end
end

-- ===== open / close =====

local function open_menu(source, profile, data)
    local original_pos = mp.get_property_number("playlist-pos")

    menu = {
        source = source,
        source_name = basename(source),
        time_pos = mp.get_property_number("time-pos", 0) or 0,
        duration = mp.get_property_number("duration"),
        profile = profile,
        items = data.items,
        presets = data.presets,
        by_number = data.by_number,
        insights = data.insights or {},
        tab = 1,
        cursor = 1,
        hover = nil,
        res_options = data.res_options,
        res_sel = data.res_default,
        two_x_is_4k = data.two_x_is_4k,
        interp_rows = data.interp_rows,
        interp_sel = 1,
        interp_cursor = 1,
        out_formats = data.out_formats,
        fmt_sel = 1,
        out_cursor = 1,
        shown_slug = nil,
        shown_key = nil,
        rendering_slug = nil,
        last_render_key = nil,
        cache = { original = nil, stills = {} },
        original_pos = original_pos,
        original_plindex = nil,
        appended = {},
        old_image_display_duration = mp.get_property("image-display-duration"),
    }

    mp.set_property("image-display-duration", "inf")
    mp.set_property_bool("pause", true)
    -- Keep the built-in OSC (seek bar / top bar) from popping in on mouse-move while
    -- the Topaz UI owns the screen; restored on close.
    mp.commandv("script-message", "osc-visibility", "never", "no-osd")
    enable_menu_keys()
    draw_menu()
end

function close_menu(message)
    if not menu then
        return
    end

    local state = menu
    menu = nil
    remove_menu_keys()
    stop_render_timer()
    list_badge:remove()
    params_badge:remove()
    detail_badge:remove()
    -- Restore the built-in OSC we suppressed on open.
    mp.commandv("script-message", "osc-visibility", "auto", "no-osd")

    if state.old_image_display_duration then
        mp.set_property("image-display-duration", state.old_image_display_duration)
    end

    pending_restore = { time_pos = state.time_pos, pause = true }

    local count = mp.get_property_number("playlist-count", 0)
    if state.original_pos and count > state.original_pos then
        mp.commandv("playlist-play-index", tostring(state.original_pos))
        if #state.appended > 0 then
            mp.add_timeout(0.1, function()
                local current = mp.get_property_number("playlist-count", 0)
                table.sort(state.appended, function(a, b) return a > b end)
                for _, idx in ipairs(state.appended) do
                    if idx >= 0 and current > idx then
                        mp.commandv("playlist-remove", tostring(idx))
                    end
                end
            end)
        end
    else
        mp.commandv("loadfile", state.source, "replace")
    end

    if message then
        mp.osd_message(message, 2)
    end
end

-- ===== entry points =====

local function show_transform_menu()
    if menu then
        close_menu()
    end

    local source, err = check_current_file()
    if not source then
        mp.osd_message(err, 3)
        return
    end

    local profile = source_profile()
    local res_options, res_default, two_x_is_4k = build_res_options(profile)

    local data
    data, err = load_enhancement_presets(profile, res_options, two_x_is_4k)
    if not data then
        mp.osd_message(err, 3)
        return
    end

    data.res_options = res_options
    data.res_default = res_default
    data.two_x_is_4k = two_x_is_4k
    data.interp_rows = load_interp_rows()
    data.out_formats = load_output_rows()
    data.insights = load_preset_insights()

    open_menu(source, profile, data)
end

local function topaz_workflow_external_file()
    local abs, err = check_current_file()
    if not abs then
        mp.osd_message(err, 2)
        return
    end

    local directory = utils.split_path(abs)
    local result = utils.subprocess_detached({
        args = {
            kitty_launch,
            "--tab",
            "--hold",
            "--cwd", directory,
            "--title", " topaz workflow ",
            "--",
            topaz_workflow, abs,
        },
    })

    if result == false then
        mp.osd_message("Topaz workflow launch failed", 2)
        mp.msg.error("Failed to launch Topaz workflow for: " .. abs)
        return
    end

    mp.osd_message("Opening Topaz workflow", 1.5)
end

mp.register_event("file-loaded", function()
    if not pending_restore then
        return
    end

    local restore = pending_restore
    pending_restore = nil

    if restore.time_pos and restore.time_pos > 0 then
        mp.commandv("seek", restore.time_pos, "absolute", "exact")
    end

    if restore.pause ~= nil then
        mp.set_property_bool("pause", restore.pause)
    end
end)

mp.register_event("shutdown", function()
    remove_menu_keys()
    if encode_timer then
        encode_timer:kill()
        encode_timer = nil
    end
    list_badge:remove()
    params_badge:remove()
    detail_badge:remove()
end)

-- Row hover highlighting (no-op unless the menu is open).
mp.observe_property("mouse-pos", "native", update_hover)

mp.add_key_binding(nil, "topaz_workflow_current_file", show_transform_menu)
mp.add_key_binding(nil, "topaz_workflow_external_file", topaz_workflow_external_file)
