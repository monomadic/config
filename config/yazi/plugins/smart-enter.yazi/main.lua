---@sync entry
return {
	entry = function()
		local hovered = cx.active.current.hovered
		if hovered and hovered.cha.is_dir then
			ya.emit("enter", {})
		else
			ya.emit("open", {})
		end
	end,
}
