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

-- Space-separated terms, lowercased.
local function terms_of(query)
	local terms = {}
	for w in query:lower():gmatch("%S+") do
		terms[#terms + 1] = w
	end
	return terms
end

-- Split `text` into spans, styling every substring matching a term `match`
-- and the rest `base`. Overlapping/adjacent matches are merged.
local function highlight(text, terms, base, match)
	if #terms == 0 or text == "" then
		return { ui.Span(text):style(base) }
	end
	local lower = text:lower()
	local marks = {}
	for _, w in ipairs(terms) do
		local i = 1
		while true do
			local s, e = lower:find(w, i, true)
			if not s then
				break
			end
			marks[#marks + 1] = { s, e }
			i = e + 1
		end
	end
	if #marks == 0 then
		return { ui.Span(text):style(base) }
	end
	table.sort(marks, function(a, b) return a[1] < b[1] end)
	local merged = {}
	for _, m in ipairs(marks) do
		local last = merged[#merged]
		if last and m[1] <= last[2] + 1 then
			last[2] = math.max(last[2], m[2])
		else
			merged[#merged + 1] = { m[1], m[2] }
		end
	end
	local spans, pos = {}, 1
	for _, m in ipairs(merged) do
		if m[1] > pos then
			spans[#spans + 1] = ui.Span(text:sub(pos, m[1] - 1)):style(base)
		end
		spans[#spans + 1] = ui.Span(text:sub(m[1], m[2])):style(match)
		pos = m[2] + 1
	end
	if pos <= #text then
		spans[#spans + 1] = ui.Span(text:sub(pos)):style(base)
	end
	return spans
end

-- Space-separated terms, each must appear somewhere in "group desc".
local function filter(rows, query)
	local terms = terms_of(query)
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

-- Flat, borderless layout (Telescope-style): no box, no dividers, just a
-- filled backdrop with a search row on top, the list, and a command preview
-- row pinned to the bottom.
function M:redraw()
	local area = self._area
	if not self.children or area.h < 6 or area.w < 20 then
		return {}
	end

	local x, y, w, h = area.x, area.y, area.w, area.h
	local input = ui.Rect { x = x + 3, y = y, w = w - 3 - 10, h = 1 }
	local count = ui.Rect { x = x + w - 10, y = y, w = 9, h = 1 }
	local list = ui.Rect { x = x + 1, y = y + 1, w = w - 2, h = h - 2 }
	local bar = ui.Rect { x = x + 1, y = y + h - 1, w = w - 2, h = 1 }

	-- Backdrop: a flat panel of solid background, drawn once behind
	-- everything else so unfilled row width still reads as "inside the box".
	local backdrop_lines = {}
	for _ = 1, h do
		backdrop_lines[#backdrop_lines + 1] = ui.Line(string.rep(" ", w)):style(th.help.bg)
	end

	local terms = terms_of(self.query)
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
		local group = #r.group < GROUP_W and string.rep(" ", GROUP_W - #r.group) .. r.group or r.group

		local spans = { ui.Span(indicator):style(th.help.chord) }
		for _, s in ipairs(highlight(r.desc, terms, th.help.action, th.help.match)) do
			spans[#spans + 1] = s
		end
		if #r.desc < desc_w then
			spans[#spans + 1] = ui.Span(string.rep(" ", desc_w - #r.desc)):style(th.help.action)
		end
		spans[#spans + 1] = ui.Span(group):style(th.help.chord)

		local line = ui.Line(spans)
		lines[#lines + 1] = line:style(hovered_row and th.help.hovered or th.help.bg)
	end
	if #self.rows == 0 then
		lines[1] = ui.Line(" no matching actions"):style(ui.Style():dim())
	end

	local hovered = self.rows[self.cursor + 1]

	return {
		ui.Clear(area),
		ui.Text(backdrop_lines):area(area),
		ui.Text(ui.Line { ui.Span("  "):style(th.help.chord) }):area(ui.Rect { x = x, y = y, w = 3, h = 1 }),
		self.input:area(input):focus(true),
		ui.Text(ui.Line(string.format("%d/%d", #self.rows, #self.all)):style(th.help.border))
			:area(count)
			:align(ui.Align.RIGHT),
		ui.List(lines):area(list),
		ui.Text(ui.Line {
			ui.Span("$ "):style(th.help.chord),
			ui.Span(hovered and hovered.cmd or ""):style(ui.Style():dim()),
		})
			:area(bar)
			:style(th.help.bg)
			:wrap(ui.Wrap.YES),
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
