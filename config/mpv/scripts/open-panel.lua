local mp = require "mp"
local options = require "mp.options"
local utils = require "mp.utils"

local function load_chord_panel()
    local path = mp.find_config_file("script-modules/chord_panel.lua")
    if not path then
        local source = debug.getinfo(1, "S").source
        local script_dir = source and source:match("^@(.+)/[^/]+$")
        path = script_dir and (script_dir .. "/../script-modules/chord_panel.lua")
    end
    return assert(loadfile(assert(path, "chord_panel.lua not found")))()
end

local chord_panel = load_chord_panel()

local opts = {
    visuals_dir = "/Users/nom/Movies/Visuals",
    timeout = 10,
}

options.read_options(opts, "open-panel")

local KITTY_LAUNCH = "/Users/nom/.zsh/bin/kitty-launch"

local pending_restore = nil

local media_extensions = {
    mp4 = true, m4v = true, mkv = true, mov = true, webm = true, avi = true,
    flv = true, ts = true, m2ts = true, mpg = true, mpeg = true, hevc = true,
    mp3 = true, m4a = true, flac = true, wav = true, aiff = true, aif = true,
    opus = true, ogg = true, oga = true, wv = true,
    jpg = true, jpeg = true, png = true, gif = true, webp = true, heic = true,
}

local function basename(path)
    return path and path:match("([^/\\]+)$") or path
end

local function trim_trailing_slash(path)
    if path == "/" then
        return path
    end
    return (path or ""):gsub("/+$", "")
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

local function current_directory()
    local abs, err = absolute_media_path()
    if not abs then
        return nil, nil, err
    end

    local info = utils.file_info(abs)
    if not info then
        return nil, nil, "File no longer exists:\n" .. abs
    end

    if info.is_dir then
        return trim_trailing_slash(abs), abs
    end

    return trim_trailing_slash(utils.split_path(abs)), abs
end

local function parent_directory()
    local dir, target, err = current_directory()
    if not dir then
        return nil, nil, err
    end

    local parent = trim_trailing_slash(utils.split_path(dir))
    if parent == "" then
        parent = "/"
    end

    return parent, target
end

local function is_media_file(path)
    local ext = path and path:match("^.+%.([^./\\]+)$")
    return ext and media_extensions[ext:lower()] or false
end

local function list_media_files(dir)
    local files = {}
    local result = utils.subprocess({
        args = { "fd", "--type", "f", "--color", "never", ".", dir },
        cancellable = false,
    })

    if result.status == 0 then
        for file in result.stdout:gmatch("[^\r\n]+") do
            if is_media_file(file) then
                table.insert(files, file)
            end
        end
    else
        local entries = utils.readdir(dir, "files") or {}
        for _, entry in ipairs(entries) do
            local file = utils.join_path(dir, entry)
            if is_media_file(file) then
                table.insert(files, file)
            end
        end
    end

    table.sort(files, function(a, b)
        return a:lower() < b:lower()
    end)

    return files
end

