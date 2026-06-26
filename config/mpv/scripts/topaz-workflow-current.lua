local utils = require "mp.utils"
local input = require "mp.input"

local kitty_launch = "/Users/nom/.zsh/bin/kitty-launch"
local topaz_workflow = "/Users/nom/.zsh/bin/topaz-workflow"
local topaz_run = "/Users/nom/.zsh/bin/topaz-run"
local topaz_preview_frame = "/Users/nom/.zsh/bin/topaz-preview-frame"
local preset_catalog = "/Users/nom/config/config/zsh/bin/topaz-preset-catalog.zsh"

local preview_state = nil
local pending_restore = nil
local ab_badge = mp.create_osd_overlay("ass-events")
ab_badge.z = 30
local encode_badge = mp.create_osd_overlay("ass-events")
encode_badge.z = 31
local encode_timer = nil
local encode_start = nil

local function shell_quote(value)
    return "'" .. tostring(value):gsub("'", "'\\''") .. "'"
end

local function trim(value)
    return (value or ""):gsub("^%s+", ""):gsub("%s+$", "")
end

local function strip_videoai(value)
    return (value or ""):gsub("^videoai=", "")
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

-- Detected source video properties, used to adapt the transform menu.
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
        -- long-edge bands: <=2560 treated as "1080p-ish", >2560 as "4K"
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

local function render_filter(template, target_w, target_h)
    local up = string.format("scale=0:w=%d:h=%d", target_w, target_h)
    local tail = string.format(",scale=w=%d:h=%d:flags=lanczos:threads=0", target_w, target_h)
    local out = template:gsub("@4K@", up):gsub("@4KTAIL@", tail)
    return out
end

local function res_band_ok(band, profile)
    if band == "any" or profile.show_all then
        return true
    end
    if band == "upto1080" then
        return not profile.is_4k
    end
    if band == "atleast4k" then
        return profile.is_4k == true
    end
    return true
end

local function fps_band_ok(band, profile)
    if band == "any" or profile.show_all then
        return true
    end

    local fps = profile.fps
    if not fps or fps <= 0 then
        return true -- unknown fps: don't hide interpolation options
    end
    if band == "under60" then
        return fps < 59.5
    end
    if band == "atleast60" then
        return fps >= 59.5
    end
    return true
end

local function profile_summary(profile)
    if profile.show_all or not profile.long_edge then
        return "source unknown — all presets"
    end

    local res = profile.is_4k and "4K → cleanup/sharpen 1x" or "≤1080p → upscale to 4K"
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

local function load_transform_rows(profile)
    local stdout, err = catalog_stdout("topaz_adaptive_transform_rows")
    if not stdout then
        return nil, err
    end

    local target_w, target_h = target_4k_dims(profile.width, profile.height)

    local rows = {}
    for line in stdout:gmatch("[^\r\n]+") do
        local fields = split_tsv(line)
        -- res_band, fps_band, display, categories, slug, filter, metadata
        if #fields >= 6 then
            local res_band = fields[1] or "any"
            local fps_band = fields[2] or "any"
            if res_band_ok(res_band, profile) and fps_band_ok(fps_band, profile) then
                table.insert(rows, {
                    display = fields[3],
                    categories = fields[4] or "",
                    slug = fields[5] or "",
                    filter = render_filter(fields[6] or "", target_w, target_h),
                    metadata = fields[7] or "",
                })
            end
        end
    end

    if #rows == 0 then
        return nil, "No Topaz presets matched this source"
    end

    return rows
end

local function load_output_rows()
    local stdout, err = catalog_stdout("topaz_output_profile_rows")
    if not stdout then
        return nil, err
    end

    local rows = {}
    for line in stdout:gmatch("[^\r\n]+") do
        local fields = split_tsv(line)
        if #fields >= 4 then
            table.insert(rows, {
                display = fields[1],
                slug = fields[2] or "",
                output_ext = fields[3] or "",
                video_args = fields[4] or "",
            })
        end
    end

    if #rows == 0 then
        return nil, "No Topaz output profiles found"
    end

    return rows
end

local function select_items(rows)
    local items = {}
    for _, row in ipairs(rows) do
        table.insert(items, row.display)
    end
    return items
end

local function preview_key_message()
    return "Topaz A/B preview in this mpv window\na / TAB: flip A (original) / B (Topaz)  ·  1: A  2: B\nEnter/e: choose output + encode  ·  Esc/BS: return to source"
end

