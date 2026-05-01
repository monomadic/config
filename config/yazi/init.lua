ps.sub("cd", function()
	ya.mgr_emit("plugin", { "kitty-title", tostring(cx.active.current.cwd) })
end)
