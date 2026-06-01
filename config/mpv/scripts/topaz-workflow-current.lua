local utils = require "mp.utils"
local input = require "mp.input"

local kitty_launch = "/Users/nom/.zsh/bin/kitty-launch"
local topaz_workflow = "/Users/nom/.zsh/bin/topaz-workflow"
local topaz_run = "/Users/nom/.zsh/bin/topaz-run"
local topaz_preview_frame = "/Users/nom/.zsh/bin/topaz-preview-frame"
local preset_catalog = "/Users/nom/config/config/zsh/bin/topaz-preset-catalog.zsh"

local preview_state = nil
local pending_restore = nil

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

local function load_transform_rows()
    local stdout, err = catalog_stdout("topaz_transform_preset_rows")
    if not stdout then
        return nil, err
    end

    local rows = {}
    for line in stdout:gmatch("[^\r\n]+") do
        local fields = split_tsv(line)
        if #fields >= 4 then
            table.insert(rows, {
                display = fields[1],
                categories = fields[2] or "",
                slug = fields[3] or "",
                filter = fields[4] or "",
                metadata = fields[5] or "",
            })
        end
    end

    if #rows == 0 then
        return nil, "No Topaz transform presets found"
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
    return "Topaz preview\nEnter/e encode whole video\nEsc return to source"
end

local function remove_preview_keys()
    mp.remove_key_binding("topaz_mpv_encode_e")
    mp.remove_key_binding("topaz_mpv_encode_enter")
    mp.remove_key_binding("topaz_mpv_encode_kp_enter")
    mp.remove_key_binding("topaz_mpv_restore_esc")
    mp.remove_key_binding("topaz_mpv_restore_bs")
end

local function restore_source(message)
    if not preview_state then
        return
    end

    local state = preview_state
    preview_state = nil
    remove_preview_keys()

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
        if state.preview_index and count > state.preview_index then
            mp.add_timeout(0.1, function()
                local current_count = mp.get_property_number("playlist-count", 0)
                if current_count > state.preview_index then
                    mp.commandv("playlist-remove", tostring(state.preview_index))
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

local function topaz_preset_label(transform, output)
    return transform.display .. " - " .. output.display
end

local function launch_encode()
    if not preview_state then
        return
    end

    local state = preview_state
    local transform = state.transform
    local output = state.output
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
        "--preset_name", topaz_preset_label(transform, output),
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

local function enable_preview_keys()
    mp.add_forced_key_binding("e", "topaz_mpv_encode_e", launch_encode)
    mp.add_forced_key_binding("ENTER", "topaz_mpv_encode_enter", launch_encode)
    mp.add_forced_key_binding("KP_ENTER", "topaz_mpv_encode_kp_enter", launch_encode)
    mp.add_forced_key_binding("ESC", "topaz_mpv_restore_esc", function()
        restore_source("Topaz preview closed")
    end)
    mp.add_forced_key_binding("BS", "topaz_mpv_restore_bs", function()
        restore_source("Topaz preview closed")
    end)
end

local function load_preview_image(image_path, source, transform, output)
    if preview_state then
        restore_source()
    end

    local playlist_count = mp.get_property_number("playlist-count", 0)
    local original_pos = mp.get_property_number("playlist-pos")

    preview_state = {
        source = source,
        source_name = basename(source),
        transform = transform,
        output = output,
        time_pos = mp.get_property_number("time-pos", 0) or 0,
        pause = mp.get_property_bool("pause"),
        original_pos = original_pos,
        preview_index = playlist_count,
        old_image_display_duration = mp.get_property("image-display-duration"),
    }

    mp.set_property("image-display-duration", "inf")
    mp.commandv("loadfile", image_path, "append")
    mp.add_timeout(0.05, function()
        if not preview_state then
            return
        end
        mp.commandv("playlist-play-index", tostring(preview_state.preview_index))
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

local function run_preview(source, transform, output)
    local time_pos = mp.get_property_number("time-pos", 0) or 0
    local preset_name = topaz_preset_label(transform, output)
    local time_arg = string.format("%.3f", time_pos)

    mp.osd_message("Rendering Topaz preview frame...", 3)

    mp.command_native_async({
        name = "subprocess",
        args = {
            topaz_preview_frame,
            "--input", source,
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
        if not success or not result or result.status ~= 0 then
            local stderr = result and trim(result.stderr) or ""
            local detail = stderr ~= "" and stderr or tostring(error or "unknown error")
            mp.osd_message("Topaz preview failed", 3)
            mp.msg.error("Topaz preview failed: " .. detail)
            return
        end

        local paths = parse_preview_paths(result.stdout)
        local image = paths.compare or paths.topaz
        if not image or image == "" then
            mp.osd_message("Topaz preview did not return an image", 3)
            mp.msg.error("Topaz preview stdout:\n" .. (result.stdout or ""))
            return
        end

        load_preview_image(image, source, transform, output)
    end)
end

local function show_output_menu(source, transform)
    local outputs, err = load_output_rows()
    if not outputs then
        mp.osd_message(err, 3)
        return
    end

    input.select({
        prompt = "Topaz Output",
        items = select_items(outputs),
        submit = function(index)
            if not index then
                return
            end
            run_preview(source, transform, outputs[index])
        end,
    })
end

local function show_transform_menu()
    local source, err = check_current_file()
    if not source then
        mp.osd_message(err, 3)
        return
    end

    local transforms
    transforms, err = load_transform_rows()
    if not transforms then
        mp.osd_message(err, 3)
        return
    end

    input.select({
        prompt = "Topaz Transform",
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
                show_output_menu(source, transform)
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

mp.register_event("shutdown", remove_preview_keys)

mp.add_key_binding(nil, "topaz_workflow_current_file", show_transform_menu)
mp.add_key_binding(nil, "topaz_workflow_external_file", topaz_workflow_external_file)
