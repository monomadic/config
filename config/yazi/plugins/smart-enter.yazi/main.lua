---@sync entry
return {
	entry = function()
		local hovered = cx.active.current.hovered
		if hovered and hovered.cha.is_dir then
			ya.mgr_emit("enter", {})
		else
			ya.mgr_emit("open", {})
		end
	end,
}
