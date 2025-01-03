-- hs.hotkey.bind({ "cmd" }, "return", function()
-- 	-- Attempt to discover KITTY_LISTEN_ON dynamically
-- 	local socketPath = hs.execute(
-- 		"ps aux | grep '[k]itty' | grep -- '--listen-on' | awk -F'--listen-on=' '{print $2}' | awk '{print $1}'")
-- 	socketPath = socketPath:gsub("\n", "") -- Remove trailing newline
--
-- 	-- Fallback to a default socket path if discovery fails
-- 	if not socketPath or socketPath == "" then
-- 		socketPath = "unix:/tmp/kitty-socket" -- Replace with your fixed socket path if needed
-- 	end
--
-- 	-- Build and execute the Kitty command
-- 	local kittyPath = "/opt/homebrew/bin/kitty"
-- 	local kittyCommand = string.format('%s @ --to %s launch --type tab --cwd "%s"', kittyPath, socketPath,
-- 		os.getenv("HOME"))
-- 	local output, status, type, rc = hs.execute(kittyCommand, true)
--
-- 	-- Show debugging output
-- 	if not status then
-- 		hs.alert.show(string.format("Failed: %s", output or "No output"))
-- 	else
-- 		hs.alert.show("Kitty tab launched successfully!")
-- 	end
-- end)

hs.hotkey.bind({"cmd", "alt"}, "return", function()
    -- Define the Kitty command
    local kitty_cmd = "/Applications/kitty.app/Contents/MacOS/kitty @ new-window yazi"
    
    -- Execute the command in the shell
    hs.execute(kitty_cmd)
end)
