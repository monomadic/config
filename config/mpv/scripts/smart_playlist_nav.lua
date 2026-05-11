math.randomseed(os.time())

local mp = require("mp")

local EDGE_FRACTION = 0.10
local SEEK_FRACTION = 0.10

local function get_number(name, fallback)
    local value = mp.get_property_number(name)
    if value == nil then
        return fallback
    end
    return value
end

local function get_playlist_info()
    local pos = get_number("playlist-pos", 0)
    local count = get_number("playlist-count", 1)
    return pos, count
end

local function has_previous_entry()
    local pos = get_number("playlist-pos", 0)
    return pos > 0
end

local function has_next_entry()
    local pos, count = get_playlist_info()
    return pos < (count - 1)
end

local function play_previous_entry()
    if has_previous_entry() then
        mp.commandv("playlist-prev", "force")
    else
        mp.commandv("seek", "0", "absolute")
    end
end

local function play_next_entry()
    if has_next_entry() then
        mp.commandv("playlist-next", "force")
    end
end

local function get_chapter_count()
    local chapters = mp.get_property_native("chapter-list")
    if type(chapters) ~= "table" then
        return 0
    end
    return #chapters
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
    if count <= 1 then
        return
    end

    local target = pos
    while target == pos do
        target = math.random(0, count - 1)
    end

    mp.commandv("playlist-play-index", tostring(target))
end

local function random_chapter_or_seek()
    local chapter_count = get_chapter_count()
    if chapter_count > 0 then
        if chapter_count == 1 then
            return
        end

        local current = get_number("chapter", 0)
        local target = current
        while target == current do
            target = math.random(0, chapter_count - 1)
        end

        mp.commandv("set", "chapter", tostring(target))
        return
    end

    local duration = get_number("duration", nil)
    if duration == nil or duration <= 0 then
        return
    end

    mp.commandv("seek", tostring(math.random() * duration), "absolute")
end

mp.add_forced_key_binding("[", "smart-back", smart_back)
mp.add_forced_key_binding("]", "smart-forward", smart_forward)
mp.add_forced_key_binding("{", "playlist-prev-track", play_previous_entry)
mp.add_forced_key_binding("}", "playlist-next-track", play_next_entry)
mp.add_forced_key_binding("\\", "smart-random-position", random_chapter_or_seek)
mp.add_forced_key_binding("|", "playlist-random-track", random_playlist_entry)
