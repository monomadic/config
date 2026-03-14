local utils = require 'mp.utils'

function get_files_in_dir()
    local path = mp.get_property("path")
    if not path then return nil end
    
    local dir = utils.split_path(path)
    local files = utils.readdir(dir, "files")
    if not files then return nil end
    
    local media_exts = {mkv=true, mp4=true, avi=true, webm=true, flv=true, 
                        mov=true, wmv=true, m4v=true, mpg=true, mpeg=true}
    local media_files = {}
    for _, f in ipairs(files) do
        local ext = f:match("^.+%.(.+)$")
        if ext and media_exts[ext:lower()] then
            table.insert(media_files, f)
        end
    end
    table.sort(media_files)
    
    return dir, media_files
end

function nav_dir(direction)
    local dir, files = get_files_in_dir()
    if not files then return end
    
    local current = mp.get_property("filename")
    local idx = nil
    for i, f in ipairs(files) do
        if f == current then idx = i break end
    end
    
    if idx then
        local new_idx = idx + direction
        if new_idx >= 1 and new_idx <= #files then
            -- Clear and rebuild playlist with all directory files
            mp.commandv("playlist-clear")
            for i, f in ipairs(files) do
                mp.commandv("loadfile", utils.join_path(dir, f), "append")
            end
            -- Jump to the target file (0-indexed)
            mp.set_property_number("playlist-pos", new_idx - 1)
        end
    end
end

mp.add_key_binding(">", "next-file-dir", function() nav_dir(1) end)
mp.add_key_binding("<", "prev-file-dir", function() nav_dir(-1) end)
