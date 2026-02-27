local mp = require("mp")

local enabled = true                 -- default ON (change if you want)
local ROTATE_LABEL = "auto_rotate"
local OSD_SECS = 1.2

local last_osd = ""
local applied_for_path = nil
local last_decision = nil            -- {need=true/false, spec="..."}

local prev_hwdec = nil
local forced_hwdec = false

local function osd(msg)
  if msg == last_osd then return end
  last_osd = msg
  mp.osd_message(msg, OSD_SECS)
end

local function vf_list()
  local v = mp.get_property_native("vf")
  return (type(v) == "table") and v or {}
end

local function vf_has_label(label)
  for _, f in ipairs(vf_list()) do
    if type(f) == "table" and f.label == label then return true end
  end
  return false
end

local function vf_remove_label(label)
  if not vf_has_label(label) then return false end
  pcall(mp.commandv, "vf", "remove", "@" .. label)
  return true
end

local function vf_set_label(label, spec)
  -- replace under one label (no growth)
  vf_remove_label(label)
  if spec and spec ~= "" then
    pcall(mp.commandv, "vf", "add", "@" .. label .. ":" .. spec)
  end
end

local function hwdec_current()
  return mp.get_property("hwdec-current") or ""
end

local function ensure_hwdec_copy()
  local cur = hwdec_current()
  if cur == "videotoolbox" then
    if not forced_hwdec then
      prev_hwdec = mp.get_property("hwdec")
      forced_hwdec = true
    end
    mp.set_property("hwdec", "videotoolbox-copy")
  end
end

local function restore_hwdec()
  if forced_hwdec then
    mp.set_property("hwdec", prev_hwdec or "auto")
    forced_hwdec = false
    prev_hwdec = nil
  end
end

-- Decide once using *source* params (not dwidth/dheight which change after filters).
-- Also respect container rotation metadata (rotate tag).
local function compute_decision()
  local vp = mp.get_property_native("video-params") -- can be nil briefly
  if type(vp) ~= "table" then return nil end

  local w = tonumber(vp.w) or 0
  local h = tonumber(vp.h) or 0
  if w <= 0 or h <= 0 then return nil end

  -- Some files have rotate metadata; mpv may expose it as vp.rotate (degrees).
  local r = tonumber(vp.rotate) or 0
  r = ((r % 360) + 360) % 360

  -- Effective source display orientation before our filter:
  -- if metadata says 90/270, swap.
  local eff_w, eff_h = w, h
  if r == 90 or r == 270 then
    eff_w, eff_h = h, w
  end

  local is_portrait = eff_h > eff_w

  if not is_portrait then
    return { need = false, spec = "" , note = "landscape" }
  end

  -- We want it horizontal. Rotate 90° clockwise.
  -- transpose=clock is stable.
  return { need = true, spec = "transpose=clock", note = "portrait→landscape" }
end

local function apply_once(reason)
  if not enabled then
    restore_hwdec()
    vf_remove_label(ROTATE_LABEL)
    last_decision = nil
    applied_for_path = nil
    osd("Auto-rotate: OFF" .. (reason and (" • " .. reason) or ""))
    return
  end

  local path = mp.get_property("path") or ""
  if path ~= "" and applied_for_path == path then
    -- already applied for this file
    return
  end

  local d = compute_decision()
  if not d then return end

  -- mark as applied for this file once we have stable params
  applied_for_path = path
  last_decision = d

  if d.need then
    ensure_hwdec_copy()
    vf_set_label(ROTATE_LABEL, d.spec)
    osd("Auto-rotate: ON • " .. d.note .. (reason and (" • " .. reason) or ""))
  else
    restore_hwdec()
    vf_remove_label(ROTATE_LABEL)
    osd("Auto-rotate: ON • " .. d.note .. (reason and (" • " .. reason) or ""))
  end
end

local function on_file_loaded()
  last_osd = ""
  applied_for_path = nil
  last_decision = nil
  apply_once("file-loaded")
end

-- video-params often becomes valid slightly after file-loaded; react once when it appears
mp.observe_property("video-params", "native", function()
  apply_once(nil)
end)

mp.register_event("file-loaded", on_file_loaded)

mp.add_key_binding("a", "toggle_auto_rotate", function()
  enabled = not enabled
  last_osd = ""
  applied_for_path = nil -- allow re-apply immediately
  apply_once("toggled")
end)

-- If mpv disables our filter, stop and tell you (don’t retry-loop).
mp.register_event("log-message", function(e)
  if not enabled then return end
  if not e or not e.text then return end
  if e.text:find("Disabling filter auto_rotate because it has failed", 1, true) then
    vf_remove_label(ROTATE_LABEL)
    restore_hwdec()
    applied_for_path = nil
    last_osd = ""
    osd("Auto-rotate: FAILED • filter removed")
  end
end)
