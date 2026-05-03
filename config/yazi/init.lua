require("dual-pane"):setup()

ps.sub("cd", function()
	ya.emit("plugin", { "kitty-title", tostring(cx.active.current.cwd) })
end)
