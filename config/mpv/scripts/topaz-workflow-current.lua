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
local encode_badge = mp.create_osd_overlay("ass-events")
encode_badge.z = 31
local encode_timer = nil
local encode_start = nil

-- Categories shown in the enhancement menu, in display order. `upscale` rows are
-- hidden when the source is already >= 4K.
local CATEGORY_ORDER = {
    { key = "upscale-2x", label = "Upscale 2x", upscale = true },
    { key = "upscale-4k", label = "Upscale to 4K", upscale = true },
    { key = "repair", label = "Repair", upscale = false },
    { key = "sharpen", label = "Sharpen", upscale = false },
    { key = "focus-fix", label = "Focus Fix", upscale = false },
}

-- Forward declarations (so the menu functions can reference each other).
local draw_menu, show_original, render_or_show, move_cursor, select_number
local render_cursor, choose_enhancement, choose_interpolation, choose_output
local start_encode, close_menu, enable_menu_keys, remove_menu_keys
local draw_picker, open_picker, remove_picker_keys
local space_toggle, show_render, show_orig, toggle_ui

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

-- Escape text for use inside an ASS event (literal braces/backslashes).
local function ass_escape(value)
    return (tostring(value or ""))
        :gsub("\\", "\\\\")
        :gsub("{", "\\{")
        :gsub("}", "\\}")
end

-- Soft-wrap a comma-separated filter string into ASS lines (\N) for the params pane.
local function wrap_commas(value, width)
    width = width or 78
    local out, line = {}, ""
    for token in value:gmatch("[^,]+,?") do
        if #line > 0 and #line + #token > width then
            table.insert(out, line)
            line = token
        else
            line = line .. token
        end
    end
    if line ~= "" then
        table.insert(out, line)
    end
    return table.concat(out, "\\N")
end

-- ===== ASS drawing helpers / shared menu layout =====
-- Coordinates are in the 1280x720 virtual space used by every overlay here.

local FONT = "Helvetica Neue"  -- system UI font for the whole panel
local LIST_X = 20       -- left edge of the selectable row / selection box
local ROW_W = 250       -- selection box width
local RH = 30           -- preset / picker row height (two text lines)
local HH = 22           -- category header row advance
local LIST_TOP = 84     -- y where the row list starts
local CHIP_R = 10       -- index circle radius
local PANEL_X = 8       -- left menu backing panel
local PANEL_W = 270
local PANEL_TOP = 8

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

-- Circle drawing path with radius r, bounding box top-left at the event \pos.
local function ass_circle(r)
    local k = r * 0.5523
    return string.format(
        "m %g 0 b %g 0 %g %g %g %g b %g %g %g %g %g %g "
            .. "b %g %g %g %g %g %g b %g %g %g %g %g %g",
        r,
        r + k, 2 * r, r - k, 2 * r, r,
        2 * r, r + k, r + k, 2 * r, r, 2 * r,
        r - k, 2 * r, 0, r + k, 0, r,
        0, r - k, r - k, 0, r, 0)
end

-- The index "chip": a transparent circle with a thin border plus a centred number,
-- vertically centred on `center_y` so it aligns to the middle of the whole row unit.
-- `border` overrides the ring colour (BGR; default white). Returns two ASS events
-- (circle, number) and the row-text start x.
local function chip_events(center_y, number, border)
    local cx = LIST_X + 11
    local circle = string.format(
        "{\\an7\\pos(%d,%d)\\bord1.2\\shad0\\1a&HFF&\\3c%s\\3a&H00&\\p1}%s{\\p0}",
        cx - CHIP_R, center_y - CHIP_R, border or "&HFFFFFF&", ass_circle(CHIP_R))
    local num = string.format(
        "{\\an5\\pos(%d,%d)\\bord1\\shad0\\3c&H000000&\\fn%s\\fs12\\b0\\1c&HFFFFFF&}%s",
        cx, center_y, FONT, tostring(number))
    return circle, num, cx + CHIP_R + 10
end

-- White outline box around the selected row (drawn with a transparent fill so the
-- row contents do not shift when selection moves).
local function selection_box_event(y)
    return string.format(
        "{\\an7\\pos(%d,%d)\\bord2\\shad0\\1a&HFF&\\3c&HFFFFFF&\\3a&H00&\\p1}%s{\\p0}",
        LIST_X, y, ass_round_rect(ROW_W, RH, 6))
end

-- Semi-transparent dark backing panel for the menu / picker list.
local function panel_bg_event(top, bottom)
    return string.format(
        "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c&H000000&\\1a&H66&\\p1}%s{\\p0}",
        PANEL_X, top, ass_round_rect(PANEL_W, bottom - top, 10))
end

