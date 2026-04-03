local utils = require 'mp.utils'

local pending_restore = nil

local function get_files_in_dir()
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

local function nav_dir(direction)
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
            local was_paused = mp.get_property_bool("pause")

            pending_restore = {
                filename = files[new_idx],
                pause = was_paused,
            }

            mp.set_property_bool("pause", true)
            mp.commandv("loadfile", utils.join_path(dir, files[1]), "replace")
            for i = 2, #files do
                local f = files[i]
                mp.commandv("loadfile", utils.join_path(dir, f), "append")
            end

            if new_idx > 1 then
                mp.add_timeout(0, function()
                    mp.commandv("playlist-play-index", tostring(new_idx - 1))
                end)
            end
        end
    end
end

mp.register_event("file-loaded", function()
    if not pending_restore then return end
    if mp.get_property("filename") ~= pending_restore.filename then
        return
    end

    local restore = pending_restore
    pending_restore = nil

    if restore.pause ~= nil then
        mp.set_property_bool("pause", restore.pause)
    end
end)

mp.add_key_binding(nil, "next-file-dir", function() nav_dir(1) end)
mp.add_key_binding(nil, "prev-file-dir", function() nav_dir(-1) end)
