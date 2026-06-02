--- @sync entry

local function configured_ratio()
	local ratio = rt and rt.mgr and rt.mgr.ratio or MANAGER.ratio

	return {
		parent = ratio.parent,
		current = ratio.current,
		preview = ratio.preview,
	}
end

local function small_preview(ratio)
	return math.max(1, math.floor(ratio.preview / 2))
end

local function measured_chunks(ratio)
	local width = math.max(1, ratio.parent + ratio.current + ratio.preview)
	local tab = Tab:new(ui.Rect({ x = 0, y = 0, w = width, h = 10 }), cx.active)

	return {
		parent = tab._chunks[1].w,
		current = tab._chunks[2].w,
		preview = tab._chunks[3].w,
	}
end

local function init_state(st)
	if st.parent and st.current and st.preview then
		return configured_ratio()
	end

	local ratio = configured_ratio()
	local chunks = measured_chunks(ratio)

	st.parent = chunks.parent == 0 and 0 or ratio.parent
	st.current = ratio.current
	if chunks.preview == 0 then
		st.preview = 0
	elseif chunks.preview < ratio.preview then
		st.preview = small_preview(ratio)
	else
		st.preview = ratio.preview
	end

	return ratio
end

local function default_layout(st, ratio)
	return st.parent == ratio.parent and st.current == ratio.current and st.preview == ratio.preview
end

local function reset_layout(st)
	if st.original_layout then
		Tab.layout = st.original_layout
	end

	st.original_layout = nil
	st.parent = nil
	st.current = nil
	st.preview = nil
	ya.emit("app:resize", {})
end

local function apply_layout(st, ratio)
	if default_layout(st, ratio) then
		return reset_layout(st)
	end

	if not st.original_layout then
		st.original_layout = Tab.layout
	end

	Tab.layout = function(self)
		local all = st.parent + st.current + st.preview
		if all <= 0 then
			all = 1
		end

		self._chunks = ui.Layout()
			:direction(ui.Layout.HORIZONTAL)
			:constraints({
				ui.Constraint.Ratio(st.parent, all),
				ui.Constraint.Ratio(st.current, all),
				ui.Constraint.Ratio(st.preview, all),
			})
			:split(self._area)
	end

	ya.emit("app:resize", {})
end

local function cycle_preview(st, ratio)
	local small = small_preview(ratio)

	if st.preview == 0 then
		st.preview = ratio.preview
	elseif st.preview < ratio.preview then
		st.preview = 0
	else
		st.preview = small
	end
end

local function toggle_parent(st, ratio)
	st.parent = st.parent == 0 and ratio.parent or 0
end

local function entry(st, job)
	local ratio = init_state(st)
	local args = type(job) == "table" and (job.args or job) or { job }
	local action = args[1]

	if action == "cycle-preview" then
		cycle_preview(st, ratio)
	elseif action == "toggle-parent" then
		toggle_parent(st, ratio)
	else
		return
	end

	apply_layout(st, ratio)
end

return { entry = entry }
