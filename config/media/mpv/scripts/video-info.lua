local display_info = false

mp.register_script_message("toggle_info", function()
	display_info = not display_info
	if display_info then
		mp.commandv("show-text", string.format(
			"\\fs20%s\n%dx%d %dmbps %s @ %dfps\n%dmb",
			mp.get_property("media-title"),
			mp.get_property("width"),
			mp.get_property("height"),
			math.floor(mp.get_property_native("video-bitrate") / 1000000),
			mp.get_property("video-codec"),
			math.floor(mp.get_property("fps")),
			math.floor(mp.get_property_native("file-size") / (1024 * 1024))
		))
	else
		mp.commandv("show-text", "")
	end
end)
