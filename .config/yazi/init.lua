-- Yazi init.lua - startup customization

-- Show keybinding hints on startup
local function show_help_hint()
	ya.notify({
		title = "Keybindings",
		content = [[
? or ~  Full help
gh/gc/gr/gd  Go home/config/repo/downloads
.  Toggle hidden
s  Shell here
Space  Select
y/p/d  Yank/paste/delete
o  Open
q  Quit]],
		timeout = 5,
		level = "info",
	})
end

-- Run on startup
show_help_hint()
