local mp = require("mp")
local math = require("math")

local EDGE_FRACTION = 0.10
local SEEK_FRACTION = 0.10
local RANDOM_SEEK_MARGIN = 0.10
local DEFAULT_JUMP_DELAY = 15

math.randomseed((os.time() + math.floor((mp.get_time() or 0) * 1000)) % 2147483647)

local pending_playlist_seek = false
local jump_delay = DEFAULT_JUMP_DELAY
local auto_jump_active = false
local auto_jump_timer = nil
local auto_seek_active = false
local auto_seek_timer = nil

local function get_number(name, fallback)
    local value = mp.get_property_number(name)
    if value == nil then
        return fallback
    end
    return value
end

local function get_playlist_info()
    return get_number("playlist-pos", 0), get_number("playlist-count", 1)
end

local function get_chapter_count()
    local chapters = mp.get_property_native("chapter-list")
    if type(chapters) == "table" then
        return #chapters
    end

    return get_number("chapter-list/count", 0)
end

local function random_index(count, current, exclude_current)
    if count <= 0 or (exclude_current and count <= 1) then
        return nil
    end

    local target = current
    while exclude_current and target == current do
        target = math.random(0, count - 1)
    end

    if not exclude_current then
        target = math.random(0, count - 1)
    end

    return target
end

local function random_time(duration, margin)
    if duration == nil or duration <= 0 then
        return nil
    end

    margin = margin or 0
    local min_time = duration * margin
    local max_time = duration * (1 - margin)
    if max_time <= min_time then
        return duration / 2
    end

    return min_time + math.random() * (max_time - min_time)
end

local function seek_absolute(seconds)
    if seconds ~= nil then
        mp.commandv("seek", tostring(seconds), "absolute")
    end
end

local function random_seek(margin)
    local duration = get_number("duration", nil)
    local target = random_time(duration, margin)
    if target == nil then
        mp.msg.warn("Invalid duration; cannot seek.")
        return false
    end

    seek_absolute(target)
    return true
end

local function send_random_jump_state()
    mp.commandv(
        "script-message",
        "keybar-randjump-state",
        auto_jump_active and "1" or "0",
        auto_seek_active and "1" or "0",
        tostring(jump_delay)
    )
end

local function play_previous_entry()
    local pos = get_number("playlist-pos", 0)
    if pos > 0 then
        mp.commandv("playlist-prev", "force")
    else
        mp.commandv("seek", "0", "absolute")
    end
end

local function play_next_entry()
    local pos, count = get_playlist_info()
    if pos < count - 1 then
        mp.commandv("playlist-next", "force")
    end
end

local function get_fraction_position()
    local percent = get_number("percent-pos", nil)
    if percent ~= nil then
        return percent / 100
    end

    local time_pos = get_number("time-pos", nil)
    local duration = get_number("duration", nil)
    if time_pos == nil or duration == nil or duration <= 0 then
        return nil
    end

    return time_pos / duration
end

local function smart_back()
    if get_chapter_count() > 0 then
        mp.commandv("add", "chapter", "-1")
        return
    end

    local fraction = get_fraction_position()
    if fraction ~= nil and fraction <= EDGE_FRACTION then
        play_previous_entry()
        return
    end

    mp.commandv("seek", string.format("-%d", SEEK_FRACTION * 100), "relative-percent")
end

local function smart_forward()
    if get_chapter_count() > 0 then
        mp.commandv("add", "chapter", "1")
        return
    end

    local fraction = get_fraction_position()
    if fraction ~= nil and fraction >= (1 - EDGE_FRACTION) then
        play_next_entry()
        return
    end

    mp.commandv("seek", string.format("%d", SEEK_FRACTION * 100), "relative-percent")
end

local function random_playlist_entry()
    local pos, count = get_playlist_info()
    local target = random_index(count, pos, true)
    if target ~= nil then
        mp.commandv("playlist-play-index", tostring(target))
    end
end

local function random_chapter()
    local count = get_chapter_count()
    local current = get_number("chapter", 0)
    local target = random_index(count, current, true)
    if target ~= nil then
        mp.commandv("set", "chapter", tostring(target))
        return true
    end

    return false
end

local function random_chapter_or_seek()
    if random_chapter() then
        return
    end

    random_seek(0)
end

local function random_seek_within_file()
    random_seek(RANDOM_SEEK_MARGIN)
end

