local utils = require "mp.utils"

local kitty_launch = "/Users/nom/.zsh/bin/kitty-launch"
local topaz_workflow = "/Users/nom/.zsh/bin/topaz-workflow"
local topaz_run = "/Users/nom/.zsh/bin/topaz-encode"
local topaz_preview_frame = "/Users/nom/.zsh/bin/topaz-preview-frame"
local preset_catalog = "/Users/nom/config/config/zsh/bin/lib/topaz-preset-catalog.zsh"

-- Active enhancement render-menu session (nil when closed). Persists across the
-- interpolation and output steps so appended preview images can be cleaned up at the end.
local menu = nil
-- Active simple picker (interpolation / output steps), nil when not open.
local picker = nil
local pending_restore = nil

local list_badge = mp.create_osd_overlay("ass-events")
list_badge.z = 30
local params_badge = mp.create_osd_overlay("ass-events")
params_badge.z = 30
local encode_timer = nil
local encode_start = nil

-- Categories shown in the enhancement menu, in display order. `upscale` rows are
-- hidden when the source is already >= 4K.
local CATEGORY_ORDER = {
    { key = "upscale-2x", label = "Upscale 2x", upscale = true },
    { key = "upscale-4k", label = "Upscale to 4K", upscale = true },
    { key = "repair", label = "Repair", upscale = false },
    { key = "sharpen", label = "Sharpen & Deblur", upscale = false },
}

-- Forward declarations (so the menu functions can reference each other).
local draw_menu, show_original, render_or_show, move_cursor, select_number
local render_cursor, choose_enhancement, choose_interpolation, choose_output
local start_encode, close_menu, enable_menu_keys, remove_menu_keys
local draw_picker, open_picker, remove_picker_keys
local space_toggle, show_render, show_orig, toggle_ui, menu_seek

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
-- "Proteus 2x — Sharp" -> "Proteus", "Sharp"; "Nyx Heavy Denoise" -> "Nyx",
-- "Heavy Denoise". Resolution tokens are dropped — the section header carries them.
local TWO_WORD_MODELS = { "Starlight Mini", "Gaia HQ", "Focus Fix" }

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

-- Floating "sheet" geometry (iOS grouped-list feel): the panel no longer spans the
-- full height flush to the edge — it sits inset from the top-left and grows only as
-- tall as its content.
local MARGIN_L = 14     -- gap from the left screen edge (so the sheet floats)
local MARGIN_T = 12     -- gap from the top screen edge
local PANEL_W = 340     -- floating panel width
local PAD = 14          -- panel edge -> content inset
local LIST_X = MARGIN_L + PAD        -- group / row left edge (also mouse hit-test left)
local ROW_W = PANEL_W - 2 * PAD      -- group + row width (also mouse hit-test right)
local RH = 30           -- preset / picker row height (two text lines)
local HH = 22           -- section-header row advance
local LIST_TOP = 56     -- y where the first group starts
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
-- for the blue "shown" highlight, the white rendering throb, and the cursor / hover focus.
local function row_pill(y, color, alpha, rt, rb)
    return string.format(
        "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c%s\\1a%s\\p1}%s{\\p0}",
        LIST_X, y, color, alpha, ass_round_rect2(ROW_W, RH, rt or 0, rb or 0))
end

-- Blue accent pill marking the render currently shown on screen.
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

-- Dark, translucent floating "sheet" backing the menu / picker. Rounded on every
-- corner with a faint light hairline — an iOS grouped-list card. More see-through than
-- the old sidebar so the video reads through it.
local function panel_bg_event(top, bottom)
    return string.format(
        "{\\an7\\pos(%d,%d)\\bord1\\shad0\\1c&H1A1216&\\1a&H3E&\\3c&HFFFFFF&\\3a&HDA&\\p1}%s{\\p0}",
        PANEL_X, top, ass_round_rect(PANEL_W, bottom - top, 20))
end

-- ===== keyboard-hint bar (macOS-style keycaps) =====
-- A row of little rounded keycap blocks, each optionally followed by a dim label.
-- Used for the shortcut legend under the panel title.

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

