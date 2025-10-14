local utils = require 'mp.utils'

local function toggle_panscan()
    local current = mp.get_property_number("panscan", 0)
    mp.set_property_number("panscan", current == 0 and 1 or 0)
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
    local items = {
        {
            title = "Toggle Pan & Scan",
            cmd = toggle_panscan,
        },
        {
            title = "Load All Directory Files",
            cmd = load_directory_files,
        },
        {
            title = "Cycle Aspect Ratio",
            cmd = function() mp.commandv("cycle-values", "video-aspect-override", "16:9", "4:3", "2.35:1", "-1") end,
        },
        {
            title = "Screenshot (With Subtitles)",
            cmd = function() mp.commandv("screenshot", "subtitles") end,
        },
        {
            title = "Screenshot (Without Subtitles)",
            cmd = function() mp.commandv("screenshot", "video") end,
        },
    }
    
    local list = {}
    for i, item in ipairs(items) do
        list[i] = {
            title = item.title,
            value = i,
        }
    end
    
    mp.commandv("script-message-to", "select", "show-and-select", 
                utils.format_json(list),
                utils.format_json({
                    on_select = mp.get_script_name() .. "/menu-select",
                    on_close = "",
                }))
    
    mp.register_script_message("menu-select", function(_, value)
        items[tonumber(value)].cmd()
    end)
end

mp.register_script_message("show-menu", show_menu)
mp.add_key_binding("M", "show-menu", show_menu)