-- Bottom parameter pane: a thin semi-transparent strip showing only the cursor
-- preset's filter chain (left), plus an optional bottom-right "pill" showing a bold
-- value (e.g. output resolution) over a smaller label. Shared by menu + picker.
local function draw_params(body, right_value, right_label)
    local has_pill = right_value and right_value ~= ""
    local has_body = body and body ~= ""
    if not has_body and not has_pill then
        params_badge:remove()
        return
    end

    params_badge.res_x = 1280
    params_badge.res_y = 720

    local ev = {}
    if has_body then
        ev[#ev + 1] = "{\\an7\\pos(0,686)\\bord0\\shad0\\1c&H000000&\\1a&H44&\\p1}"
            .. ass_rect(1280, 34) .. "{\\p0}"
        ev[#ev + 1] = string.format(
            "{\\an4\\pos(20,703)\\bord1\\shad0\\3c&H000000&\\fn%s\\fs12\\1c&HBBBBBB&}%s",
            FONT, wrap_commas(ass_escape(body), 150))
    end

    if has_pill then
        local label = right_label or ""
        local w = math.max(#right_value * 13, #label * 7) + 30
        if w < 96 then
            w = 96
        end
        local h = (label ~= "") and 50 or 34
        local right = 1268
        local px = right - w
        local top = 678 - h
        ev[#ev + 1] = string.format(
            "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c&H222222&\\1a&H1A&\\p1}%s{\\p0}",
            px, top, ass_round_rect(w, h, 12))
        ev[#ev + 1] = string.format(
            "{\\an6\\pos(%d,%d)\\bord1\\shad0\\3c&H000000&\\fn%s\\fs22\\b1\\1c&H66FF66&}%s",
            right - 14, top + (label ~= "" and 17 or math.floor(h / 2)), FONT, ass_escape(right_value))
        if label ~= "" then
            ev[#ev + 1] = string.format(
                "{\\an6\\pos(%d,%d)\\bord1\\shad0\\3c&H000000&\\fn%s\\fs13\\1c&HCCCCCC&}%s",
                right - 14, top + 36, FONT, ass_escape(label))
        end
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

local function profile_summary(profile)
    if not profile or profile.show_all or not profile.long_edge then
        return "source unknown — all presets"
    end

    local res = profile.is_4k and "4K source" or "<4K · upscale offered"
    local fps = profile.fps and string.format("%.0ffps", profile.fps) or "fps?"
    return string.format("%dx%d · %s · %s", profile.width, profile.height, fps, res)
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

-- ===== persistent "rendering" badge (top center) =====

local function update_encode_badge(label)
    encode_badge.res_x = 1280
    encode_badge.res_y = 720
    local elapsed = encode_start and (mp.get_time() - encode_start) or 0
    local dots = string.rep(".", (math.floor(elapsed) % 3) + 1)
    -- bottom-right corner, compact
    encode_badge.data = string.format(
        "{\\an3\\pos(1264,706)\\fs17\\bord2\\shad0\\3c&H000000&\\1c&H66CCFF&\\b1}"
            .. "Rendering preview%s  %.0fs",
        dots,
        elapsed
    )
    encode_badge:update()
end

local function start_encode_badge(label)
    encode_start = mp.get_time()
    update_encode_badge(label)
    if encode_timer then
        encode_timer:kill()
    end
    encode_timer = mp.add_periodic_timer(0.5, function()
        update_encode_badge(label)
    end)
end

local function stop_encode_badge()
    if encode_timer then
        encode_timer:kill()
        encode_timer = nil
    end
    encode_start = nil
    encode_badge:remove()
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

    -- Pass 1: lay out the rows (so we know the panel height before drawing it).
    -- Also records per-row hitboxes (virtual 1280x720 coords) for mouse clicks.
    local body = {}
    local y = LIST_TOP
    menu.hitboxes = {}

    for _, item in ipairs(menu.items) do
        if item.kind == "header" then
            body[#body + 1] = string.format(
                "{\\an4\\pos(%d,%d)\\bord2\\shad0\\3c&H000000&\\fn%s\\b1\\fs12\\1c&H99DDDD&}%s",
                LIST_X + 2, y + math.floor(HH / 2), FONT, ass_escape(item.label))
            y = y + HH
        else
            local p = item.preset
            local is_cursor = (p == cursor_preset)
            local cached = menu.cache.by_slug[p.slug] ~= nil
            local is_shown = p.is_original and (menu.shown_slug == nil)
                or (menu.shown_slug == p.slug)
            local is_rendering = (menu.rendering_slug == p.slug)

            local center = y + 15  -- vertical centre of the two-line row unit
            table.insert(menu.hitboxes, { y0 = y, y1 = y + RH, preset = p })

            if is_cursor then
                body[#body + 1] = selection_box_event(y)
            end

            local chip_border = p.is_original and "&H888888&" or "&HFFFFFF&"
            local box, num, text_x = chip_events(center, p.number, chip_border)
            body[#body + 1] = box
            body[#body + 1] = num

            local namecolor = is_rendering and "&H66CCFF&"
                or (cached and "&H66FF66&" or "&HFFFFFF&")
            local bold = is_cursor and "\\b1" or "\\b0"
            local suffix = ""
            if is_shown and not is_rendering then
                suffix = "  {\\fs11\\1c&H66FF66&}● shown"
            end

            -- line 1: name (+ shown marker)
            body[#body + 1] = string.format(
                "{\\an4\\pos(%d,%d)\\bord2\\shad0\\3c&H000000&\\fn%s\\fs15%s\\1c%s}%s%s",
                text_x, y + 10, FONT, bold, namecolor, ass_escape(p.display), suffix)
            -- line 2: short description (never bold, even on the cursor row)
            body[#body + 1] = string.format(
                "{\\an4\\pos(%d,%d)\\bord1\\shad0\\3c&H000000&\\fn%s\\fs11\\b0\\1c&H9A9A9A&}%s",
                text_x, y + 22, FONT, ass_escape(p.blurb))

            -- cached badge (small check), centred on the row unit at the right edge
            if cached then
                body[#body + 1] = string.format(
                    "{\\an6\\pos(%d,%d)\\bord2\\shad0\\3c&H000000&\\fs13\\1c&H66FF66&}✓",
                    LIST_X + ROW_W - 10, center)
            end

            y = y + RH
        end
    end

    -- Pass 2: panel backing, header text, then the rows on top.
    local ev = {}
    ev[#ev + 1] = panel_bg_event(PANEL_TOP, y + 6)
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(%d,16)\\bord2\\shad0\\3c&H000000&\\fn%s\\b1\\fs20\\1c&HFFFFFF&}TOPAZ RENDER",
        LIST_X, FONT)
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(%d,40)\\bord2\\shad0\\3c&H000000&\\fn%s\\fs11\\1c&HBBBBBB&}%s  @ %s",
        LIST_X, FONT, ass_escape(profile_summary(menu.profile)), format_time(menu.time_pos))
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(%d,55)\\bord2\\shad0\\3c&H000000&\\fn%s\\fs10\\1c&H888888&}%s",
        LIST_X, FONT, "0-9/R/Enter render · j/k move · Tab fullscreen")
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(%d,68)\\bord2\\shad0\\3c&H000000&\\fn%s\\fs10\\1c&H888888&}%s",
        LIST_X, FONT, "Space or h/l: A/B vs original · c proceed · Esc")
    for _, e in ipairs(body) do
        ev[#ev + 1] = e
    end

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

    draw_params(cursor_preset and cursor_preset.enh_filter, res_value, res_label)
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
    start_encode_badge(string.format("%s  @ %s", preset.display, format_time(menu.time_pos)))

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
        stop_encode_badge()
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

    -- Pass 1: rows (recording hitboxes for mouse clicks).
    local body = {}
    local y = LIST_TOP
    picker.hitboxes = {}
    for i, row in ipairs(picker.rows) do
        if i == picker.cursor then
            body[#body + 1] = selection_box_event(y)
        end
        table.insert(picker.hitboxes, { y0 = y, y1 = y + RH, index = i })

        local center = y + 15
        local box, num, text_x = chip_events(center, i)
        body[#body + 1] = box
        body[#body + 1] = num

        local bold = (i == picker.cursor) and "\\b1" or "\\b0"
        body[#body + 1] = string.format(
            "{\\an4\\pos(%d,%d)\\bord2\\shad0\\3c&H000000&\\fn%s\\fs15%s\\1c&HFFFFFF&}%s",
            text_x, center, FONT, bold, ass_escape(row.display))

        y = y + RH
    end

    -- Pass 2: panel + heading on top.
    local ev = {}
    ev[#ev + 1] = panel_bg_event(PANEL_TOP, y + 6)
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(%d,16)\\bord2\\shad0\\3c&H000000&\\fn%s\\b1\\fs20\\1c&HFFFFFF&}%s",
        LIST_X, FONT, ass_escape(picker.title))
    if picker.subtitle and picker.subtitle ~= "" then
        ev[#ev + 1] = string.format(
            "{\\an7\\pos(%d,42)\\bord2\\shad0\\3c&H000000&\\fn%s\\fs11\\1c&H888888&}%s",
            LIST_X, FONT, ass_escape(picker.subtitle))
    end
    for _, e in ipairs(body) do
        ev[#ev + 1] = e
    end

    list_badge.data = table.concat(ev, "\n")
    list_badge:update()

    local row = picker.rows[picker.cursor]
    draw_params(row and row.detail)
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
        title = "INTERPOLATION",
        subtitle = "Apollo fixes duplicate frames · Enter pick · Esc back",
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
        title = "OUTPUT FORMAT",
        subtitle = "Enter pick · Esc back",
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

-- ===== key bindings =====

local MENU_KEY_NAMES = {
    "topaz_menu_down", "topaz_menu_down2",
    "topaz_menu_up", "topaz_menu_up2",
    "topaz_menu_render_r", "topaz_menu_render_R",
    "topaz_menu_render_enter", "topaz_menu_render_kp_enter",
    "topaz_menu_space", "topaz_menu_tab",
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
        profile = profile,
        items = data.items,
        presets = data.presets,
        by_number = data.by_number,
        cursor = 1,
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
    stop_encode_badge()
    list_badge:remove()
    params_badge:remove()

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
    encode_badge:remove()
end)

mp.add_key_binding(nil, "topaz_workflow_current_file", show_transform_menu)
mp.add_key_binding(nil, "topaz_workflow_external_file", topaz_workflow_external_file)
