--- @since 26.8.15
-- Searchable action menu for the selection (or the hovered file), built the
-- way the `~` help box is: one bordered box holding a `ui.Input`, a divider
-- and the list. Entries and their conditions live in `items.lua`.
--
-- Type to filter; <C-n>/<C-p> (or <Down>/<Up>) move; <Enter> runs; <Esc>
-- closes. Moving needs input-layer bindings in keymap.toml:
--   run = "mgr:plugin action-menu -- down"
-- (routed through mgr because an input only runs `plugin` in normal mode).
-- Outside the menu, those keys fall back to the input's history recall.

local KINDS = {
	video = { "mp4", "m4v", "mov", "mkv", "webm", "avi", "wmv", "flv", "mpg", "mpeg", "ts", "mts", "m2ts", "3gp", "ogv" },
	audio = { "mp3", "m4a", "aac", "flac", "wav", "aif", "aiff", "ogg", "opus", "alac", "wma" },
	image = { "jpg", "jpeg", "png", "gif", "webp", "heic", "heif", "tif", "tiff", "bmp", "avif", "icns" },
}

-- Max box height (width tracks the current column, see M:new).
local HEIGHT = 25
-- Group column on the right of each row, 2 chars wider than before.
local GROUP_W = 18

---------------------------------------------------------------------- matching

local function ext_of(name) return (name:match("%.([^.]+)$") or ""):lower() end

local function in_list(list, v)
	for _, x in ipairs(list) do
		if x == v then
			return true
		end
	end
	return false
end

local function is_kind(t, kind)
	if kind == "dir" then
		return t.is_dir
	elseif kind == "file" then
		return not t.is_dir
	end
	return not t.is_dir and in_list(KINDS[kind] or {}, t.ext)
end

-- Every target must satisfy every condition in `when`.
local function applies(when, targets)
	if not when then
		return true
	end
	if type(when) == "function" then
		return when(targets)
	end
	if when.single and #targets ~= 1 then
		return false
	end
	for _, t in ipairs(targets) do
		if when.kind then
			local kinds = type(when.kind) == "table" and when.kind or { when.kind }
			local ok = false
			for _, k in ipairs(kinds) do
				ok = ok or is_kind(t, k)
			end
			if not ok then
				return false
			end
		end
		if when.ext and (t.is_dir or not in_list(when.ext, t.ext)) then
			return false
		end
	end
	return true
end

-- The command as shown in the bar: repo tools by bare name, so the part that
-- matters isn't pushed off the end by the install path.
local function display(run) return (run:gsub("%$HOME/%.local/bin/", ""):gsub("~/%.local/bin/", "")) end

-- Flatten groups into rows, keeping only entries that apply.
local function collect(groups, targets)
	local rows = {}
	for _, g in ipairs(groups) do
		if applies(g.when, targets) then
			for _, it in ipairs(g) do
				if applies(it.when, targets) then
					rows[#rows + 1] = {
						group = g.group,
						desc = it.desc,
						run = it.run,
						block = it.block or false,
						orphan = it.orphan or false,
						cmd = display(it.run),
						-- The command is searchable too, so a tool can be found by name.
						hay = (g.group .. " " .. it.desc .. " " .. display(it.run)):lower(),
					}
				end
			end
		end
	end
	return rows
end

