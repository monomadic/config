local mp = require "mp"
local assdraw = require "mp.assdraw"
local options = require "mp.options"
local utils = require "mp.utils"

local opts = {
    visuals_dir = "/Users/nom/Movies/Visuals",
    timeout = 10,
}

options.read_options(opts, "open-panel")

local INPUT_SECTION = "open-panel"
local KITTY_LAUNCH = "/Users/nom/.zsh/bin/kitty-launch"

local overlay = mp.create_osd_overlay("ass-events")
overlay.z = 220

local active = false
local hide_timer = nil
local pending_restore = nil

local media_extensions = {
    mp4 = true, m4v = true, mkv = true, mov = true, webm = true, avi = true,
    flv = true, ts = true, m2ts = true, mpg = true, mpeg = true, hevc = true,
    mp3 = true, m4a = true, flac = true, wav = true, aiff = true, aif = true,
    opus = true, ogg = true, oga = true, wv = true,
    jpg = true, jpeg = true, png = true, gif = true, webp = true, heic = true,
}

local actions = {
    { key = "d", label = "open current directory" },
    { key = "p", label = "open parent directory" },
    { key = "t", label = "open in kitty" },
    { key = "y", label = "reveal in yazi" },
    { key = "f", label = "reveal in Finder" },
    { key = "l", label = "open library" },
    { key = "v", label = "open visuals" },
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

local function stop_timer()
    if hide_timer then
        hide_timer:kill()
        hide_timer = nil
    end
end

local function clear_panel()
    stop_timer()
    overlay:remove()
    if active then
        mp.commandv("disable-section", INPUT_SECTION)
    end
    active = false
end

local function render_panel()
    local dim = mp.get_property_native("osd-dimensions")
    if not dim or not dim.w or dim.w <= 0 or not dim.h or dim.h <= 0 then
        local lines = { "open" }
        for _, item in ipairs(actions) do
            table.insert(lines, item.key .. " " .. item.label)
        end
        table.insert(lines, "ESC cancel")
        mp.osd_message(table.concat(lines, "\n"), opts.timeout)
        return
    end

    local w, h = dim.w, dim.h
    local panel_w = math.min(560, math.floor(w * 0.76))
    local row_h = 30
    local panel_h = 64 + (#actions * row_h) + 20
    local x = math.floor((w - panel_w) / 2)
    local y = math.floor(math.max(40, h * 0.15))
    local ass = assdraw.ass_new()

    ass:new_event()
    ass:append(("{\\an7\\pos(%d,%d)\\bord0\\shad0\\1c&H111111&\\alpha&H20&}"):format(x, y))
    ass:draw_start()
    ass:rect_cw(0, 0, panel_w, panel_h)
    ass:draw_stop()

    ass:new_event()
    ass:append(("{\\an7\\pos(%d,%d)\\bord0\\shad0\\fs24\\1c&H00FFFF&}open\\N"):format(x + 24, y + 18))
    ass:append("{\\fs16\\1c&HAAAAAA&}press a key")

    for i, item in ipairs(actions) do
        local row_y = y + 58 + ((i - 1) * row_h)
        ass:new_event()
        ass:append(("{\\an7\\pos(%d,%d)\\bord0\\shad0\\fs20\\1c&H9CFF00&}%s"):format(x + 26, row_y, item.key))
        ass:append(("{\\1c&HFFFFFF&}  %s"):format(item.label))
    end

    ass:new_event()
    ass:append(("{\\an7\\pos(%d,%d)\\bord0\\shad0\\fs16\\1c&H777777&}ESC cancel"):format(x + 26, y + panel_h - 28))

    overlay.data = ass.text
    overlay.res_x = w
    overlay.res_y = h
    overlay:update()
end

local function show_panel()
    active = true
    mp.commandv("enable-section", INPUT_SECTION, "exclusive")
    render_panel()
    stop_timer()
    hide_timer = mp.add_timeout(tonumber(opts.timeout) or 10, clear_panel)
end

local function run_action(fn)
    clear_panel()
    fn()
end

local function open_current_directory()
    run_action(function()
        local dir, target, err = current_directory()
        if not dir then
            mp.osd_message(err, 3)
            return
        end
        load_directory(dir, target)
    end)
end

local function open_parent_directory()
    run_action(function()
        local dir, target, err = parent_directory()
        if not dir then
            mp.osd_message(err, 3)
            return
        end
        load_directory(dir, target)
    end)
end

local function open_in_kitty()
    run_action(function()
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
    end)
end

local function reveal_in_yazi()
    run_action(function()
        mp.commandv("script-binding", "reveal_in_yazi")
    end)
end

local function reveal_in_finder()
    run_action(function()
        mp.commandv("script-binding", "reveal_in_finder")
    end)
end

local function open_library()
    run_action(function()
        mp.commandv("script-binding", "load-all-media")
    end)
end

local function open_visuals()
    run_action(function()
        load_directory(opts.visuals_dir)
    end)
end

mp.observe_property("osd-dimensions", "native", function()
    if active then
        render_panel()
    end
end)

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

mp.register_event("shutdown", stop_timer)

mp.add_key_binding(nil, "open_panel_show", show_panel)
mp.add_key_binding(nil, "open_panel_exit", clear_panel)
mp.add_key_binding(nil, "open_panel_current_directory", open_current_directory)
mp.add_key_binding(nil, "open_panel_parent_directory", open_parent_directory)
mp.add_key_binding(nil, "open_panel_kitty", open_in_kitty)
mp.add_key_binding(nil, "open_panel_yazi", reveal_in_yazi)
mp.add_key_binding(nil, "open_panel_finder", reveal_in_finder)
mp.add_key_binding(nil, "open_panel_library", open_library)
mp.add_key_binding(nil, "open_panel_visuals", open_visuals)
