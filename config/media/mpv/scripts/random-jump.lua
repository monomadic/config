local mp   = require "mp"
local math = require "math"

-- better-ish seed (avoids same-seed on fast reloads)
math.randomseed((os.time() + math.floor((mp.get_time() or 0) * 1000)) % 2147483647)

local pending_jump = false
local jump_delay = 15 -- seconds

-- Autojump mode toggle (playlist jump)
local active = false
local timer = nil

-- Autojump *within* file toggle
local file_jump_active = false
local file_jump_timer = nil

-- share state with keybar (or anything else)
local function send_state()
  mp.commandv(
    "script-message",
    "keybar-randjump-state",
    active and "1" or "0",
    file_jump_active and "1" or "0",
    tostring(jump_delay)
  )
end

mp.register_script_message("randjump-query", function()
  send_state()
end)

-- Random seek within file excluding first/last 10%
local function random_seek_within_file()
  local duration = mp.get_property_number("duration", 0)
  if not duration or duration <= 0 then
    mp.msg.warn("Invalid duration; cannot seek.")
    return
  end

  local min_time = duration * 0.10
  local max_time = duration * 0.90
  local rnd_time = min_time + math.random() * (max_time - min_time)
  mp.msg.info(string.format("Random seek within file to %.2f seconds", rnd_time))
  mp.commandv("seek", rnd_time, "absolute")
end

-- Jump to a random playlist file (and then random-seek on file-loaded)
function random_playlist_jump()
  local pl_count = mp.get_property_number("playlist-count", 0)
  if pl_count < 1 then
    mp.msg.warn("No playlist items available!")
    return
  end

  local cur = mp.get_property_number("playlist-pos", 0)
  local rnd_index = math.random(0, pl_count - 1)
  mp.msg.info(string.format("Jumping to playlist index %d", rnd_index))

  mp.set_property("pause", "yes")

  if rnd_index == cur then
    -- IMPORTANT: file-loaded won't fire; avoid getting stuck paused/pending
    random_seek_within_file()
    mp.set_property("pause", "no")
    pending_jump = false
    return
  end

  mp.set_property_number("playlist-pos", rnd_index)
  pending_jump = true
end

-- After playlist jump: seek to random position then unpause
mp.register_event("file-loaded", function()
  if not pending_jump then return end
  pending_jump = false

  -- reuse the same "avoid first/last 10%" behavior
  random_seek_within_file()

  mp.set_property("pause", "no")
end)

-- Autojump toggle (playlist)
local function toggle_auto_jump()
  active = not active
  if active then
    timer = mp.add_periodic_timer(jump_delay, random_playlist_jump)
    mp.osd_message("Autojump: ON (delay: " .. jump_delay .. "s)")
  else
    if timer then timer:kill(); timer = nil end
    mp.osd_message("Autojump: OFF")
  end
  send_state()
end

-- Set jump delay (shared by both timers)
local function set_jump_delay(seconds)
  jump_delay = seconds
  mp.osd_message("Jump delay set to: " .. seconds .. "s")

  if timer then timer:kill(); timer = mp.add_periodic_timer(jump_delay, random_playlist_jump) end
  if file_jump_timer then file_jump_timer:kill(); file_jump_timer = mp.add_periodic_timer(jump_delay, random_seek_within_file) end

  send_state()
end

-- Percentage-based delay (based on current file duration)
local function set_jump_delay_percentage(percent)
  local duration = mp.get_property_number("duration", 0)
  if duration and duration > 0 then
    set_jump_delay(duration * percent)
  else
    mp.osd_message("Cannot set delay: no valid duration")
  end
end

-- Always jump to last 10%
local function jump_to_last_10_percent()
  local duration = mp.get_property_number("duration", 0)
  if not duration or duration <= 0 then
    mp.msg.warn("Invalid duration.")
    return
  end
  local pos = duration * 0.90
  mp.msg.info(string.format("Jumping to last 10%% at %.2f seconds", pos))
  mp.commandv("seek", pos, "absolute")
end

-- Autojump within file toggle
local function toggle_auto_seek_within_file()
  file_jump_active = not file_jump_active
  if file_jump_active then
    file_jump_timer = mp.add_periodic_timer(jump_delay, random_seek_within_file)
    mp.osd_message("File AutoSeek: ON (delay: " .. jump_delay .. "s)")
  else
    if file_jump_timer then file_jump_timer:kill(); file_jump_timer = nil end
    mp.osd_message("File AutoSeek: OFF")
  end
  send_state()
end

-- Key bindings
mp.add_key_binding("ENTER", "random_playlist_jump", random_playlist_jump)
mp.add_key_binding("1", "toggle_auto_jump", toggle_auto_jump)
mp.add_key_binding("2", "toggle_auto_seek_within_file", toggle_auto_seek_within_file)
mp.add_key_binding("j", "random_seek_within_file", random_seek_within_file)

-- CMD+[1–9] (META+digit)
mp.add_key_binding("meta+1", "set_delay_1", function() set_jump_delay(1) end)
mp.add_key_binding("meta+2", "set_delay_3", function() set_jump_delay(3) end)
mp.add_key_binding("meta+3", "set_delay_5", function() set_jump_delay(5) end)
mp.add_key_binding("meta+4", "set_delay_10", function() set_jump_delay(10) end)
mp.add_key_binding("meta+5", "set_delay_20", function() set_jump_delay(20) end)
mp.add_key_binding("meta+6", "set_delay_5pct", function() set_jump_delay_percentage(0.05) end)
mp.add_key_binding("meta+7", "set_delay_10pct", function() set_jump_delay_percentage(0.10) end)
mp.add_key_binding("meta+8", "set_delay_15pct", function() set_jump_delay_percentage(0.15) end)
mp.add_key_binding("meta+9", "jump_last_10pct", jump_to_last_10_percent)

-- push initial state once mpv is up
mp.add_timeout(0, send_state)