local function ab_badge_text()
    if not preview_state then
        return ""
    end

    local active = "{\\1c&H66FF66&\\b1}"
    local idle = "{\\1c&HBBBBBB&\\b0}"
    local hint = "{\\1c&HBBBBBB&\\b0\\fs20}"
    local a_tag = preview_state.side == "a" and active or idle
    local b_tag = preview_state.side == "b" and active or idle

    return string.format(
        "{\\an7\\pos(24,24)\\fs30\\bord2\\3c&H000000&\\1c&HFFFFFF&\\b1}A/B MODE   "
            .. "%s[1] A · original   %s[2] B · Topaz\\N"
            .. "%sa / TAB flip   ·   e / Enter encode   ·   Esc close",
        a_tag,
        b_tag,
        hint
    )
end

local function update_ab_badge()
    if not preview_state then
        ab_badge:remove()
        return
    end

    ab_badge.res_x = 1280
    ab_badge.res_y = 720
    ab_badge.data = ab_badge_text()
    ab_badge:update()
end

local function show_preview_side(side)
    if not preview_state then
        return
    end

    preview_state.side = side
    local index = side == "a" and preview_state.index_a or preview_state.index_b
    if index then
        mp.commandv("playlist-play-index", tostring(index))
    end
    update_ab_badge()
end

local function flip_preview_side()
    if not preview_state then
        return
    end

    show_preview_side(preview_state.side == "a" and "b" or "a")
end

local function remove_preview_keys()
    mp.remove_key_binding("topaz_mpv_encode_e")
    mp.remove_key_binding("topaz_mpv_encode_enter")
    mp.remove_key_binding("topaz_mpv_encode_kp_enter")
    mp.remove_key_binding("topaz_mpv_restore_esc")
    mp.remove_key_binding("topaz_mpv_restore_bs")
    mp.remove_key_binding("topaz_mpv_flip_a")
    mp.remove_key_binding("topaz_mpv_flip_tab")
    mp.remove_key_binding("topaz_mpv_flip_underscore")
    mp.remove_key_binding("topaz_mpv_show_a")
    mp.remove_key_binding("topaz_mpv_show_b")
end

