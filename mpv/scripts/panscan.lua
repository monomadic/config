-- Reapply panscan on each file; toggle + step bindings via script-binding.
local mp = require 'mp'
local enabled = true
local level = 1.0

local function apply()
  mp.set_property_number("panscan", enabled and level or 0)
end

mp.register_event("file-loaded", apply)

mp.add_key_binding(nil, "toggle", function()
  enabled = not enabled
  apply()
  mp.osd_message(("panscan: %s"):format(enabled and ("ON ("..level..")") or "OFF"))
end)

mp.add_key_binding(nil, "inc", function()
  level = math.min(1.0, level + 0.1)
  if enabled then apply() end
  mp.osd_message(("panscan=%.1f"):format(level))
end)

mp.add_key_binding(nil, "dec", function()
  level = math.max(0.0, level - 0.1)
  if enabled then apply() end
  mp.osd_message(("panscan=%.1f"):format(level))
end)