local function load_directory(dir, target_path)
    local info = utils.file_info(dir)
    if not info or not info.is_dir then
        mp.osd_message("Directory not found:\n" .. dir, 3)
        return
    end

    local files = list_media_files(dir)
    if #files == 0 then
        mp.osd_message("No media files found:\n" .. dir, 3)
        return
    end

    local target_index = nil
    if target_path then
        for i, file in ipairs(files) do
            if file == target_path then
                target_index = i
                break
            end
        end
    end

    if target_index then
        pending_restore = {
            path = files[target_index],
            time_pos = mp.get_property_number("time-pos"),
            pause = mp.get_property_bool("pause"),
        }
        mp.set_property_bool("pause", true)
    else
        pending_restore = nil
    end

    mp.commandv("loadfile", files[1], "replace")
    for i = 2, #files do
        mp.commandv("loadfile", files[i], "append")
    end

    if target_index and target_index > 1 then
        mp.add_timeout(0, function()
            mp.commandv("playlist-play-index", tostring(target_index - 1))
        end)
    end

    mp.osd_message(string.format("Opened %s (%d items)", basename(dir) or dir, #files), 2)
end

local function open_current_directory()
    local dir, target, err = current_directory()
    if not dir then
        mp.osd_message(err, 3)
        return
    end
    load_directory(dir, target)
end

local function open_parent_directory()
    local dir, target, err = parent_directory()
    if not dir then
        mp.osd_message(err, 3)
        return
    end
    load_directory(dir, target)
end

local function open_in_kitty()
    local dir, _, err = current_directory()
    if not dir then
        mp.osd_message(err, 3)
        return
    end

    local result = utils.subprocess_detached({
        args = {
            KITTY_LAUNCH,
            "--cwd", dir,
            "--title", " mpv ",
        },
    })

    if result == false then
        mp.osd_message("Failed to open kitty", 2)
        return
    end

    mp.osd_message("Opened kitty", 1.5)
end

local function reveal_in_yazi()
    mp.commandv("script-binding", "reveal_in_yazi")
end

local function reveal_in_finder()
    mp.commandv("script-binding", "reveal_in_finder")
end

local function open_library()
    mp.commandv("script-binding", "load-all-media")
end

local function open_visuals()
    load_directory(opts.visuals_dir)
end

local function extract_url(value)
    if type(value) ~= "table" then
        return tostring(value or ""):match("https?://[^%s\"'<>]+")
    end
    for _, item in pairs(value) do
        local url = extract_url(item)
        if url then
            return url
        end
    end
    return nil
end

local function find_metadata_url()
    local preferred_keys = { "purl", "url", "webpageurl", "comment", "description", "synopsis" }
    local sources = {
        mp.get_property_native("metadata") or {},
        mp.get_property_native("filtered-metadata") or {},
    }

    for _, source in ipairs(sources) do
        local normalized = {}
        for key, value in pairs(source) do
            normalized[tostring(key):lower():gsub("[^%w]", "")] = value
        end
        for _, key in ipairs(preferred_keys) do
            local url = extract_url(normalized[key])
            if url then
                return url
            end
        end
        -- no url-ish key matched; take a url from any metadata value
        for _, value in pairs(source) do
            local url = extract_url(value)
            if url then
                return url
            end
        end
    end

    return nil
end

local function open_metadata_url()
    local url = find_metadata_url()
    if not url then
        mp.osd_message("No URL found in metadata", 2)
        return
    end

    local result = utils.subprocess_detached({ args = { "open", url } })
    if result == false then
        mp.osd_message("Failed to open URL", 2)
        return
    end

    mp.osd_message("Opened " .. url, 1.5)
end

local panel = chord_panel.new({
    name = "open-panel",
    title = "open",
    timeout = opts.timeout,
    actions = {
        { key = "d", label = "open current directory", fn = open_current_directory },
        { key = "p", label = "open parent directory", fn = open_parent_directory },
        { key = "t", label = "open in kitty", fn = open_in_kitty },
        { key = "y", label = "reveal in yazi", fn = reveal_in_yazi },
        { key = "f", label = "reveal in Finder", fn = reveal_in_finder },
        { key = "l", label = "open library", fn = open_library },
        { key = "v", label = "open visuals", fn = open_visuals },
        { key = "u", label = "open embedded URL", fn = open_metadata_url },
    },
})

mp.register_event("file-loaded", function()
    if not pending_restore then
        return
    end

    local abs = absolute_media_path()
    if abs ~= pending_restore.path then
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

local function run_action(fn)
    return function()
        panel:hide()
        fn()
    end
end

mp.add_key_binding(nil, "open_panel_show", function() panel:show() end)
mp.add_key_binding(nil, "open_panel_exit", function() panel:hide() end)
mp.add_key_binding(nil, "open_panel_current_directory", run_action(open_current_directory))
mp.add_key_binding(nil, "open_panel_parent_directory", run_action(open_parent_directory))
mp.add_key_binding(nil, "open_panel_kitty", run_action(open_in_kitty))
mp.add_key_binding(nil, "open_panel_yazi", run_action(reveal_in_yazi))
mp.add_key_binding(nil, "open_panel_finder", run_action(reveal_in_finder))
mp.add_key_binding(nil, "open_panel_library", run_action(open_library))
mp.add_key_binding(nil, "open_panel_visuals", run_action(open_visuals))
mp.add_key_binding(nil, "open_panel_url", run_action(open_metadata_url))
