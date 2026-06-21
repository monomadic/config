local utils = require "mp.utils"

local kitty_launch = "/Users/nom/.zsh/bin/kitty-launch"
local rating_command = "/Users/nom/.zsh/bin/media-set-rating"

local function absolute_media_path()
    local media_path = mp.get_property("path")

    if not media_path or media_path == "" then
        return nil, "No media loaded"
    end

    local proto = mp.get_property("protocol") or ""
    if proto ~= "" and proto ~= "file" then
        return nil, "Not a local file (" .. proto .. ")"
    end

    return utils.join_path(mp.get_property("working-directory") or "", media_path)
end

local function set_rating_current_file()
    local abs, err = absolute_media_path()
    if not abs then
        mp.osd_message(err, 2)
        return
    end

    local info = utils.file_info(abs)
    if not info then
        mp.osd_message("File no longer exists:\n" .. abs, 3)
        mp.msg.warn("Set rating failed, file missing: " .. abs)
        return
    end

    local directory = utils.split_path(abs)
    local result = utils.subprocess_detached({
        args = {
            kitty_launch,
            "--tab",
            "--cwd", directory,
            "--title", " set rating ",
            "--",
            rating_command, abs,
        },
    })

    if result == false then
        mp.osd_message("Set rating launch failed", 2)
        mp.msg.error("Failed to launch set rating for: " .. abs)
        return
    end

    mp.osd_message("Opening set rating", 1.5)
end

mp.add_key_binding(nil, "set_rating_current_file", set_rating_current_file)
