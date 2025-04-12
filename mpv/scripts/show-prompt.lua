-- file: prompt.lua
local prompt_active = false

-- Remove temporary bindings
local function clear_prompt_bindings()
	mp.remove_key_binding("prompt-option-y")
	mp.remove_key_binding("prompt-option-n")
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
	mp.add_forced_key_binding("y", "prompt-option-y", on_option_y)
	mp.add_forced_key_binding("n", "prompt-option-n", on_option_n)
end

-- Bind a key to launch the prompt (e.g. "p")
mp.add_key_binding("p", "show-prompt", show_prompt)
