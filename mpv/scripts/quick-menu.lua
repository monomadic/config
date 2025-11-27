--
-- mp.add_key_binding(nil, "show-menu", show_menu)
-- 
local utils = require 'mp.utils'
local input = require 'mp.input'

-- Track states for external scripts
local script_states = {
    auto_rotate = false,
    metadata = true  -- metadata starts ON by default
}

-- Track pause state to restore it after menu
-- local was_paused = false

local function toggle_panscan()
    mp.commandv("script-binding", "auto-panscan/toggle")
end

local function cycle_aspect_ratio()
    local aspects = {"16:9", "4:3", "2.35:1", "21:9", "1:1", "-1"}
    local aspect_names = {
        ["16:9"] = "16:9 (Widescreen)",
        ["4:3"] = "4:3 (Classic)",
        ["2.35:1"] = "2.35:1 (Cinema)",
        ["21:9"] = "21:9 (Ultrawide)",
        ["1:1"] = "1:1 (Square)",
        ["-1"] = "Auto"
    }
    
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
    mp.osd_message("Aspect Ratio: " .. aspect_names[next_aspect])
end

local function toggle_auto_rotate()
    script_states.auto_rotate = not script_states.auto_rotate
    mp.commandv("script-message-to", "auto-rotate", "toggle")
end

local function toggle_metadata()
    script_states.metadata = not script_states.metadata
    mp.commandv("script-message-to", "metadata", "toggle")
end

local function cycle_osd_level()
    local current = mp.get_property_number("osd-level", 1)
    local next_level = (current + 1) % 4
    mp.set_property_number("osd-level", next_level)
    
    local level_names = {
        [0] = "Off",
        [1] = "Seekbar Only",
        [2] = "File Info",
        [3] = "Full Info"
    }
    mp.osd_message("OSD Level: " .. level_names[next_level])
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

local function show_menu()
    -- Store current pause state and pause if playing
    -- was_paused = mp.get_property_bool("pause")
    mp.set_property_bool("pause", true)
    
    local panscan = mp.get_property_number("panscan", 0)
    local aspect = mp.get_property("video-aspect-override", "-1")
    local osd_level = mp.get_property_number("osd-level", 1)
    
    -- Format aspect ratio display
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
    
    local items = {
        "  Open Directory",
        "────────────────────",
        "Pan & Scan:      " .. (panscan > 0 and " ON" or "OFF"),
        "Aspect Ratio:    " .. (aspect_display[aspect] or aspect),
        "OSD Level:       " .. osd_display[osd_level],
        "────────────────────",
        "Auto Rotate:     " .. (script_states.auto_rotate and " ON" or "OFF"),
        "Metadata HUD:    " .. (script_states.metadata and " ON" or "OFF"),
    }
    
    local actions = {
        load_directory_files,
        nil,  -- Separator (no action)
        toggle_panscan,
        cycle_aspect_ratio,
        cycle_osd_level,
        nil,  -- Separator (no action)
        toggle_auto_rotate,
        toggle_metadata,
    }
    
    input.select({
        prompt = "Quick Menu",
        items = items,
        submit = function(index)
            -- Execute action if not a separator
            if index and actions[index] then
                actions[index]()
            end
            
            -- Restore pause state after menu closes
            mp.set_property_bool("pause", false)
        end,
        on_close = function()
            -- Also restore pause state if menu is cancelled (ESC)
            mp.set_property_bool("pause", false)
        end,
    })
end

mp.add_key_binding("Tab", "show-menu", show_menu)
