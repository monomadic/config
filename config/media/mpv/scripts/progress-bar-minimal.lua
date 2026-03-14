local mp = require "mp"
local assdraw = require "mp.assdraw"

local ov = mp.create_osd_overlay("ass-events")
ov.z = 100            -- behind keybar

local visible = true

local function draw_bar()
  local osd_level = mp.get_property_number("osd-level", 1)
  if not visible or osd_level == 0 then
    ov:remove()
    return
  end

  local pos = mp.get_property_number("percent-pos")
  if not pos then return end

  local dim = mp.get_property_native("osd-dimensions")
  if not dim or not dim.w or not dim.h then return end
  local w, h = dim.w, dim.h

  local bar_h = 5
  local ass = assdraw.ass_new()

  ass:new_event()
  ass:append("{\\bord0\\shad0\\1c&H000000&}")
  ass:pos(0, h - bar_h)
  ass:draw_start(); ass:rect_cw(0, 0, w, bar_h); ass:draw_stop()

  ass:new_event()
  ass:append("{\\bord0\\shad0\\1c&H444444&}")
  ass:pos(0, h - bar_h)
  ass:draw_start(); ass:rect_cw(0, 0, w * pos / 100, bar_h); ass:draw_stop()

  ov.data  = ass.text
  ov.res_x = w
  ov.res_y = h
  ov:update()
end

mp.observe_property("percent-pos", "number", draw_bar)
mp.observe_property("osd-dimensions", "native", draw_bar)
mp.observe_property("osd-level", "number", draw_bar)

mp.add_key_binding("Ctrl+p", "toggle-progress", function()
  visible = not visible
  draw_bar()
end)
