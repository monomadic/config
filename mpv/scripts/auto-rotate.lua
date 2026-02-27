local mp = require("mp")

local enabled = false
local ROTATE_LABEL = "@auto_rotate"

local OSD_SECS = 1.2
local last_osd = ""
local last_need = nil

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
  pcall(mp.commandv, "vf", "remove", label)
  return true
end

-- Rotate that works with videotoolbox: download hw frames first, pick a sane sw format.
-- nv12 is widely supported and matches many hwdec outputs.
local function vf_add_rotate_90(label)
  vf_remove_label(label)
  -- chain is under one label; mpv will treat the whole thing as one filter entry
  -- "hwdownload,format=nv12,rotate=90"
  pcall(mp.commandv, "vf", "add", label .. ":hwdownload,format=nv12,rotate=90")
end

local function get_dims()
  local dw = mp.get_property_number("dwidth", 0)
  local dh = mp.get_property_number("dheight", 0)
  if dw <= 0 or dh <= 0 then return nil, nil end
  return dw, dh
end

local function hwdec_active()
  -- mpv reports "current" as actually used ("videotoolbox" etc), or "no"
  local cur = mp.get_property("hwdec-current") or ""
  return cur ~= "" and cur ~= "no"
end

local function apply_rotate_if_needed(reason)
  if not enabled then
    local removed = vf_remove_label(ROTATE_LABEL)
    if removed then osd("Auto-rotate: OFF" .. (reason and (" • " .. reason) or "")) end
    last_need = nil
    return
  end

  local dw, dh = get_dims()
  if not dw or not dh then return end

  local need = (dh > dw)
  if last_need == need and reason == nil then
    -- avoid churning on dimension observers once stable
    return
  end
  last_need = need

  if not need then
    local removed = vf_remove_label(ROTATE_LABEL)
    if removed then
      osd("Auto-rotate: ON • landscape • no rotate" .. (reason and (" • " .. reason) or ""))
    end
    return
  end

  -- portrait: ensure rotate filter exists (rebuild to recover from “failed” state)
  vf_add_rotate_90(ROTATE_LABEL)

  -- If it failed previously, mpv may disable it again; give a hint about hwdec.
  local hw = hwdec_active()
  osd("Auto-rotate: ON • portrait • rotate=90" .. (hw and " • hwdownload" or "") .. (reason and (" • " .. reason) or ""))
end

local function on_file_loaded()
  last_osd = ""
  last_need = nil
  apply_rotate_if_needed("file-loaded")
end

local function toggle_auto_rotate()
  enabled = not enabled
  last_osd = ""
  if enabled then
    osd("Auto-rotate: ON")
    apply_rotate_if_needed("toggled")
  else
    apply_rotate_if_needed("toggled")
  end
end

-- Optional: if rotate fails even with hwdownload, fall back by disabling hwdec for this file.
-- This reacts to mpv’s log message “Disabling filter ... because it has failed.”
-- Requires msg-level=... and the message to pass through. Safe no-op otherwise.
mp.register_event("log-message", function(e)
  if not enabled then return end
  if not e or e.level == nil or e.text == nil then return end
  if e.text:find("Disabling filter auto_rotate because it has failed", 1, true) then
    -- aggressive fallback: turn off hwdec and retry once
    mp.set_property("hwdec", "no")
    last_osd = ""
    osd("Auto-rotate: rotate filter failed • hwdec=no • retry")
    -- re-add filter after changing hwdec
    vf_add_rotate_90(ROTATE_LABEL)
  end
end)

mp.register_event("file-loaded", on_file_loaded)
mp.register_event("video-reconfig", function() apply_rotate_if_needed("reconfig") end)
mp.observe_property("dwidth", "number", function() apply_rotate_if_needed() end)
mp.observe_property("dheight", "number", function() apply_rotate_if_needed() end)

mp.add_key_binding("a", "toggle", toggle_auto_rotate)
