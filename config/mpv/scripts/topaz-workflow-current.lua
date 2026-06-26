local utils = require "mp.utils"

local kitty_launch = "/Users/nom/.zsh/bin/kitty-launch"
local topaz_workflow = "/Users/nom/.zsh/bin/topaz-workflow"
local topaz_run = "/Users/nom/.zsh/bin/topaz-run"
local topaz_preview_frame = "/Users/nom/.zsh/bin/topaz-preview-frame"
local preset_catalog = "/Users/nom/config/config/zsh/bin/topaz-preset-catalog.zsh"

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
local toggle_ab, tab_cycle_cached

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

local LIST_X = 24       -- left edge of the selectable row / selection box
local ROW_W = 604       -- selection box width
local RH = 32           -- preset / picker row height
local HH = 30           -- category header row height
local LIST_TOP = 92     -- y where the row list starts
local CHIP_W = 28
local CHIP_H = 22

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

-- The accent index "chip" (filled rounded box + centred number) drawn at row top `y`.
-- Returns two ASS events (box, number) and the x where the row text should start.
local function chip_events(y, number)
    local chip_x = LIST_X + 12
    local chip_y = y + math.floor((RH - CHIP_H) / 2)
    local box = string.format(
        "{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c&HCC6600&\\p1}%s{\\p0}",
        chip_x, chip_y, ass_round_rect(CHIP_W, CHIP_H, 6))
    local num = string.format(
        "{\\an5\\pos(%d,%d)\\bord1\\shad0\\3c&H000000&\\fs16\\b1\\1c&HFFFFFF&}%d",
        chip_x + math.floor(CHIP_W / 2), chip_y + math.floor(CHIP_H / 2), number)
    return box, num, chip_x + CHIP_W + 14
end

-- White outline box around the selected row (drawn with a transparent fill so the
-- row contents do not shift when selection moves).
local function selection_box_event(y)
    return string.format(
        "{\\an7\\pos(%d,%d)\\bord2\\shad0\\1a&HFF&\\3c&HFFFFFF&\\3a&H00&\\p1}%s{\\p0}",
        LIST_X, y, ass_round_rect(ROW_W, RH, 6))
end