-- Space-separated terms, each must appear somewhere in "group desc".
local function filter(rows, query)
	local terms = {}
	for w in query:lower():gmatch("%S+") do
		terms[#terms + 1] = w
	end
	local out = {}
	for _, r in ipairs(rows) do
		local keep = true
		for _, w in ipairs(terms) do
			if not r.hay:find(w, 1, true) then
				keep = false
				break
			end
		end
		if keep then
			out[#out + 1] = r
		end
	end
	return out
end

---------------------------------------------------------------------- sync side
-- The menu lives in the sync context: the ui.Input is a userdata that can't
-- cross into the async entry, and its events arrive through ps.sub there.

local function close(self)
	ps.unsub("input")
	if self.children then
		Modal:children_remove(self.children)
	end
	self.children, self.input, self.all, self.rows = nil, nil, nil, nil
	ui.render()
end

local function on_input(self, e)
	if e.type == "type" then
		-- Only a real edit re-filters (and resets the cursor).
		if e.value ~= self.query then
			self.query, self.rows, self.cursor = e.value, filter(self.all, e.value), 0
		end
		ui.render()
	elseif e.type == "submit" then
		local r = self.rows[self.cursor + 1]
		close(self)
		if r then
			ya.emit("shell", { r.run, block = r.block, orphan = r.orphan })
		end
	elseif e.type == "cancel" then
		close(self)
	end
end

local open = ya.sync(function(self, all)
	if self.children then
		return
	end
	self.all, self.rows, self.cursor, self.query = all, all, 0, ""
	self.input = ui.Input { realtime = true }
	ps.sub("input", function(e) on_input(self, e) end)
	self.children = Modal:children_add(self, 10)
	ui.render()
end)

-- Called from a second invocation (`plugin action-menu -- down`), so it only
-- touches shared state. Returns false when no menu is open.
local move = ya.sync(function(self, step)
	if not self.children then
		return false
	end
	if #self.rows > 0 then
		self.cursor = (self.cursor + step) % #self.rows
	end
	ui.render()
	return true
end)

-- Plain data only: Files don't cross the sync boundary.
local get_targets = ya.sync(function()
	local tab, out = cx.active, {}
	for _, f in pairs(tab.selected) do
		out[#out + 1] = { name = f.name, is_dir = f.cha.is_dir }
	end
	local h = tab.current.hovered
	if #out == 0 and h then
		out[1] = { name = h.name, is_dir = h.cha.is_dir }
	end
	return out
end)

---------------------------------------------------------------------- render

local M = {}

-- Anchored under the hovered file's row, in the "current" column, rather
-- than the help box's centred position. Falls back to opening upward when
-- there isn't room below.
--
-- Yazi doesn't hand plugins the current pane's rect, so this rebuilds it the
-- same way `Root`/`Tab` do (yazi-plugin/preset/components/{root,tab}.lua):
-- a 1-row header, a tabs row only when there's more than one tab, the body
-- split horizontally by `rt.mgr.ratio`, and the current column's chunk
-- padded by 1 column on each side.
function M:new(area)
	local tabs_h = #cx.tabs > 1 and 1 or 0
	local body = ui.Rect { x = area.x, y = area.y + 1 + tabs_h, w = area.w, h = area.h - 2 - tabs_h }

	local ratio = rt.mgr.ratio
	local sum = ratio[1] + ratio[2] + ratio[3]
	local chunks = ui.Layout()
		:direction(ui.Layout.HORIZONTAL)
		:constraints {
			ui.Constraint.Ratio(ratio[1], sum),
			ui.Constraint.Ratio(ratio[2], sum),
			ui.Constraint.Ratio(ratio[3], sum),
		}
		:split(body)
	local current = chunks[2]:pad(ui.Pad.x(1))

	local row = cx.active.current.cursor - cx.active.current.offset
	local row_y = current.y + row

	local below = (body.y + body.h) - (row_y + 1)
	local h, y
	if below >= 8 then
		h, y = math.min(HEIGHT, below), row_y + 1
	else
		local above = row_y - body.y
		h, y = math.min(HEIGHT, above), row_y - math.min(HEIGHT, above)
	end

	self._area = ui.Rect { x = current.x, y = y, w = current.w, h = h }
	return self
end

function M:reflow() return { self } end

-- Layout matches the help box (border, input row, divider, list), plus a
-- two-line bar under a second divider showing the highlighted command.
function M:redraw()
	local area = self._area
	if not self.children or area.h < 8 or area.w < 10 then
		return {}
	end

	local x, y, w, h = area.x, area.y, area.w, area.h
	local input = ui.Rect { x = x + 2, y = y + 1, w = w - 4, h = 1 }
	local divider = ui.Rect { x = x, y = y + 2, w = w, h = 1 }
	local list = ui.Rect { x = x + 1, y = y + 3, w = w - 2, h = h - 7 }
	local divider2 = ui.Rect { x = x, y = y + h - 4, w = w, h = 1 }
	local bar = ui.Rect { x = x + 2, y = y + h - 3, w = w - 4, h = 2 }

	local offset = math.max(0, self.cursor - list.h + 1)
	local lines = {}
	for i = offset + 1, math.min(#self.rows, offset + list.h) do
		local r = self.rows[i]
		local hovered_row = i == self.cursor + 1
		-- Same "> " hovered marker the opener uses, group label on the right.
		local indicator = hovered_row and "> " or "  "
		-- Pad by hand: string.format's width spec caps out well below a
		-- full-terminal-width row, so a dynamic "%-Ns" blows up.
		local desc_w = math.max(0, list.w - #indicator - GROUP_W)
		local desc = #r.desc < desc_w and r.desc .. string.rep(" ", desc_w - #r.desc) or r.desc
		local group = #r.group < GROUP_W and string.rep(" ", GROUP_W - #r.group) .. r.group or r.group
		local line = ui.Line {
			ui.Span(indicator):style(th.help.chord),
			ui.Span(desc):style(th.help.action),
			ui.Span(group):style(th.help.chord),
		}
		lines[#lines + 1] = hovered_row and line:style(th.help.hovered) or line
	end
	if #self.rows == 0 then
		lines[1] = ui.Line(" no matching actions"):style(ui.Style():dim())
	end

	local hovered = self.rows[self.cursor + 1]
	-- A fresh Line each time: ui.Text consumes the one it's given.
	local function rule() return ui.Line("├" .. string.rep("─", w - 2) .. "┤"):style(th.help.border) end

	local count = ui.Rect { x = x + 2, y = y + h - 1, w = w - 4, h = 1 }
	return {
		ui.Clear(area),
		ui.Border(ui.Edge.ALL)
			:area(area)
			:type(ui.Border.ROUNDED)
			:style(th.help.border)
			:title(ui.Line(" Actions: "):align(ui.Align.LEFT)),
		self.input:area(input):focus(true),
		ui.Text(rule()):area(divider),
		ui.List(lines):area(list),
		ui.Text(rule()):area(divider2),
		ui.Text(ui.Line {
			ui.Span("$ "):style(th.help.chord),
			ui.Span(hovered and hovered.cmd or ""):style(ui.Style():dim()),
		})
			:area(bar)
			:wrap(ui.Wrap.YES),
		ui.Text(ui.Line(string.format(" %d/%d ", #self.rows, #self.all)):style(th.help.border))
			:area(count)
			:align(ui.Align.RIGHT),
	}
end

---------------------------------------------------------------------- entry

function M:entry(job)
	local step = ({ down = 1, up = -1 })[job.args[1]]
	if step then
		if not move(step) then
			ya.emit("input:recall", { step })
		end
		return
	end

	local targets = get_targets()
	if #targets == 0 then
		return
	end
	for _, t in ipairs(targets) do
		t.ext = ext_of(t.name)
	end

	-- Yazi's require() hands back a proxy, so `#` and ipairs() can't see a
	-- bare list; items.lua exposes it as a field instead.
	local all = collect(require(".items").groups, targets)
	if #all == 0 then
		return ya.notify { title = "Actions", content = "Nothing applies to this selection", timeout = 3 }
	end

	open(all)
end

return M
