local utils = require "mp.utils"

local kitty_launch = "/Users/nom/.zsh/bin/kitty-launch"

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

local function reveal_in_yazi()
    local abs, err = absolute_media_path()
    if not abs then
        mp.osd_message(err, 2)
        return
    end

    local info = utils.file_info(abs)
    if not info then
        mp.osd_message("File no longer exists:\n" .. abs, 3)
        mp.msg.warn("Reveal in yazi failed, file missing: " .. abs)
        return
    end

    local directory = utils.split_path(abs)
    local result = utils.subprocess_detached({
        args = {
            kitty_launch,
            "--window",
            "--cwd", directory,
            "--title", " yazi ",
            "--",
            "yazi", abs,
        },
    })

    if result == false then
        mp.osd_message("Reveal in yazi failed", 2)
        mp.msg.error("Failed to launch yazi for: " .. abs)
        return
    end

    mp.osd_message("Revealed in yazi", 1.5)
end

mp.add_key_binding(nil, "reveal_in_yazi", reveal_in_yazi)
