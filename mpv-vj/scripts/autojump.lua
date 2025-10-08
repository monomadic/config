-- Auto-jump script for mpv
-- Place in ~/.config/mpv/scripts/autojump.lua

local options = {
    interval = 5,  -- Jump interval in seconds
    jump_size = 10, -- Size of jump in seconds
}

require 'mp.options'
read_options(options, "autojump")

local timer = nil
local enabled = false

function jump_forward()
    local current_pos = mp.get_property_number("time-pos", 0)
    local duration = mp.get_property_number("duration", 0)
    
    if duration > 0 then
        local new_pos = current_pos + options.jump_size
        
        if new_pos >= duration then
            -- Jump to next file in playlist
            mp.command("playlist-next")
        else
            -- Jump forward in current file
            mp.set_property_number("time-pos", new_pos)
        end
    end
end

function start_timer()
    if timer then
        timer:kill()
    end
    
    timer = mp.add_periodic_timer(options.interval, jump_forward)
    enabled = true
    mp.osd_message("Auto-jump enabled", 1)
end

function stop_timer()
    if timer then
        timer:kill()
        timer = nil
    end
    enabled = false
    mp.osd_message("Auto-jump disabled", 1)
end

function toggle_timer()
    if enabled then
        stop_timer()
    else
        start_timer()
    end
end

-- Auto-start on file load
mp.register_event("file-loaded", function()
    start_timer()
end)

-- Key bindings
mp.add_key_binding("ctrl+j", "toggle-autojump", toggle_timer)
mp.add_key_binding("ctrl+[", "decrease-interval", function()
    options.interval = math.max(1, options.interval - 1)
    if timer then start_timer() end
    mp.osd_message("Interval: " .. options.interval .. "s", 1)
end)
mp.add_key_binding("ctrl+]", "increase-interval", function()
    options.interval = options.interval + 1
    if timer then start_timer() end
    mp.osd_message("Interval: " .. options.interval .. "s", 1)
end)
