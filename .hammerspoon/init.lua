-- Vaporwave colors for alerts
hs.alert.defaultStyle.fillColor = {red = 0.1, green = 0.1, blue = 0.18, alpha = 0.95}
hs.alert.defaultStyle.strokeColor = {red = 0.67, green = 0, blue = 0.91, alpha = 1}
hs.alert.defaultStyle.textColor = {red = 0.36, green = 0.93, blue = 1, alpha = 1}
hs.alert.defaultStyle.textFont = "SF Pro"
hs.alert.defaultStyle.textSize = 16
hs.alert.defaultStyle.radius = 10

-- Focus follows mouse across apps and displays (excludes dialogs)
local function focusWindowUnderMouse()
    local mousePos = hs.mouse.absolutePosition()
    local screen = hs.mouse.getCurrentScreen()
    local windows = hs.window.orderedWindows()

    -- Skip if a dialog/sheet is focused (don't steal focus from it)
    local focused = hs.window.focusedWindow()
    if focused then
        local role = focused:subrole()
        if role == "AXDialog" or role == "AXSheet" or role == "AXSystemDialog" then
            return
        end
    end

    for _, win in ipairs(windows) do
        if win:isStandard() and win:screen() == screen then
            local frame = win:frame()
            if mousePos.x >= frame.x and mousePos.x <= frame.x + frame.w and
               mousePos.y >= frame.y and mousePos.y <= frame.y + frame.h then
                if win ~= focused then
                    local role = win:subrole()
                    if role ~= "AXDialog" and role ~= "AXSheet" and role ~= "AXSystemDialog" then
                        win:focus()
                    end
                end
                return
            end
        end
    end
end

focusFollowsMouse = hs.timer.new(0.3, focusWindowUnderMouse)
focusFollowsMouse:start()

-- Clipboard history
clipboardHistory = {}
clipboardHistorySize = 50
lastClipboardContent = ""

local function updateClipboardHistory()
    local content = hs.pasteboard.getContents()
    if content and content ~= lastClipboardContent and content ~= "" then
        lastClipboardContent = content
        -- Remove if already exists
        for i, v in ipairs(clipboardHistory) do
            if v == content then
                table.remove(clipboardHistory, i)
                break
            end
        end
        -- Add to front
        table.insert(clipboardHistory, 1, content)
        -- Trim to max size
        while #clipboardHistory > clipboardHistorySize do
            table.remove(clipboardHistory)
        end
    end
end

clipboardWatcher = hs.timer.new(0.5, updateClipboardHistory)
clipboardWatcher:start()

-- Show clipboard history chooser (cmd+shift+v)
hs.hotkey.bind({"cmd", "shift"}, "v", function()
    local chooser = hs.chooser.new(function(choice)
        if choice then
            hs.pasteboard.setContents(choice.text)
            hs.eventtap.keyStroke({"cmd"}, "v")
        end
    end)

    local choices = {}
    for i, item in ipairs(clipboardHistory) do
        local preview = item:gsub("\n", " "):sub(1, 80)
        if #item > 80 then preview = preview .. "..." end
        table.insert(choices, {
            text = item,
            subText = "Item " .. i,
            ["text"] = item
        })
    end

    chooser:choices(choices)
    chooser:placeholderText("Clipboard History")
    chooser:show()
end)

-- Window snapping (backup hotkeys, skhd handles most)
-- cmd+ctrl+left = left half
hs.hotkey.bind({"cmd", "ctrl"}, "left", function()
    local win = hs.window.focusedWindow()
    if win then
        local screen = win:screen():frame()
        win:setFrame({x=screen.x, y=screen.y, w=screen.w/2, h=screen.h})
    end
end)

-- cmd+ctrl+right = right half
hs.hotkey.bind({"cmd", "ctrl"}, "right", function()
    local win = hs.window.focusedWindow()
    if win then
        local screen = win:screen():frame()
        win:setFrame({x=screen.x+screen.w/2, y=screen.y, w=screen.w/2, h=screen.h})
    end
end)

-- cmd+ctrl+up = maximize
hs.hotkey.bind({"cmd", "ctrl"}, "up", function()
    local win = hs.window.focusedWindow()
    if win then win:maximize() end
end)

-- cmd+ctrl+down = center at 80%
hs.hotkey.bind({"cmd", "ctrl"}, "down", function()
    local win = hs.window.focusedWindow()
    if win then
        local screen = win:screen():frame()
        local w = screen.w * 0.8
        local h = screen.h * 0.8
        win:setFrame({x=screen.x+(screen.w-w)/2, y=screen.y+(screen.h-h)/2, w=w, h=h})
    end
end)

-- Caffeine (prevent sleep toggle) - cmd+ctrl+c
caffeineEnabled = false
hs.hotkey.bind({"cmd", "ctrl"}, "c", function()
    caffeineEnabled = not caffeineEnabled
    if caffeineEnabled then
        hs.caffeinate.set("displayIdle", true)
        hs.alert.show("Caffeine ON - display won't sleep")
    else
        hs.caffeinate.set("displayIdle", false)
        hs.alert.show("Caffeine OFF")
    end
end)

-- Reload config shortcut (cmd+ctrl+r)
hs.hotkey.bind({"cmd", "ctrl"}, "r", function()
    hs.reload()
end)

hs.alert.show("Hammerspoon loaded")