-- Bottom parameter pane: a semi-transparent black strip plus title + wrapped body,
-- so the text stays legible over light video. Shared by the menu and the picker.
local function draw_params(title, body)
    if not title or title == "" then
        params_badge:remove()
        return
    end

    params_badge.res_x = 1280
    params_badge.res_y = 720

    local ev = {}
    ev[#ev + 1] = "{\\an7\\pos(0,640)\\bord0\\shad0\\1c&H000000&\\1a&H44&\\p1}"
        .. ass_rect(1280, 80) .. "{\\p0}"
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(24,646)\\bord1\\shad0\\3c&H000000&\\fs17\\b1\\1c&HFFFFFF&}%s",
        ass_escape(title))
    if body and body ~= "" then
        ev[#ev + 1] = string.format(
            "{\\an7\\pos(24,668)\\bord1\\shad0\\3c&H000000&\\fs13\\1c&HAAAAAA&}%s",
            wrap_commas(ass_escape(body)))
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
        -- category, display, slug, filter_body, metadata
        if #f >= 4 then
            local cat = f[1]
            by_cat[cat] = by_cat[cat] or {}
            table.insert(by_cat[cat], {
                category = cat,
                display = f[2],
                slug = f[3],
                filter_body = f[4] or "",
                metadata = f[5] or "",
            })
        end
    end

    local target_w, target_h = target_4k_dims(profile.width, profile.height)
    local up = string.format("scale=0:w=%d:h=%d", target_w, target_h)
    local tail = string.format(",scale=w=%d:h=%d:flags=lanczos:threads=0", target_w, target_h)

    local items = {}
    local presets = {}
    local number = 0

    for _, cat in ipairs(CATEGORY_ORDER) do
        if not (cat.upscale and profile.is_4k) then
            local rows = by_cat[cat.key]
            if rows and #rows > 0 then
                table.insert(items, { kind = "header", label = cat.label })
                for _, p in ipairs(rows) do
                    local has_4k = p.filter_body:find("@4K@", 1, true) ~= nil
                    -- enhancement-only filter (no lanczos tail yet); tail kept separate
                    -- so interpolation can be inserted before it at encode time.
                    p.enh_filter = p.filter_body:gsub("@4K@", up)
                    p.enh_tail = has_4k and tail or ""
                    -- still preview = enhancement + tail (tvai_fi is irrelevant to stills)
                    p.preview_filter = p.enh_filter .. p.enh_tail
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

    return { items = items, presets = presets }
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
    encode_badge.data = string.format(
        "{\\an8\\pos(640,28)\\fs28\\bord2\\3c&H000000&\\1c&H66CCFF&\\b1}"
            .. "Rendering Topaz preview%s\\N"
            .. "{\\fs20\\1c&HDDDDDD&\\b0}%s   ·   %.0fs",
        dots,
        ass_escape(label),
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
    if not menu then
        list_badge:remove()
        params_badge:remove()
        return
    end

    list_badge.res_x = 1280
    list_badge.res_y = 720

    local ev = {}
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(%d,26)\\bord2\\shad0\\3c&H000000&\\b1\\fs26\\1c&H66CCFF&}TOPAZ ENHANCE",
        LIST_X)
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(%d,56)\\bord2\\shad0\\3c&H000000&\\fs15\\1c&HBBBBBB&}%s",
        LIST_X, ass_escape(profile_summary(menu.profile)))
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(%d,74)\\bord2\\shad0\\3c&H000000&\\fs14\\1c&H888888&}"
            .. "@ %s   ·   1-9 / R render · j/k move · Enter pick · Esc cancel",
        LIST_X, format_time(menu.time_pos))

    local cursor_preset = menu.presets[menu.cursor]
    local y = LIST_TOP

    for _, item in ipairs(menu.items) do
        if item.kind == "header" then
            y = y + 6
            ev[#ev + 1] = string.format(
                "{\\an4\\pos(%d,%d)\\bord2\\shad0\\3c&H000000&\\b1\\fs18\\1c&H99DDDD&}%s",
                LIST_X + 4, y + math.floor(HH / 2), ass_escape(item.label))
            y = y + HH
        else
            local p = item.preset
            local is_cursor = (p == cursor_preset)
            local cached = menu.cache.by_slug[p.slug] ~= nil
            local is_shown = (menu.shown_slug == p.slug)
            local is_rendering = (menu.rendering_slug == p.slug)

            if is_cursor then
                ev[#ev + 1] = selection_box_event(y)
            end

            local box, num, text_x = chip_events(y, p.number)
            ev[#ev + 1] = box
            ev[#ev + 1] = num

            local color = cached and "&H66FF66&" or "&HFFFFFF&"
            local bold = is_cursor and "\\b1" or "\\b0"
            local suffix = ""
            if is_rendering then
                suffix = "  {\\fs13\\1c&H66CCFF&}rendering…"
            elseif is_shown then
                suffix = "  {\\fs13\\1c&H66FF66&}● shown"
            end

            ev[#ev + 1] = string.format(
                "{\\an4\\pos(%d,%d)\\bord2\\shad0\\3c&H000000&\\fs20%s\\1c%s}%s%s",
                text_x, y + math.floor(RH / 2), bold, color, ass_escape(p.display), suffix)

            -- cached-render badge, right-aligned within the row
            if cached then
                ev[#ev + 1] = string.format(
                    "{\\an6\\pos(%d,%d)\\bord2\\shad0\\3c&H000000&\\fs17\\1c&H66FF66&}✓ rendered",
                    LIST_X + ROW_W - 12, y + math.floor(RH / 2))
            end

            y = y + RH
        end
    end

    list_badge.data = table.concat(ev, "\n")
    list_badge:update()

    if cursor_preset then
        draw_params(cursor_preset.display, cursor_preset.enh_filter)
    else
        params_badge:remove()
    end
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

    local cached = menu.cache.by_slug[preset.slug]
    if cached then
        menu.shown_slug = preset.slug
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

        mp.add_timeout(0.03, function()
            if menu and menu.shown_slug == preset.slug then
                mp.commandv("playlist-play-index", tostring(plindex))
            end
            draw_menu()
        end)
    end)
end

-- ===== navigation / key actions =====

function move_cursor(delta)
    if not menu then
        return
    end
    local n = #menu.presets
    if n == 0 then
        return
    end
    menu.cursor = ((menu.cursor - 1 + delta) % n) + 1

    -- If the newly highlighted preset is already rendered, switching to it is free,
    -- so preview it immediately on hover.
    local preset = menu.presets[menu.cursor]
    local cached = preset and menu.cache.by_slug[preset.slug]
    if cached and not menu.rendering_slug and menu.shown_slug ~= preset.slug then
        menu.shown_slug = preset.slug
        mp.commandv("playlist-play-index", tostring(cached.plindex))
    end

    draw_menu()
end

function select_number(num)
    if not menu then
        return
    end
    local preset = menu.presets[num]
    if not preset then
        return
    end
    menu.cursor = num
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
    choose_interpolation(menu.presets[menu.cursor])
end

-- A/B flick: if the cursor preset's render is currently shown, flip to the original;
-- otherwise flip (back) to its render. Only meaningful once the preset is cached.
function toggle_ab()
    if not menu then
        return
    end
    local preset = menu.presets[menu.cursor]
    local cached = preset and menu.cache.by_slug[preset.slug]
    if not cached then
        mp.osd_message("Render this preset first (R) to A/B it", 1.5)
        return
    end

    if menu.shown_slug == preset.slug then
        show_original()
    else
        menu.shown_slug = preset.slug
        mp.commandv("playlist-play-index", tostring(cached.plindex))
    end
    draw_menu()
end

-- Cycle the cursor through the presets that already have a cached render, showing
-- each as we land on it.
function tab_cycle_cached()
    if not menu then
        return
    end
    local n = #menu.presets
    for step = 1, n do
        local idx = ((menu.cursor - 1 + step) % n) + 1
        local preset = menu.presets[idx]
        local cached = menu.cache.by_slug[preset.slug]
        if cached then
            menu.cursor = idx
            menu.shown_slug = preset.slug
            mp.commandv("playlist-play-index", tostring(cached.plindex))
            draw_menu()
            return
        end
    end
    mp.osd_message("No rendered presets yet", 1.5)
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

    local ev = {}
    ev[#ev + 1] = string.format(
        "{\\an7\\pos(%d,26)\\bord2\\shad0\\3c&H000000&\\b1\\fs26\\1c&H66CCFF&}%s",
        LIST_X, ass_escape(picker.title))
    if picker.subtitle and picker.subtitle ~= "" then
        ev[#ev + 1] = string.format(
            "{\\an7\\pos(%d,58)\\bord2\\shad0\\3c&H000000&\\fs14\\1c&H888888&}%s",
            LIST_X, ass_escape(picker.subtitle))
    end

    local y = LIST_TOP
    for i, row in ipairs(picker.rows) do
        if i == picker.cursor then
            ev[#ev + 1] = selection_box_event(y)
        end

        local box, num, text_x = chip_events(y, i)
        ev[#ev + 1] = box
        ev[#ev + 1] = num

        local bold = (i == picker.cursor) and "\\b1" or "\\b0"
        ev[#ev + 1] = string.format(
            "{\\an4\\pos(%d,%d)\\bord2\\shad0\\3c&H000000&\\fs20%s\\1c&HFFFFFF&}%s",
            text_x, y + math.floor(RH / 2), bold, ass_escape(row.display))

        y = y + RH
    end

    list_badge.data = table.concat(ev, "\n")
    list_badge:update()

    local row = picker.rows[picker.cursor]
    draw_params(row and row.display, row and row.detail)
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
        subtitle = "Apollo is best at fixing duplicate/repeated frames · 1-9 / Enter pick · Esc back",
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
        subtitle = "1-9 / Enter pick · Esc back to interpolation",
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
    local body = enh.enh_filter
    if interp and interp.fi and interp.fi ~= "" then
        body = body .. "," .. interp.fi
    end
    return body .. enh.enh_tail
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
    "topaz_menu_ab_enter", "topaz_menu_ab_kp_enter",
    "topaz_menu_tab", "topaz_menu_proceed_c", "topaz_menu_proceed_right",
    "topaz_menu_block_n", "topaz_menu_block_N",
    "topaz_menu_esc", "topaz_menu_bs",
}

function enable_menu_keys()
    mp.add_forced_key_binding("j", "topaz_menu_down", function() move_cursor(1) end)
    mp.add_forced_key_binding("DOWN", "topaz_menu_down2", function() move_cursor(1) end)
    mp.add_forced_key_binding("k", "topaz_menu_up", function() move_cursor(-1) end)
    mp.add_forced_key_binding("UP", "topaz_menu_up2", function() move_cursor(-1) end)
    for i = 1, 9 do
        mp.add_forced_key_binding(tostring(i), "topaz_menu_num_" .. i, function()
            select_number(i)
        end)
    end
    mp.add_forced_key_binding("r", "topaz_menu_render_r", render_cursor)
    mp.add_forced_key_binding("R", "topaz_menu_render_R", render_cursor)
    -- Enter flicks the shown preset against the original for A/B comparison.
    mp.add_forced_key_binding("ENTER", "topaz_menu_ab_enter", toggle_ab)
    mp.add_forced_key_binding("KP_ENTER", "topaz_menu_ab_kp_enter", toggle_ab)
    -- Tab cycles through the presets that already have a cached render.
    mp.add_forced_key_binding("TAB", "topaz_menu_tab", tab_cycle_cached)
    -- Proceed to the interpolation step with the highlighted preset.
    mp.add_forced_key_binding("c", "topaz_menu_proceed_c", choose_enhancement)
    mp.add_forced_key_binding("RIGHT", "topaz_menu_proceed_right", choose_enhancement)
    -- Swallow playlist-next/prev so preview stills don't get navigated away.
    mp.add_forced_key_binding("n", "topaz_menu_block_n", function() end)
    mp.add_forced_key_binding("N", "topaz_menu_block_N", function() end)
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
    for i = 1, 9 do
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
