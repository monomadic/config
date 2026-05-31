-- file: prompt.lua
local mp = require "mp"

local INPUT_SECTION = "prompt"
local prompt_active = false

-- Remove temporary bindings
local function clear_prompt_bindings()
	mp.commandv("disable-section", INPUT_SECTION)
	prompt_active = false
end

local function on_option_y()
	mp.osd_message("You chose YES", 2)
	clear_prompt_bindings()
end

local function on_option_n()
	mp.osd_message("You chose NO", 2)
	clear_prompt_bindings()
end

local function show_prompt()
	if prompt_active then return end -- Prevent overlapping prompts
	prompt_active = true
	mp.osd_message("Press Y for YES, N for NO", 5)
	mp.commandv("enable-section", INPUT_SECTION)
end

mp.add_key_binding(nil, "show-prompt", show_prompt)
mp.add_key_binding(nil, "option-y", on_option_y)
mp.add_key_binding(nil, "option-n", on_option_n)
