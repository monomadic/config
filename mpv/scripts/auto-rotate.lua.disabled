-- auto-rotate.lua
-- Auto-rotate portrait videos 90° clockwise, with toggle support

local mp = require 'mp'
local utils = require 'mp.utils'

local state_file = mp.command_native({'expand-path', '~~/script-opts/auto-rotate-state.json'})
local enabled = false

local function load_state()
  local f = io.open(state_file, 'r')
  if not f then return end
  local s = f:read('*a'); f:close()
  if not s or s == '' then return end
  local t = utils.parse_json(s)
  if t and t.enabled ~= nil then
    enabled = not not t.enabled
  end
end

local function save_state()
  local s = utils.format_json({ enabled = enabled })
  local f = io.open(state_file, 'w')
  if not f then return end
  f:write(s); f:close()
end

local function auto_rotate()
  -- Get raw video size
  local w = mp.get_property_number("video-params/w")
  local h = mp.get_property_number("video-params/h")
  if not w or not h then return end

  -- Rotation coming from container/codec metadata
  local meta_rotate = mp.get_property_number("video-params/rotate") or 0

  if enabled then
    -- Only mess with it if there's no metadata rotation already
    if h > w and meta_rotate == 0 then
      -- Portrait: rotate 90° clockwise
      mp.set_property_number("video-rotate", 90)
      mp.osd_message("Auto-rotated 90° (portrait)")
    else
      -- Force back to normal for non-portrait or already-rotated stuff
      mp.set_property_number("video-rotate", 0)
    end
  else
    -- When disabled, don't auto-rotate, just reset to 0
    mp.set_property_number("video-rotate", 0)
  end
end

load_state()
mp.register_event("file-loaded", auto_rotate)
mp.register_event('shutdown', save_state)

-- Toggle handler
mp.register_script_message('toggle', function()
  enabled = not enabled
  auto_rotate()
  save_state()
  mp.osd_message(("Auto-rotate: %s"):format(enabled and "ON" or "OFF"))
end)
