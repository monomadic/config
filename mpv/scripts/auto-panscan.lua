-- panscan-persist.lua
-- Persist panscan state (enabled + level) across files and restarts.

local mp    = require 'mp'
local utils = require 'mp.utils'

local state_file = mp.command_native({'expand-path', '~~/script-opts/panscan-state.json'})
local enabled, level = false, 1.0

local function clamp(x, lo, hi)
  if x < lo then return lo end
  if x > hi then return hi end
  return x
end

local function load_state()
  local f = io.open(state_file, 'r')
  if not f then return end
  local s = f:read('*a'); f:close()
  if not s or s == '' then return end
  local t = utils.parse_json(s)
  if t then
    if t.enabled ~= nil then enabled = not not t.enabled end
    if t.level  ~= nil then level   = clamp(tonumber(t.level) or level, 0.0, 1.0) end
  end
end

local function save_state()
  local s = utils.format_json({ enabled = enabled, level = level })
  local f = io.open(state_file, 'w')
  if not f then return end
  f:write(s); f:close()
end

local function apply()
  mp.set_property_number('panscan', enabled and level or 0)
end

load_state()
mp.register_event('file-loaded', apply)
mp.register_event('shutdown', save_state)

mp.add_key_binding(nil, 'toggle', function()
  enabled = not enabled
  apply(); save_state()
  mp.osd_message(("panscan: %s"):format(enabled and ("ON ("..level..")") or "OFF"))
end)

mp.add_key_binding(nil, 'inc', function()
  level = clamp(level + 0.1, 0.0, 1.0)
  if enabled then apply() end
  save_state()
  mp.osd_message(("panscan=%.1f"):format(level))
end)

mp.add_key_binding(nil, 'dec', function()
  level = clamp(level - 0.1, 0.0, 1.0)
  if enabled then apply() end
  save_state()
  mp.osd_message(("panscan=%.1f"):format(level))
end)
