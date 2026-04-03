-- Enhanced Alacritty toggle with window management
-- Alacritty toggle functionality with debugging
local function toggleAlacritty()
    -- Debug notification to confirm the function is being called
    hs.notify.new({title="Debug", informativeText="Toggle function called!"}):send()
    
    local app = hs.application.find("Alacritty")
    
    if app then
        if app:isFrontmost() then
            -- If Alacritty is focused, hide it
            app:hide()
            hs.notify.new({title="Alacritty", informativeText="Hidden"}):send()
        else
            -- If Alacritty exists but isn't focused, bring it to front
            app:activate()
            hs.notify.new({title="Alacritty", informativeText="Activated"}):send()
        end
    else
        -- If Alacritty isn't running, launch it
        hs.application.launchOrFocus("Alacritty")
        hs.notify.new({title="Alacritty", informativeText="Launched"}):send()
    end
end

-- Try multiple ways to bind Cmd+Enter
local hotkey1 = hs.hotkey.bind({"cmd"}, "k", toggleAlacritty)

-- Debug: Show which hotkeys were created
if hotkey1 then
    hs.notify.new({title="Hammerspoon", informativeText="Hotkey 'return' bound successfully"}):send()
else
    hs.notify.new({title="Hammerspoon", informativeText="Failed to bind 'return' key"}):send()
end

if hotkey2 then
    hs.notify.new({title="Hammerspoon", informativeText="Hotkey 'enter' bound successfully"}):send()
else
    hs.notify.new({title="Hammerspoon", informativeText="Failed to bind 'enter' key"}):send()
end

-- Show config loaded notification
hs.notify.new({title="Hammerspoon", informativeText="Config reloaded - try Cmd+Enter"}):send()