-- Shortcut legend for the full-width bottom bar (one line).
local MENU_HINTS = {
    { keys = { "0–9" }, label = "render" },
    { keys = { "J", "K" }, label = "move" },
    { keys = { "Space" }, label = "A/B" },
    { keys = { "⇧←", "⇧→" }, label = "±10s" },
    { keys = { "Tab" }, label = "hide UI" },
    { keys = { "C" }, label = "continue" },
    { keys = { "Esc" }, label = "close" },
}
local PICKER_HINTS = {
    { keys = { "1–9" }, label = "pick" },
    { keys = { "Enter" }, label = "confirm" },
    { keys = { "Esc" }, label = "back" },
}

-- A single full-width bar pinned to the bottom of the screen: shortcut keys on the
-- left, and on the right either the on-screen resolution (menu) or the cursor row's
-- detail line (picker). One comfortable line spanning edge to edge.
local function draw_bottom(opts)
    opts = opts or {}
    local hints = opts.hints
    local right_value = opts.right_value
    local right_label = opts.right_label or ""
    local right_text = trim(opts.right_text or "")
    local has_res = right_value and right_value ~= ""

    if not hints and not has_res and right_text == "" then
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
        "{\\an7\\pos(%d,%d)\\bord1\\shad0\\1c&H1A1216&\\1a&H3E&\\3c&HFFFFFF&\\3a&HDA&\\p1}%s{\\p0}",
        left, top, ass_round_rect(width, bar_h, 15)) }

    if hints then
        draw_hint_segments(ev, left + pad, cy, hints)
    end

    if has_res then
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

-- Detected source video properties, used to gate the enhancement categories.
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
        -- long-edge band: <=2560 treated as "<4K" (upscalers shown), >2560 as "4K"
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

