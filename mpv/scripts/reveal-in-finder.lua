local utils = require "mp.utils"

local function reveal_in_finder()
    local media_path = mp.get_property("path")

    if not media_path or media_path == "" then
        mp.osd_message("No media loaded", 2)
        return
    end

    -- Only allow real files
    local proto = mp.get_property("protocol") or ""
    if proto ~= "" and proto ~= "file" then
        mp.osd_message("Not a local file (" .. proto .. ")", 2)
        return
    end

    -- Normalize to absolute path
    local abs = utils.join_path(
        mp.get_property("working-directory") or "",
        media_path
    )

    local info = utils.file_info(abs)
    if not info then
        mp.osd_message("File no longer exists:\n" .. abs, 3)
        mp.msg.warn("Reveal failed, file missing: " .. abs)
        return
    end

    local result = utils.subprocess({
        args = { "/usr/bin/open", "-R", abs },
        cancellable = false,
    })

    if result.status ~= 0 then
        mp.osd_message("Reveal failed", 2)
        mp.msg.error("open -R failed: " .. (result.error or "unknown"))
        return
    end

    mp.osd_message("Revealed in Finder", 1.5)
end

mp.add_key_binding("Ctrl+f", "reveal_in_finder", reveal_in_finder)
