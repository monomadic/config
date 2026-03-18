local utils = require 'mp.utils'
local input = require 'mp.input'

local state = {
    metadata = true,
    auto_landscape = false,
    auto_jump = false,
    auto_seek = false,
    progress = true,
    stats = false,
    jump_delay = nil,
}

local aspect_display = {
    ["16:9"] = "16:9",
    ["4:3"] = "4:3",
    ["2.35:1"] = "2.35:1",
    ["21:9"] = "21:9",
    ["1:1"] = "1:1",
    ["-1"] = "Auto"
}

local osd_display = {
    [0] = "Off",
    [1] = "Seekbar Only",
    [2] = "File Info",
    [3] = "Full Info"
}

local function on_off(v)
    return v and "ON" or "OFF"
end

local function format_delay(seconds)
    local d = tonumber(seconds)
    if not d or d <= 0 then
        return "manual"
    end
    if d >= 60 then
        return string.format("%dm", math.floor(d / 60 + 0.5))
    end
    return string.format("%ds", math.floor(d + 0.5))
end

local function refresh_state()
    mp.commandv("script-message", "randjump-query")
    mp.commandv("script-message", "auto-landscape-query")
    mp.commandv("script-message", "metadata-query")
    mp.commandv("script-message", "progress-bar-query")
    mp.commandv("script-message", "realtime-stats-query")
end

local function toggle_panscan()
    mp.command("script-binding toggle-pan-scan")
end

local function cycle_aspect_ratio()
    local aspects = {"16:9", "4:3", "2.35:1", "21:9", "1:1", "-1"}
    
    local current = mp.get_property("video-aspect-override", "-1")
    local next_index = 1
    
    for i, aspect in ipairs(aspects) do
        if aspect == current then
            next_index = (i % #aspects) + 1
            break
        end
    end
    
    local next_aspect = aspects[next_index]
    mp.set_property("video-aspect-override", next_aspect)
    mp.osd_message("Aspect Ratio: " .. (aspect_display[next_aspect] or next_aspect))
end

local function toggle_auto_landscape()
    mp.commandv("script-message-to", "auto-landscape", "toggle")
end

local function toggle_metadata()
    mp.commandv("script-message-to", "metadata", "toggle")
end

local function cycle_osd_level()
    local current = mp.get_property_number("osd-level", 1)
    local next_level = (current + 1) % 4
    mp.set_property_number("osd-level", next_level)
    
    mp.osd_message("OSD Level: " .. osd_display[next_level])
end

local function load_directory_files()
    local path = mp.get_property("path")
    if not path then return end
    
    local dir = utils.split_path(path)
    local files = utils.readdir(dir, "files")
    
    if not files then
        mp.osd_message("Could not read directory")
        return
    end
    
    -- Filter for media files
    local media_extensions = {
        mp4=true, mkv=true, avi=true, mov=true, webm=true, flv=true,
        mp3=true, flac=true, opus=true, ogg=true, m4a=true, wav=true,
    }
    
    local media_files = {}
    for _, file in ipairs(files) do
        local ext = file:match("^.+%.(.+)$")
        if ext and media_extensions[ext:lower()] then
            table.insert(media_files, utils.join_path(dir, file))
        end
    end
    
    table.sort(media_files)
    
    -- Clear playlist and add files
    mp.commandv("playlist-clear")
    for _, file in ipairs(media_files) do
        mp.commandv("loadfile", file, "append")
    end
    
    mp.osd_message(string.format("Loaded %d files from directory", #media_files))
end

local function toggle_progress_bar()
    mp.commandv("script-binding", "progress-bar-minimal/toggle-progress")
end

local function toggle_stats()
    mp.commandv("script-binding", "toggle_stats")
end

local function toggle_auto_jump()
    mp.commandv("script-binding", "toggle_auto_jump")
end

local function toggle_auto_seek()
    mp.commandv("script-binding", "toggle_auto_seek_within_file")
end

local function cycle_jump_delay()
    local options = {
        { label = "1s", action = "set_delay_1" },
        { label = "3s", action = "set_delay_3" },
        { label = "5s", action = "set_delay_5" },
        { label = "10s", action = "set_delay_10" },
        { label = "20s", action = "set_delay_20" },
    }
    local current = format_delay(state.jump_delay)
    local next_idx = 1

    for i, item in ipairs(options) do
        if item.label == current then
            next_idx = (i % #options) + 1
            break
        end
    end

    mp.commandv("script-binding", options[next_idx].action)
end

local function with_resume(fn)
    return function()
        fn()
        mp.add_timeout(0, refresh_state)
    end
end

mp.register_script_message("metadata-state", function(enabled)
    state.metadata = (enabled == "yes")
end)

mp.register_script_message("auto_landscape_broadcast", function(enabled)
    state.auto_landscape = (enabled == "yes")
end)

mp.register_script_message("keybar-randjump-state", function(auto_jump, auto_seek, delay)
    state.auto_jump = (auto_jump == "1" or auto_jump == "true")
    state.auto_seek = (auto_seek == "1" or auto_seek == "true")
    state.jump_delay = delay
end)

mp.register_script_message("progress_bar_state", function(enabled)
    state.progress = (enabled == "yes")
end)

mp.register_script_message("realtime-stats-state", function(enabled)
    state.stats = (enabled == "yes")
end)

local function show_menu()
    refresh_state()
    local panscan = mp.get_property_number("panscan", 0)
    local aspect = mp.get_property("video-aspect-override", "-1")
    local osd_level = mp.get_property_number("osd-level", 1)
    
    local items = {
        "Open Dir",
        "Load Library",
        "────────────────────",
        "Pan+Scan:      " .. on_off(panscan > 0),
        "Aspect:        " .. (aspect_display[aspect] or aspect),
        "OSD:           " .. osd_display[osd_level],
        "Progress:      " .. on_off(state.progress),
        "Stats:         " .. on_off(state.stats),
        "────────────────────",
        "Auto-Land:     " .. on_off(state.auto_landscape),
        "Metadata:      " .. on_off(state.metadata),
        "Auto Jump:     " .. on_off(state.auto_jump),
        "Auto Seek:     " .. on_off(state.auto_seek),
        "Jump Delay:    " .. format_delay(state.jump_delay),
        "────────────────────",
        "Sort by mtime",
        "Expand Dirs",
        "Expand+Sort+Play Newest",
    }
    
    local actions = {
        with_resume(load_directory_files),
        with_resume(function()
            mp.commandv("script-binding", "load-all-media")
        end),
        nil,  -- Separator (no action)
        with_resume(toggle_panscan),
        with_resume(cycle_aspect_ratio),
        with_resume(cycle_osd_level),
        with_resume(toggle_progress_bar),
        with_resume(toggle_stats),
        nil,  -- Separator (no action)
        with_resume(toggle_auto_landscape),
        with_resume(toggle_metadata),
        with_resume(toggle_auto_jump),
        with_resume(toggle_auto_seek),
        with_resume(cycle_jump_delay),
        nil,  -- Separator (no action)
        with_resume(function()
            mp.commandv("script-binding", "sort_playlist_by_mtime")
        end),
        with_resume(function()
            mp.commandv("script-binding", "expand_playlist_dirs")
        end),
        with_resume(function()
            mp.commandv("script-binding", "expand_sort_play_newest")
        end),
    }
    
    input.select({
        prompt = "Quick Menu",
        items = items,
        submit = function(index)
            if index and actions[index] then
                actions[index]()
            end
        end,
    })
end

mp.add_key_binding(nil, "show-menu", show_menu)