-- Read enhancement presets, group by category, gate the upscale categories on >=4K
-- sources, substitute the @4K@ token (orientation-aware) and pre-compute per-preset
-- filters. Returns { items = {header|preset...}, presets = {selectable in order} }.
local function load_enhancement_presets(profile)
    local stdout, err = catalog_stdout("topaz_enhancement_preset_rows")
    if not stdout then
        return nil, err
    end

    local by_cat = {}
    for line in stdout:gmatch("[^\r\n]+") do
        local f = split_tsv(line)
        -- category, display, slug, filter_body, blurb, metadata
        if #f >= 4 then
            local cat = f[1]
            by_cat[cat] = by_cat[cat] or {}
            table.insert(by_cat[cat], {
                category = cat,
                display = f[2],
                slug = f[3],
                filter_body = f[4] or "",
                blurb = f[5] or "",
                metadata = f[6] or "",
            })
        end
    end

    local target_w, target_h = target_4k_dims(profile.width, profile.height)
    local up = string.format("scale=0:w=%d:h=%d", target_w, target_h)
    local tail = string.format(",scale=w=%d:h=%d:flags=lanczos:threads=0", target_w, target_h)
    local sw, sh = profile.width, profile.height

    -- For exactly-1080p sources (either orientation) 2x IS the 4K target, so the
    -- two upscale categories collapse into a single "Upscale to 4K" group.
    local two_x_is_4k = sw and sh and sw * 2 == target_w and sh * 2 == target_h

    -- Orientation/target-aware category labels (never claim "4K" on a 4K source —
    -- the upscale categories are hidden there anyway, but show the real target dims).
    local function cat_label(cat)
        if cat.key == "upscale-4k" then
            return string.format("Upscale to 4K → %d×%d", target_w, target_h)
        elseif cat.key == "upscale-2x" and sw and sh then
            return string.format("Upscale 2x → %d×%d", sw * 2, sh * 2)
        end
        return cat.label
    end

    local items = {}
    local presets = {}
    local number = 0

    -- "Do nothing" option so the user can skip enhancement and go straight to
    -- interpolation / re-encode. Numbered 0; always first.
    do
        local original = {
            is_original = true,
            display = "Original (no enhancement)",
            blurb = "Skip enhance — interpolate / re-encode only",
            slug = "__original__",
            cat_label = "Original",
            enh_filter = "",
            enh_tail = "",
            preview_filter = "",
            number = 0,
            out_w = sw,
            out_h = sh,
            metadata = "videoai=Original (no enhancement)",
        }
        table.insert(presets, original)
        table.insert(items, { kind = "preset", preset = original })
    end

    for _, cat in ipairs(CATEGORY_ORDER) do
        local skip = (cat.upscale and profile.is_4k)
            or (cat.key == "upscale-2x" and two_x_is_4k)
        if not skip then
            local rows = by_cat[cat.key]
            if cat.key == "upscale-4k" and two_x_is_4k then
                -- 2x == 4K here: fold the 2x presets into this group.
                rows = {}
                for _, r in ipairs(by_cat["upscale-2x"] or {}) do
                    rows[#rows + 1] = r
                end
                for _, r in ipairs(by_cat["upscale-4k"] or {}) do
                    rows[#rows + 1] = r
                end
            end
            if rows and #rows > 0 then
                local label = cat_label(cat)
                table.insert(items, { kind = "header", label = label })
                for _, p in ipairs(rows) do
                    p.cat_label = label
                    local has_4k = p.filter_body:find("@4K@", 1, true) ~= nil
                    -- enhancement-only filter (no lanczos tail yet); tail kept separate
                    -- so interpolation can be inserted before it at encode time.
                    p.enh_filter = p.filter_body:gsub("@4K@", up)
                    p.enh_tail = has_4k and tail or ""
                    -- still preview = enhancement + tail (tvai_fi is irrelevant to stills)
                    p.preview_filter = p.enh_filter .. p.enh_tail

                    -- predicted output resolution, for the bottom-right readout
                    if has_4k then
                        p.out_w, p.out_h = target_w, target_h
                    elseif p.filter_body:find("scale=2", 1, true) and sw and sh then
                        p.out_w, p.out_h = sw * 2, sh * 2
                    else
                        p.out_w, p.out_h = sw, sh
                    end

                    number = number + 1
                    p.number = number
                    table.insert(presets, p)
                    table.insert(items, { kind = "preset", preset = p })
                end
            end
        end
    end

    if #presets == 0 then
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

local function load_output_rows()
    local stdout, err = catalog_stdout("topaz_output_profile_rows")
    if not stdout then
        return nil, err
    end

    local rows = {}
    for line in stdout:gmatch("[^\r\n]+") do
        local f = split_tsv(line)
        if #f >= 4 then
            table.insert(rows, {
                display = f[1],
                slug = f[2] or "",
                output_ext = f[3] or "",
                video_args = f[4] or "",
            })
        end
    end

    if #rows == 0 then
        return nil, "No Topaz output profiles found"
    end

    return rows
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
        if not picker then
            draw_menu()
        end
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

function draw_menu()
    if not menu or menu.ui_hidden then
        list_badge:remove()
        params_badge:remove()
        return
    end

    list_badge.res_x = 1280
    list_badge.res_y = 720

    local cursor_preset = menu.presets[menu.cursor]

    -- Pass 1: lay each section out as a grouped "card". Events are bucketed by z-layer
    -- so cards paint behind the accent selection pill, which paints behind the row
    -- text. Section labels sit above their card. Records per-row hitboxes too.
    local cards, pills, fg, labels = {}, {}, {}, {}
    menu.hitboxes = {}

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
            local cached = menu.cache.by_slug[p.slug] ~= nil
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
                if (is_shown or cached) and p.render_secs then
                    fg[#fg + 1] = string.format(
                        "{\\an6\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs10\\1c%s}%s",
                        time_x, center, FONT,
                        lifted and "&HD9D9D9&" or "&H8A8A8A&",
                        short_secs(p.render_secs))
                end
            end

            y = y + RH
            grp_count = grp_count + 1
        end
    end
    flush_group()

    -- Panel wraps its content: short lists give a short sheet.
    local panel_bottom = math.min(y + 10, 712)

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

    -- Pass 2: sheet backing + header, then cards, pills, rows, section labels.
    local ev = {}
    ev[#ev + 1] = panel_bg_event(MARGIN_T, panel_bottom)
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(%d,%d)\\bord0\\shad0\\fn%s\\b1\\fs18\\1c&HFFFFFF&}Enhance",
        LIST_X, MARGIN_T + 12, FONT)
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs10\\1c&H9A9A9A&}%s",
        LIST_X, MARGIN_T + 32, FONT, ass_escape(meta))

    for _, e in ipairs(cards) do ev[#ev + 1] = e end
    for _, e in ipairs(pills) do ev[#ev + 1] = e end
    for _, e in ipairs(fg) do ev[#ev + 1] = e end
    for _, e in ipairs(labels) do ev[#ev + 1] = e end

    list_badge.data = table.concat(ev, "\n")
    list_badge:update()

    -- Bottom-right pill: on-screen resolution (bold) + what produced it (label).
    local res_value, res_label = nil, nil
    if not menu.rendering_slug then
        if menu.shown_slug then
            for _, p in ipairs(menu.presets) do
                if p.slug == menu.shown_slug then
                    if p.out_w and p.out_h then
                        res_value = string.format("%d×%d", p.out_w, p.out_h)
                        res_label = p.cat_label
                    end
                    break
                end
            end
        elseif menu.profile.width then
            res_value = string.format("%d×%d", menu.profile.width, menu.profile.height)
            res_label = "source"
        end
    end

    draw_bottom({
        hints = MENU_HINTS,
        right_value = res_value,
        right_label = res_label,
    })
end

-- ===== frame display / playlist management =====

-- Show the original frame: a cached original still if we have one, else the source
-- video paused at the captured timestamp.
function show_original()
    if not menu then
        return
    end

    menu.shown_slug = nil

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
        menu.last_render_slug = nil
        show_original()
        draw_menu()
        return
    end

    local cached = menu.cache.by_slug[preset.slug]
    if cached then
        menu.shown_slug = preset.slug
        menu.last_render_slug = preset.slug
        mp.commandv("playlist-play-index", tostring(cached.plindex))
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

    local render_started = mp.get_time()
    local time_arg = string.format("%.3f", menu.time_pos)
    mp.command_native_async({
        name = "subprocess",
        args = {
            topaz_preview_frame,
            "--input", menu.source,
            "--preset-name", preset.display,
            "--preset-flag", "--filter_complex",
            "--filter", preset.preview_filter,
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
        menu.cache.by_slug[preset.slug] = { image = topaz_image, plindex = plindex }
        preset.render_secs = mp.get_time() - render_started
        menu.shown_slug = preset.slug
        menu.last_render_slug = preset.slug

        mp.add_timeout(0.03, function()
            if menu and menu.shown_slug == preset.slug then
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
-- focus wash, and redraws only when the hovered row changes. No-op when nothing is open.
local function update_hover()
    local target = (menu and not menu.ui_hidden) and menu or picker
    if not target then
        return
    end

    local mx, my = mouse_virtual_pos()
    local hit = nil
    if mx and mx >= LIST_X and mx <= LIST_X + ROW_W then
        for _, hb in ipairs(target.hitboxes or {}) do
            if my >= hb.y0 and my < hb.y1 then
                hit = hb.preset and hb.preset.slug or hb.index
                break
            end
        end
    end

    if target.hover ~= hit then
        target.hover = hit
        if target == menu then
            draw_menu()
        else
            draw_picker()
        end
    end
end

-- Click on a preset row = select it (same as pressing its number).
local function menu_click()
    if not menu or menu.ui_hidden then
        return
    end
    local mx, my = mouse_virtual_pos()
    if not mx or mx < LIST_X or mx > LIST_X + ROW_W then
        return
    end
    for _, hb in ipairs(menu.hitboxes or {}) do
        if my >= hb.y0 and my < hb.y1 then
            menu.cursor = hb.preset.index
            render_or_show(hb.preset)
            return
        end
    end
end

function move_cursor(delta)
    if not menu then
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
            local cached = menu.cache.by_slug[preset.slug]
            if cached and menu.shown_slug ~= preset.slug then
                menu.shown_slug = preset.slug
                menu.last_render_slug = preset.slug
                mp.commandv("playlist-play-index", tostring(cached.plindex))
            end
        end
    end

    draw_menu()
end

function select_number(num)
    if not menu then
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
    render_or_show(menu.presets[menu.cursor])
end

function choose_enhancement()
    if not menu then
        return
    end
    menu.ui_hidden = false
    choose_interpolation(menu.presets[menu.cursor])
end

-- The render currently being compared: the cursor preset if it is cached, else the
-- most recently displayed render.
local function current_render_slug()
    if not menu then
        return nil
    end
    local cp = menu.presets[menu.cursor]
    if cp and menu.cache.by_slug[cp.slug] then
        return cp.slug
    end
    if menu.last_render_slug and menu.cache.by_slug[menu.last_render_slug] then
        return menu.last_render_slug
    end
    return nil
end

-- Show the active render (l / RIGHT). No-op if nothing is rendered yet.
function show_render()
    if not menu then
        return
    end
    local slug = current_render_slug()
    if not slug then
        return
    end
    menu.shown_slug = slug
    menu.last_render_slug = slug
    mp.commandv("playlist-play-index", tostring(menu.cache.by_slug[slug].plindex))
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

-- Tab: hide/show the whole menu UI for a clean fullscreen preview. Navigation and
-- A/B keys still work while hidden, so you can flick presets at full size.
function toggle_ui()
    if not menu then
        return
    end
    menu.ui_hidden = not menu.ui_hidden
    if menu.ui_hidden then
        list_badge:remove()
        params_badge:remove()
    else
        draw_menu()
    end
end

-- ===== generic left-docked picker (interpolation / output steps) =====
-- Same look and key model as the enhancement menu (number keys / j-k / Enter / Esc),
-- but with no rendering. Each row carries .display and an optional .detail (shown in
-- the bottom pane). Runs while the enhancement `menu` state stays alive in the
-- background so its preview images can still be cleaned up at the end.

local PICKER_KEY_NAMES = {
    "topaz_pick_down", "topaz_pick_down2",
    "topaz_pick_up", "topaz_pick_up2",
    "topaz_pick_enter", "topaz_pick_kp_enter",
    "topaz_pick_block_n", "topaz_pick_block_N",
    "topaz_pick_click",
    "topaz_pick_esc", "topaz_pick_bs",
}

function draw_picker()
    if not picker then
        list_badge:remove()
        params_badge:remove()
        return
    end

    list_badge.res_x = 1280
    list_badge.res_y = 720

    -- One grouped card holding every picker row; the accent pill marks the cursor.
    local pills, fg = {}, {}
    picker.hitboxes = {}
    local count = #picker.rows
    local y = LIST_TOP
    local grp_y0 = y
    for i, row in ipairs(picker.rows) do
        local rt = (i == 1) and 12 or 0
        local rb = (i == count) and 12 or 0
        if i == picker.cursor then
            pills[#pills + 1] = selection_pill_event(y, rt, rb)
        elseif picker.hover == i then
            pills[#pills + 1] = row_pill(y, "&HFFFFFF&", "&HD2&", rt, rb)
        end
        table.insert(picker.hitboxes, { y0 = y, y1 = y + RH, index = i })

        local center = y + 15
        local cap, num = keycap_events(center, i, i == picker.cursor)
        fg[#fg + 1] = cap
        fg[#fg + 1] = num

        local bold = (i == picker.cursor) and "\\b1" or "\\b0"
        fg[#fg + 1] = string.format(
            "{\\an4\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs13%s\\1c&HFFFFFF&}%s",
            TEXT_X, center, FONT, bold, ass_escape(row.display))

        y = y + RH
    end

    local panel_bottom = math.min(y + 10, 712)

    -- Pass 2: sheet backing + heading, then the card, pills, and rows.
    local ev = {}
    ev[#ev + 1] = panel_bg_event(MARGIN_T, panel_bottom)
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(%d,%d)\\bord0\\shad0\\fn%s\\b1\\fs18\\1c&HFFFFFF&}%s",
        LIST_X, MARGIN_T + 12, FONT, ass_escape(picker.title))
    if picker.subtitle and picker.subtitle ~= "" then
        ev[#ev + 1] = string.format(
            "{\\an7\\pos(%d,%d)\\bord0\\shad0\\fn%s\\fs10\\1c&H9A9A9A&}%s",
            LIST_X, MARGIN_T + 32, FONT, ass_escape(picker.subtitle))
    end
    if count > 0 then
        for _, e in ipairs(group_card_events(grp_y0, count)) do
            ev[#ev + 1] = e
        end
    end
    for _, e in ipairs(pills) do ev[#ev + 1] = e end
    for _, e in ipairs(fg) do ev[#ev + 1] = e end

    list_badge.data = table.concat(ev, "\n")
    list_badge:update()

    local row = picker.rows[picker.cursor]
    draw_bottom({
        hints = PICKER_HINTS,
        right_text = row and row.detail,
    })
end

local function picker_move(delta)
    if not picker then
        return
    end
    local n = #picker.rows
    if n == 0 then
        return
    end
    picker.cursor = ((picker.cursor - 1 + delta) % n) + 1
    draw_picker()
end

function remove_picker_keys()
    for _, name in ipairs(PICKER_KEY_NAMES) do
        mp.remove_key_binding(name)
    end
    for i = 1, 9 do
        mp.remove_key_binding("topaz_pick_num_" .. i)
    end
end

local function picker_submit(index)
    if not picker then
        return
    end
    if not (index and picker.rows[index]) then
        return
    end
    local fn = picker.submit
    remove_picker_keys()
    picker = nil
    if fn then
        fn(index)
    end
end

local function picker_cancel()
    if not picker then
        return
    end
    local fn = picker.cancel
    remove_picker_keys()
    picker = nil
    if fn then
        fn()
    end
end

-- Click a picker row = choose it (same as pressing its number).
local function picker_click()
    if not picker then
        return
    end
    local mx, my = mouse_virtual_pos()
    if not mx or mx < LIST_X or mx > LIST_X + ROW_W then
        return
    end
    for _, hb in ipairs(picker.hitboxes or {}) do
        if my >= hb.y0 and my < hb.y1 then
            picker.cursor = hb.index
            picker_submit(hb.index)
            return
        end
    end
end

local function enable_picker_keys()
    mp.add_forced_key_binding("j", "topaz_pick_down", function() picker_move(1) end)
    mp.add_forced_key_binding("DOWN", "topaz_pick_down2", function() picker_move(1) end)
    mp.add_forced_key_binding("k", "topaz_pick_up", function() picker_move(-1) end)
    mp.add_forced_key_binding("UP", "topaz_pick_up2", function() picker_move(-1) end)
    for i = 1, 9 do
        mp.add_forced_key_binding(tostring(i), "topaz_pick_num_" .. i, function()
            if picker and picker.rows[i] then
                picker.cursor = i
                picker_submit(i)
            end
        end)
    end
    mp.add_forced_key_binding("ENTER", "topaz_pick_enter", function()
        picker_submit(picker and picker.cursor)
    end)
    mp.add_forced_key_binding("KP_ENTER", "topaz_pick_kp_enter", function()
        picker_submit(picker and picker.cursor)
    end)
    -- Swallow playlist-next/prev: preview stills are still in the playlist here.
    mp.add_forced_key_binding("n", "topaz_pick_block_n", function() end)
    mp.add_forced_key_binding("N", "topaz_pick_block_N", function() end)
    mp.add_forced_key_binding("MBTN_LEFT", "topaz_pick_click", picker_click)
    mp.add_forced_key_binding("ESC", "topaz_pick_esc", picker_cancel)
    mp.add_forced_key_binding("BS", "topaz_pick_bs", picker_cancel)
end

function open_picker(opts)
    remove_menu_keys()
    picker = {
        title = opts.title,
        subtitle = opts.subtitle,
        rows = opts.rows,
        cursor = 1,
        submit = opts.submit,
        cancel = opts.cancel,
    }
    enable_picker_keys()
    draw_picker()
end

-- ===== step 2: interpolation =====

function choose_interpolation(enh_preset)
    if not menu then
        return
    end

    local rows = { {
        display = "None (enhancement only)",
        fi = nil,
        detail = "Encode the enhancement as-is, keeping the source frame rate.",
    } }
    local stdout = catalog_stdout("topaz_interpolation_preset_rows")
    if stdout then
        for line in stdout:gmatch("[^\r\n]+") do
            local f = split_tsv(line)
            -- display, slug, fi_filter, metadata
            if #f >= 3 then
                table.insert(rows, {
                    display = f[1],
                    slug = f[2],
                    fi = f[3],
                    detail = (f[4] or ""):gsub("^videoai=", ""),
                })
            end
        end
    end

    open_picker({
        title = "Interpolation",
        subtitle = "Apollo fixes duplicate frames",
        rows = rows,
        submit = function(index)
            choose_output(enh_preset, rows[index])
        end,
        cancel = function()
            if menu then
                enable_menu_keys()
                draw_menu()
            end
        end,
    })
end

-- ===== step 3: output format + launch encode =====

function choose_output(enh_preset, interp)
    if not menu then
        return
    end

    local outputs, err = load_output_rows()
    if not outputs then
        mp.osd_message(err, 3)
        if menu then
            enable_menu_keys()
            draw_menu()
        end
        return
    end

    for _, row in ipairs(outputs) do
        row.detail = row.video_args
    end

    open_picker({
        title = "Output format",
        subtitle = "",
        rows = outputs,
        submit = function(index)
            start_encode(enh_preset, interp, outputs[index])
        end,
        cancel = function()
            choose_interpolation(enh_preset)
        end,
    })
end

local function compose_final_filter(enh, interp)
    local parts = {}
    if enh.enh_filter and enh.enh_filter ~= "" then
        parts[#parts + 1] = enh.enh_filter
    end
    if interp and interp.fi and interp.fi ~= "" then
        parts[#parts + 1] = interp.fi
    end
    local body = table.concat(parts, ",")
    -- "original" with no interpolation has no filter at all; pass a no-op so the
    -- re-encode still runs (format/codec change only).
    if body == "" and (not enh.enh_tail or enh.enh_tail == "") then
        return "null"
    end
    return body .. (enh.enh_tail or "")
end

function start_encode(enh, interp, output)
    if not menu then
        return
    end

    local source = menu.source
    local directory = utils.split_path(source)
    local final_filter = compose_final_filter(enh, interp)
    local interp_label = (interp and interp.fi) and (" + " .. interp.display) or ""
    local preset_name = enh.display .. interp_label .. " - " .. output.display

    local metadata = enh.metadata
    if not metadata or metadata == "" then
        metadata = "videoai=" .. enh.display
    end

    local args = {
        kitty_launch,
        "--tab",
        "--hold",
        "--cwd", directory,
        "--title", " topaz encode ",
        "--",
        topaz_run,
        "--preset_name", preset_name,
        "--filter_complex", final_filter,
        "--output_ext", output.output_ext,
        "--video_args", output.video_args,
        "--metadata", metadata,
        "--",
        source,
    }

    local result = utils.subprocess_detached({ args = args })
    if result == false then
        mp.osd_message("Topaz encode launch failed", 2)
        mp.msg.error("Failed to launch Topaz encode for: " .. source)
        return
    end

    close_menu("Topaz encode started")
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
    for _, p in ipairs(menu.presets) do
        p.render_secs = nil
    end
    local stale = menu.appended
    menu.appended = {}
    menu.cache = { original = nil, by_slug = {} }
    menu.shown_slug = nil
    menu.original_plindex = nil
    menu.last_render_slug = nil

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
    "topaz_menu_space", "topaz_menu_tab",
    "topaz_menu_seek_fwd", "topaz_menu_seek_back",
    "topaz_menu_show_render_l", "topaz_menu_show_render_right",
    "topaz_menu_show_orig_h", "topaz_menu_show_orig_left",
    "topaz_menu_proceed_c",
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
    -- Render (or show cached render of) the cursor preset.
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
    -- Tab hides/shows the UI for a clean fullscreen preview.
    mp.add_forced_key_binding("TAB", "topaz_menu_tab", toggle_ui)
    -- Skip the preview point ±10s without leaving the renderer.
    mp.add_forced_key_binding("Shift+RIGHT", "topaz_menu_seek_fwd", function() menu_seek(10) end)
    mp.add_forced_key_binding("Shift+LEFT", "topaz_menu_seek_back", function() menu_seek(-10) end)
    -- Proceed to the interpolation step with the highlighted preset.
    mp.add_forced_key_binding("c", "topaz_menu_proceed_c", choose_enhancement)
    -- Swallow playlist-next/prev so preview stills don't get navigated away.
    mp.add_forced_key_binding("n", "topaz_menu_block_n", function() end)
    mp.add_forced_key_binding("N", "topaz_menu_block_N", function() end)
    -- Click a preset row to select it.
    mp.add_forced_key_binding("MBTN_LEFT", "topaz_menu_click", menu_click)
    mp.add_forced_key_binding("ESC", "topaz_menu_esc", function()
        close_menu("Topaz menu closed")
    end)
    mp.add_forced_key_binding("BS", "topaz_menu_bs", function()
        close_menu("Topaz menu closed")
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
    local playlist_count = mp.get_property_number("playlist-count", 0)
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
        cursor = 1,
        hover = nil,
        shown_slug = nil,
        rendering_slug = nil,
        cache = { original = nil, by_slug = {} },
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
    picker = nil
    remove_menu_keys()
    remove_picker_keys()
    stop_render_timer()
    list_badge:remove()
    params_badge:remove()
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
    local data
    data, err = load_enhancement_presets(profile)
    if not data then
        mp.osd_message(err, 3)
        return
    end

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
    remove_picker_keys()
    if encode_timer then
        encode_timer:kill()
        encode_timer = nil
    end
    list_badge:remove()
    params_badge:remove()
end)

-- Row hover highlighting (no-op unless a menu / picker is open).
mp.observe_property("mouse-pos", "native", update_hover)

mp.add_key_binding(nil, "topaz_workflow_current_file", show_transform_menu)
mp.add_key_binding(nil, "topaz_workflow_external_file", topaz_workflow_external_file)
