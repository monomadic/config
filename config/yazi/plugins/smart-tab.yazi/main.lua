---@sync entry
return {
	entry = function()
		local hovered = cx.active.current.hovered
		if hovered and hovered.cha.is_dir then
			ya.mgr_emit("tab_create", { hovered.url })
		else
			ya.mgr_emit("tab_create", { current = true })
		end
	end,
}