local function restore_source(message)
    if not preview_state then
        return
    end

    local state = preview_state
    preview_state = nil
    remove_preview_keys()
    ab_badge:remove()

    if state.old_image_display_duration then
        mp.set_property("image-display-duration", state.old_image_display_duration)
    end

    pending_restore = {
        time_pos = state.time_pos,
        pause = state.pause,
    }

    local count = mp.get_property_number("playlist-count", 0)
    if state.original_pos and count > state.original_pos then
        mp.commandv("playlist-play-index", tostring(state.original_pos))
        if state.preview_first and state.preview_count then
            mp.add_timeout(0.1, function()
                local current_count = mp.get_property_number("playlist-count", 0)
                for index = state.preview_first + state.preview_count - 1, state.preview_first, -1 do
                    if index >= 0 and current_count > index then
                        mp.commandv("playlist-remove", tostring(index))
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

local function topaz_encode_label(transform, output)
    return transform.display .. " - " .. output.display
end

local enable_preview_keys

local function start_encode(output)
    if not preview_state then
        return
    end

    local state = preview_state
    local transform = state.transform
    local directory = utils.split_path(state.source)
    local metadata = transform.metadata
    if metadata == "" then
        metadata = "videoai=" .. transform.display
    end

    local args = {
        kitty_launch,
        "--tab",
        "--hold",
        "--cwd", directory,
        "--title", " topaz encode ",
        "--",
        topaz_run,
        "--preset_name", topaz_encode_label(transform, output),
        "--filter_complex", transform.filter,
        "--output_ext", output.output_ext,
        "--video_args", output.video_args,
        "--metadata", metadata,
        "--",
        state.source,
    }

    local result = utils.subprocess_detached({ args = args })
    if result == false then
        mp.osd_message("Topaz encode launch failed", 2)
        mp.msg.error("Failed to launch Topaz encode for: " .. state.source)
        return
    end

    restore_source("Topaz encode started")
end

local function choose_encode_output()
    if not preview_state then
        return
    end

    local outputs, err = load_output_rows()
    if not outputs then
        mp.osd_message(err, 3)
        return
    end

    remove_preview_keys()
    input.select({
        prompt = "Encode Output Format",
        items = select_items(outputs),
        submit = function(index)
            if not index then
                enable_preview_keys()
                mp.osd_message(preview_key_message(), 4)
                return
            end
            start_encode(outputs[index])
        end,
    })
end

function enable_preview_keys()
    mp.add_forced_key_binding("e", "topaz_mpv_encode_e", choose_encode_output)
    mp.add_forced_key_binding("ENTER", "topaz_mpv_encode_enter", choose_encode_output)
    mp.add_forced_key_binding("KP_ENTER", "topaz_mpv_encode_kp_enter", choose_encode_output)
    mp.add_forced_key_binding("ESC", "topaz_mpv_restore_esc", function()
        restore_source("Topaz preview closed")
    end)
    mp.add_forced_key_binding("BS", "topaz_mpv_restore_bs", function()
        restore_source("Topaz preview closed")
    end)
    mp.add_forced_key_binding("a", "topaz_mpv_flip_a", flip_preview_side)
    mp.add_forced_key_binding("TAB", "topaz_mpv_flip_tab", flip_preview_side)
    mp.add_forced_key_binding("_", "topaz_mpv_flip_underscore", flip_preview_side)
    mp.add_forced_key_binding("1", "topaz_mpv_show_a", function()
        show_preview_side("a")
    end)
    mp.add_forced_key_binding("2", "topaz_mpv_show_b", function()
        show_preview_side("b")
    end)
end

local function load_preview_pair(original_path, topaz_path, context, transform)
    if preview_state then
        restore_source()
    end

    local playlist_count = mp.get_property_number("playlist-count", 0)
    local original_pos = mp.get_property_number("playlist-pos")

    preview_state = {
        source = context.source,
        source_name = basename(context.source),
        transform = transform,
        time_pos = context.time_pos,
        pause = true,
        original_pos = original_pos,
        preview_first = playlist_count,
        preview_count = 2,
        index_a = playlist_count,
        index_b = playlist_count + 1,
        side = "b",
        old_image_display_duration = mp.get_property("image-display-duration"),
    }

    mp.set_property("image-display-duration", "inf")
    mp.commandv("loadfile", original_path, "append")
    mp.commandv("loadfile", topaz_path, "append")
    mp.add_timeout(0.05, function()
        if not preview_state then
            return
        end
        show_preview_side(preview_state.side)
        enable_preview_keys()
        mp.osd_message(preview_key_message(), 6)
    end)
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
        label,
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

local function run_preview(context, transform)
    local preset_name = transform.display
    local time_arg = string.format("%.3f", context.time_pos)

    start_encode_badge(string.format("%s  @ %s", preset_name, format_time(context.time_pos)))

    mp.command_native_async({
        name = "subprocess",
        args = {
            topaz_preview_frame,
            "--input", context.source,
            "--preset-name", preset_name,
            "--preset-flag", "--filter_complex",
            "--filter", transform.filter,
            "--time", time_arg,
            "--no-open",
            "--print-paths",
        },
        playback_only = false,
        capture_stdout = true,
        capture_stderr = true,
    }, function(success, result, error)
        stop_encode_badge()
        if not success or not result or result.status ~= 0 then
            local stderr = result and trim(result.stderr) or ""
            local detail = stderr ~= "" and stderr or tostring(error or "unknown error")
            mp.osd_message("Topaz preview failed", 3)
            mp.msg.error("Topaz preview failed: " .. detail)
            return
        end

        local paths = parse_preview_paths(result.stdout)
        local topaz_image = paths.topaz or paths.compare
        if not topaz_image or topaz_image == "" then
            mp.osd_message("Topaz preview did not return an image", 3)
            mp.msg.error("Topaz preview stdout:\n" .. (result.stdout or ""))
            return
        end

        local original_image = paths.original
        if not original_image or original_image == "" then
            original_image = topaz_image
        end

        load_preview_pair(original_image, topaz_image, context, transform)
    end)
end

local function show_transform_menu()
    local source, err = check_current_file()
    if not source then
        mp.osd_message(err, 3)
        return
    end

    mp.set_property_bool("pause", true)

    local profile = source_profile()

    local context = {
        source = source,
        time_pos = mp.get_property_number("time-pos", 0) or 0,
    }

    local transforms
    transforms, err = load_transform_rows(profile)
    if not transforms then
        mp.osd_message(err, 3)
        return
    end

    input.select({
        prompt = string.format(
            "Topaz @ %s  [%s]",
            format_time(context.time_pos),
            profile_summary(profile)
        ),
        items = select_items(transforms),
        submit = function(index)
            if not index then
                return
            end

            local transform = transforms[index]
            local summary = strip_videoai(transform.metadata)
            if summary ~= "" then
                mp.osd_message(summary, 4)
            end
            mp.add_timeout(0, function()
                run_preview(context, transform)
            end)
        end,
    })
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
    remove_preview_keys()
    ab_badge:remove()
    if encode_timer then
        encode_timer:kill()
        encode_timer = nil
    end
    encode_badge:remove()
end)

mp.add_key_binding(nil, "topaz_workflow_current_file", show_transform_menu)
mp.add_key_binding(nil, "topaz_workflow_external_file", topaz_workflow_external_file)
