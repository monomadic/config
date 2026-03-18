local mp = require("mp")

local enabled = false                      -- default OFF
local LABEL = "auto_landscape"             -- vf label (no @)
local OSD_SECS = 1.2

local last_osd = ""
local applied_for_path = nil
local pending_apply = false

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
  vf_remove_label(label)
  if spec and spec ~= "" then
    pcall(mp.commandv, "vf", "add", "@" .. label .. ":" .. spec)
  end
end

local function hwdec_current()
  return mp.get_property("hwdec-current") or ""
end

local function ensure_hwdec_copy()
  -- Force a SW-readable decode path before touching filters.
  -- Using "videotoolbox-copy" avoids hw surfaces in the filter graph.
  if not forced_hwdec then
    prev_hwdec = mp.get_property("hwdec")
    forced_hwdec = true
  end
  mp.set_property("hwdec", "videotoolbox-copy")
end

local function restore_hwdec()
  if forced_hwdec then
    mp.set_property("hwdec", prev_hwdec or "auto")
    forced_hwdec = false
    prev_hwdec = nil
  end
end

-- Decide based on *source* params (not dwidth/dheight).
local function compute_need_rotate()
  local vp = mp.get_property_native("video-params")
  if type(vp) ~= "table" then return nil end

  local w = tonumber(vp.w) or 0
  local h = tonumber(vp.h) or 0
  if w <= 0 or h <= 0 then return nil end

  local r = tonumber(vp.rotate) or 0
  r = ((r % 360) + 360) % 360

  local eff_w, eff_h = w, h
  if r == 90 or r == 270 then
    eff_w, eff_h = h, w
  end

  return (eff_h > eff_w)
end

local function apply_for_current_file(reason)
  if not enabled then
    pending_apply = false
    restore_hwdec()
    vf_remove_label(LABEL)
    applied_for_path = nil

    -- local v = mp.get_property_number("osd-level", OSD_FULL)
    -- if v and v > 0 then
    --   osd("Auto-landscape: OFF" .. (reason and (" • " .. reason) or ""))
    -- end
    
    return
  end

  local path = mp.get_property("path") or ""
  if path ~= "" and applied_for_path == path then
    return
  end

  local need = compute_need_rotate()
  if need == nil then return end

  applied_for_path = path

  if not need then
    pending_apply = false
    restore_hwdec()
    vf_remove_label(LABEL)
    osd("Auto-landscape: ON • already landscape" .. (reason and (" • " .. reason) or ""))
    return
  end

  -- Portrait -> rotate once.
  -- Critical: set hwdec first, then apply vf on next tick so mpv reconfigures.
  if not pending_apply then
    pending_apply = true
    ensure_hwdec_copy()
    mp.add_timeout(0, function()
      if not enabled then return end
      vf_set_label(LABEL, "transpose=clock")
      pending_apply = false
      osd("Auto-landscape: ON • portrait→landscape" .. (reason and (" • " .. reason) or ""))
    end)
  end
end

local function broadcast_state()
  mp.commandv("script-message", "auto_landscape_broadcast", enabled and "yes" or "no")
end

mp.register_script_message("auto-rotate-query", function()
  broadcast_state()
end)

mp.register_script_message("toggle", function()
  enabled = not enabled
  broadcast_state()
  last_osd = ""
  applied_for_path = nil
  pending_apply = false
  apply_for_current_file("toggled")
end)

mp.register_event("file-loaded", function()
  broadcast_state()
  last_osd = ""
  applied_for_path = nil
  pending_apply = false
  apply_for_current_file("file-loaded")
end)

-- video-params often becomes available after file-loaded; react once.
mp.observe_property("video-params", "native", function()
  apply_for_current_file(nil)
end)

mp.add_key_binding(nil, "toggle_force_landscape", function()
  enabled = not enabled
  broadcast_state()
  last_osd = ""
  applied_for_path = nil
  pending_apply = false
  apply_for_current_file("toggled")
end)

-- If mpv disables our filter, stop and report (don’t loop / re-add spam).
mp.register_event("log-message", function(e)
  if not enabled then return end
  if not e or not e.text then return end
  if e.text:find("Disabling filter " .. LABEL .. " because it has failed", 1, true) then
    vf_remove_label(LABEL)
    restore_hwdec()
    pending_apply = false
    applied_for_path = nil
    last_osd = ""
    osd("Auto-landscape: FAILED • filter removed")
  end
end)