local function random_playlist_jump()
    local pos, count = get_playlist_info()
    if count < 1 then
        mp.msg.warn("No playlist items available.")
        return
    end

    local target = random_index(count, pos, false)
    if target == nil then
        return
    end

    mp.set_property_bool("pause", true)

    if target == pos then
        random_seek_within_file()
        pending_playlist_seek = false
        mp.set_property_bool("pause", false)
        return
    end

    pending_playlist_seek = true
    mp.commandv("playlist-play-index", tostring(target))
end

local function set_jump_delay(seconds)
    jump_delay = seconds
    mp.osd_message("Jump delay set to: " .. seconds .. "s")

    if auto_jump_timer then
        auto_jump_timer:kill()
        auto_jump_timer = mp.add_periodic_timer(jump_delay, random_playlist_jump)
    end

    if auto_seek_timer then
        auto_seek_timer:kill()
        auto_seek_timer = mp.add_periodic_timer(jump_delay, random_seek_within_file)
    end

    send_random_jump_state()
end

local function set_jump_delay_percentage(percent)
    local duration = get_number("duration", nil)
    if duration == nil or duration <= 0 then
        mp.osd_message("Cannot set delay: no valid duration")
        return
    end

    set_jump_delay(duration * percent)
end

local function jump_to_last_10_percent()
    local duration = get_number("duration", nil)
    if duration == nil or duration <= 0 then
        mp.msg.warn("Invalid duration.")
        return
    end

    seek_absolute(duration * 0.90)
end

local function toggle_auto_jump()
    auto_jump_active = not auto_jump_active
    if auto_jump_active then
        auto_jump_timer = mp.add_periodic_timer(jump_delay, random_playlist_jump)
        mp.osd_message("Autojump: ON (delay: " .. jump_delay .. "s)")
    else
        if auto_jump_timer then
            auto_jump_timer:kill()
            auto_jump_timer = nil
        end
        mp.osd_message("Autojump: OFF")
    end

    send_random_jump_state()
end

local function toggle_auto_seek_within_file()
    auto_seek_active = not auto_seek_active
    if auto_seek_active then
        auto_seek_timer = mp.add_periodic_timer(jump_delay, random_seek_within_file)
        mp.osd_message("File AutoSeek: ON (delay: " .. jump_delay .. "s)")
    else
        if auto_seek_timer then
            auto_seek_timer:kill()
            auto_seek_timer = nil
        end
        mp.osd_message("File AutoSeek: OFF")
    end

    send_random_jump_state()
end

mp.register_event("file-loaded", function()
    if not pending_playlist_seek then
        return
    end

    pending_playlist_seek = false
    random_seek_within_file()
    mp.set_property_bool("pause", false)
end)

mp.register_script_message("play-random", random_playlist_entry)
mp.register_script_message("play-random-chapter", random_chapter)
mp.register_script_message("randjump-query", send_random_jump_state)

mp.add_forced_key_binding("[", "smart-back", smart_back)
mp.add_forced_key_binding("]", "smart-forward", smart_forward)
mp.add_forced_key_binding("{", "playlist-prev-track", play_previous_entry)
mp.add_forced_key_binding("}", "playlist-next-track", play_next_entry)
mp.add_forced_key_binding("\\", "smart-random-position", random_chapter_or_seek)
mp.add_forced_key_binding("|", "playlist-random-track", random_playlist_entry)

mp.add_key_binding(nil, "random_playlist_jump", random_playlist_jump)
mp.add_key_binding(nil, "toggle_auto_jump", toggle_auto_jump)
mp.add_key_binding(nil, "toggle_auto_seek_within_file", toggle_auto_seek_within_file)
mp.add_key_binding(nil, "random_seek_within_file", random_seek_within_file)
mp.add_key_binding(nil, "set_delay_1", function() set_jump_delay(1) end)
mp.add_key_binding(nil, "set_delay_3", function() set_jump_delay(3) end)
mp.add_key_binding(nil, "set_delay_5", function() set_jump_delay(5) end)
mp.add_key_binding(nil, "set_delay_10", function() set_jump_delay(10) end)
mp.add_key_binding(nil, "set_delay_20", function() set_jump_delay(20) end)
mp.add_key_binding(nil, "set_delay_5pct", function() set_jump_delay_percentage(0.05) end)
mp.add_key_binding(nil, "set_delay_10pct", function() set_jump_delay_percentage(0.10) end)
mp.add_key_binding(nil, "set_delay_15pct", function() set_jump_delay_percentage(0.15) end)
mp.add_key_binding(nil, "jump_last_10pct", jump_to_last_10_percent)

mp.add_timeout(0, send_random_jump_state)
