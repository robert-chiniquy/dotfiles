-- Load IPC for CLI access
require("hs.ipc")

-- Aggressively hide console - do this first, multiple times
hs.closeConsole()
hs.consoleOnTop(false)
hs.dockIcon(false)
hs.autoLaunch(true)
hs.console.clearConsole()
hs.closeConsole()

-- Keep trying to close console
for i = 1, 10 do
    hs.timer.doAfter(i * 0.2, function() hs.closeConsole() end)
end

-- Vaporwave colors for alerts
hs.alert.defaultStyle.fillColor = {red = 0.1, green = 0.1, blue = 0.18, alpha = 0.95}
hs.alert.defaultStyle.strokeColor = {red = 0.67, green = 0, blue = 0.91, alpha = 1}
hs.alert.defaultStyle.textColor = {red = 0.36, green = 0.93, blue = 1, alpha = 1}
hs.alert.defaultStyle.textFont = ".AppleSystemUIFont"
hs.alert.defaultStyle.textSize = 16
hs.alert.defaultStyle.radius = 10
hs.alert.defaultStyle.fadeInDuration = 0.1
hs.alert.defaultStyle.fadeOutDuration = 0.3
hs.alert.defaultStyle.atScreenEdge = 0
hs.alert.defaultStyle.strokeWidth = 0

-- All alerts auto-dismiss in 1.5s
local _origAlert = hs.alert.show
hs.alert.show = function(msg, style, screen, duration)
    return _origAlert(msg, style, screen, duration or 1.5)
end

-- PaperWM tiling window manager
PaperWM = hs.loadSpoon("PaperWM")
PaperWM.window_gap = 8
PaperWM.external_bar = {top = 80}
PaperWM.swipe_fingers = 4
PaperWM.center_mouse = false

-- Exclude non-tiling apps from PaperWM
PaperWM.window_filter:rejectApp("System Settings")
PaperWM.window_filter:rejectApp("System Preferences")
PaperWM.window_filter:rejectApp("Archive Utility")
PaperWM.window_filter:rejectApp("Calculator")
PaperWM.window_filter:rejectApp("Finder")

PaperWM:bindHotkeys({
    -- Focus navigation: cmd+ctrl + arrows
    focus_left  = {{"cmd", "ctrl"}, "left"},
    focus_right = {{"cmd", "ctrl"}, "right"},
    focus_up    = {{"cmd", "ctrl"}, "up"},
    focus_down  = {{"cmd", "ctrl"}, "down"},

    -- Swap windows: cmd+ctrl+shift + arrows
    swap_left  = {{"cmd", "ctrl", "shift"}, "left"},
    swap_right = {{"cmd", "ctrl", "shift"}, "right"},
    swap_up    = {{"cmd", "ctrl", "shift"}, "up"},
    swap_down  = {{"cmd", "ctrl", "shift"}, "down"},

    -- Sizing: center, full width, cycle width
    center_window = {{"cmd", "ctrl"}, "c"},
    full_width    = {{"cmd", "ctrl"}, "m"},
    cycle_width   = {{"cmd", "ctrl"}, "="},

    -- Column stacking: slurp into left column / barf out to own column
    slurp_in = {{"cmd", "ctrl"}, "i"},
    barf_out = {{"cmd", "ctrl"}, "o"},

    -- Retile all windows (fix dragging mess)
    refresh_windows = {{"cmd", "ctrl"}, "-"},

    -- Float toggle
    toggle_floating = {{"cmd", "ctrl"}, "t"},

    -- Switch spaces: cmd+ctrl + 1-9
    switch_space_1 = {{"cmd", "ctrl"}, "1"},
    switch_space_2 = {{"cmd", "ctrl"}, "2"},
    switch_space_3 = {{"cmd", "ctrl"}, "3"},
    switch_space_4 = {{"cmd", "ctrl"}, "4"},
    switch_space_5 = {{"cmd", "ctrl"}, "5"},
    switch_space_6 = {{"cmd", "ctrl"}, "6"},
    switch_space_7 = {{"cmd", "ctrl"}, "7"},
    switch_space_8 = {{"cmd", "ctrl"}, "8"},
    switch_space_9 = {{"cmd", "ctrl"}, "9"},

    -- Move window to space: cmd+ctrl+shift + 1-9
    move_window_1 = {{"cmd", "ctrl", "shift"}, "1"},
    move_window_2 = {{"cmd", "ctrl", "shift"}, "2"},
    move_window_3 = {{"cmd", "ctrl", "shift"}, "3"},
    move_window_4 = {{"cmd", "ctrl", "shift"}, "4"},
    move_window_5 = {{"cmd", "ctrl", "shift"}, "5"},
    move_window_6 = {{"cmd", "ctrl", "shift"}, "6"},
    move_window_7 = {{"cmd", "ctrl", "shift"}, "7"},
    move_window_8 = {{"cmd", "ctrl", "shift"}, "8"},
    move_window_9 = {{"cmd", "ctrl", "shift"}, "9"},
})
PaperWM:start()


-- Auto-slurp: new windows from the same app get stacked into one column
-- Waits 2s after startup to avoid slurping everything on reload
local autoSlurpReady = false
hs.timer.doAfter(2, function() autoSlurpReady = true end)

local autoSlurpWatcher = hs.window.filter.new():setDefaultFilter()
autoSlurpWatcher:subscribe(hs.window.filter.windowVisible, function(newWin)
    if not autoSlurpReady then return end
    if not newWin or not newWin:isStandard() then return end
    local newApp = newWin:application()
    if not newApp then return end
    local newAppName = newApp:name()

    -- Skip apps that PaperWM doesn't tile
    local skipApps = {
        ["System Settings"] = true, ["System Preferences"] = true,
        ["Archive Utility"] = true, ["Calculator"] = true, ["Finder"] = true,
    }
    if skipApps[newAppName] then return end

    -- Wait for PaperWM to tile the new window into its own column
    hs.timer.doAfter(0.6, function()
        local newIndex = PaperWM.state.windowIndex(newWin)
        if not newIndex then return end

        -- Find the column containing a same-app window
        local columns = PaperWM.state.windowList(newIndex.space)
        if not columns then return end

        local targetCol = nil
        for col = 1, #columns do
            if col ~= newIndex.col then
                local column = columns[col]
                for row = 1, #column do
                    local existingWin = column[row]
                    local existingApp = existingWin and existingWin:application()
                    if existingApp and existingApp:name() == newAppName then
                        targetCol = col
                        break
                    end
                end
                if targetCol then break end
            end
        end

        if not targetCol then return end

        -- Re-fetch index (may have shifted)
        newIndex = PaperWM.state.windowIndex(newWin)
        if not newIndex then return end

        -- Move: remove from current column, append to target column
        local srcColumn = PaperWM.state.windowList(newIndex.space, newIndex.col)
        local dstColumn = PaperWM.state.windowList(newIndex.space, targetCol)
        if not srcColumn or not dstColumn then return end

        -- Remove from source
        local removed = table.remove(srcColumn, newIndex.row)
        if not removed then return end

        -- Append to target
        table.insert(dstColumn, removed)

        -- Re-tile the space
        PaperWM:tileSpace(newIndex.space)
    end)
end)

-- Apps to exclude from focus follows mouse
excludedApps = {
    "System Settings",
    "System Preferences",
    "Finder",
    "Archive Utility",
    "Calculator",
    "zoom.us",
    "WorkFlowy",
    "Workflowy"
}

local function isExcludedApp(appName)
    for _, name in ipairs(excludedApps) do
        if appName == name then return true end
    end
    return false
end

-- Simple focus follows mouse
local function focusWindowUnderMouse()
    local mousePos = hs.mouse.absolutePosition()
    local focused = hs.window.focusedWindow()

    -- Skip if excluded app is focused
    if focused then
        local app = focused:application()
        if app and isExcludedApp(app:name()) then
            return
        end
    end

    for _, win in ipairs(hs.window.orderedWindows()) do
        if win:isStandard() then
            local frame = win:frame()
            if mousePos.x >= frame.x and mousePos.x <= frame.x + frame.w and
               mousePos.y >= frame.y and mousePos.y <= frame.y + frame.h then
                if win ~= focused then
                    win:focus()
                end
                return
            end
        end
    end
end

-- Debounced focus - only change focus if mouse stays over window for 0.3s
-- Timer runs at 0.15s intervals; debounce counts actual elapsed ticks
local FFM_INTERVAL = 0.5
local FFM_DWELL = 0.3
lastMouseWindow = nil
mouseHoverTicks = 0

local function focusWithDebounce()
    local mousePos = hs.mouse.absolutePosition()
    local windowUnderMouse = nil

    for _, win in ipairs(hs.window.orderedWindows()) do
        if win:isStandard() then
            local frame = win:frame()
            if mousePos.x >= frame.x and mousePos.x <= frame.x + frame.w and
               mousePos.y >= frame.y and mousePos.y <= frame.y + frame.h then
                windowUnderMouse = win
                break
            end
        end
    end

    if windowUnderMouse == lastMouseWindow then
        mouseHoverTicks = mouseHoverTicks + 1
    else
        lastMouseWindow = windowUnderMouse
        mouseHoverTicks = 0
    end

    -- Focus after dwelling FFM_DWELL seconds (ticks * interval)
    if mouseHoverTicks * FFM_INTERVAL >= FFM_DWELL then
        focusWindowUnderMouse()
        mouseHoverTicks = 0
    end
end

focusFollowsMouse = hs.timer.new(FFM_INTERVAL, focusWithDebounce)
-- FFM disabled by default — fights PaperWM's tiling/scrolling
-- Toggle with cmd+ctrl+f if you want it back per-session

-- Toggle focus follows mouse (cmd+ctrl+f)
focusFollowsMouseEnabled = false
hs.hotkey.bind({"cmd", "ctrl"}, "f", function()
    focusFollowsMouseEnabled = not focusFollowsMouseEnabled
    if focusFollowsMouseEnabled then
        focusFollowsMouse:start()
        hs.alert.show("Focus follows mouse ON")
    else
        focusFollowsMouse:stop()
        hs.alert.show("Focus follows mouse OFF")
    end
end)

-- Window snapping removed — PaperWM handles tiling

-- Caffeine (prevent sleep toggle) - cmd+ctrl+shift+c (moved from cmd+ctrl+c for PaperWM)
caffeineEnabled = false
hs.hotkey.bind({"cmd", "ctrl", "shift"}, "c", function()
    caffeineEnabled = not caffeineEnabled
    if caffeineEnabled then
        hs.caffeinate.set("displayIdle", true)
        hs.alert.show("Caffeine ON - display won't sleep")
    else
        hs.caffeinate.set("displayIdle", false)
        hs.alert.show("Caffeine OFF")
    end
end)


-- Wallpaper rotation (auto-rotate hourly + cmd+ctrl+w to manual rotate)
wallpaperDir = os.getenv("HOME") .. "/Pictures/dynamic-wallpaper"

local function rotateWallpaper()
    local files = {}
    local ok, iter, dir_obj = pcall(hs.fs.dir, wallpaperDir)
    if ok and iter then
        for file in iter, dir_obj do
            if file:match("%.png$") or file:match("%.jpg$") or file:match("%.jpeg$") or file:match("%.heic$") then
                table.insert(files, wallpaperDir .. "/" .. file)
            end
        end
        if #files > 0 then
            local randomWallpaper = files[math.random(#files)]
            for _, screen in ipairs(hs.screen.allScreens()) do
                screen:desktopImageURL("file://" .. randomWallpaper)
            end
            return true
        end
    end
    return false
end

-- Auto-rotate every hour
wallpaperTimer = hs.timer.new(30, rotateWallpaper)
wallpaperTimer:start()
rotateWallpaper()

-- Manual rotate still available
hs.hotkey.bind({"cmd", "ctrl"}, "w", function()
    if rotateWallpaper() then
        hs.alert.show("Wallpaper changed")
    else
        hs.alert.show("No wallpapers in ~/Pictures/dynamic-wallpaper")
    end
end)

-- Reload config shortcut (cmd+ctrl+r)
hs.hotkey.bind({"cmd", "ctrl"}, "r", function()
    hs.reload()
end)

-- CPU meter moved to Sketchybar (right side: cpu + load items)

-- Vaporwave overlay - use 'vw' shell alias instead (Hammerspoon integration was unreliable)
-- vw alias: pkill -9 -f vaporwave-overlay 2>/dev/null && echo Off || (~/Applications/VaporwaveOverlay.app/Contents/MacOS/vaporwave-overlay --fullscreen & echo On)

-- Pomodoro Timer for Ubersicht widget
-- State file: /tmp/pomodoro-state (format: state|startTime|totalSeconds)
-- States: idle, work, break, paused

local POMODORO_WORK_SECONDS = 25 * 60
local POMODORO_BREAK_SECONDS = 5 * 60
local pomodoroTimer = nil
local pomodoroState = "idle"
local pomodoroStartTime = 0
local pomodoroPausedProgress = 0

local function writePomodoroState()
    local f = io.open("/tmp/pomodoro-state", "w")
    if f then
        f:write(pomodoroState .. "|" .. pomodoroStartTime .. "|" .. pomodoroPausedProgress)
        f:close()
    end
end

local function pomodoroCheck()
    local now = os.time()
    if pomodoroState == "work" then
        local elapsed = now - pomodoroStartTime
        if elapsed >= POMODORO_WORK_SECONDS then
            -- Work complete, start break
            pomodoroState = "break"
            pomodoroStartTime = now
            writePomodoroState()
            hs.alert.show("RUBEDO - Work complete. Rest now.")
            hs.sound.getByFile(os.getenv("HOME") .. "/Library/Sounds/gong.aiff"):play()
        end
    elseif pomodoroState == "break" then
        local elapsed = now - pomodoroStartTime
        if elapsed >= POMODORO_BREAK_SECONDS then
            -- Break complete, return to idle
            pomodoroState = "idle"
            pomodoroStartTime = 0
            writePomodoroState()
            hs.alert.show("REBIRTH - Ready for new work")
            hs.sound.getByFile(os.getenv("HOME") .. "/Library/Sounds/crystal.aiff"):play()
            if pomodoroTimer then
                pomodoroTimer:stop()
                pomodoroTimer = nil
            end
        end
    end
end

local function startPomodoro(startTime)
    if pomodoroState == "idle" then
        pomodoroState = "work"
        pomodoroStartTime = startTime or os.time()
        hs.alert.show("NIGREDO - Begin the work")
    elseif pomodoroState == "paused" then
        -- Resume from paused state
        pomodoroState = "work"
        -- Adjust start time based on how much was already done
        pomodoroStartTime = os.time() - pomodoroPausedProgress
        hs.alert.show("Resuming work")
    end
    writePomodoroState()

    if not pomodoroTimer then
        pomodoroTimer = hs.timer.new(1, pomodoroCheck)
        pomodoroTimer:start()
    end
end

local function pausePomodoro()
    if pomodoroState == "work" then
        local elapsed = os.time() - pomodoroStartTime
        pomodoroPausedProgress = elapsed
        pomodoroState = "paused"
        writePomodoroState()
        hs.alert.show("STASIS - Work paused")
        if pomodoroTimer then
            pomodoroTimer:stop()
            pomodoroTimer = nil
        end
    elseif pomodoroState == "break" then
        -- Can't pause break, but can skip to idle
        pomodoroState = "idle"
        pomodoroStartTime = 0
        writePomodoroState()
        hs.alert.show("Break skipped")
        if pomodoroTimer then
            pomodoroTimer:stop()
            pomodoroTimer = nil
        end
    end
end

-- URL handler for pomodoro://action
hs.urlevent.bind("pomodoro", function(eventName, params)
    local action = params.action
    if action == "start" then
        local time = tonumber(params.time) or os.time()
        startPomodoro(time)
    elseif action == "pause" then
        pausePomodoro()
    end
end)

-- Initialize state file
writePomodoroState()

-- URL handler for tarot://shuffle
hs.urlevent.bind("tarot", function(eventName, params)
    if params.action == "shuffle" then
        local f = io.open("/tmp/tarot-shuffle", "w")
        if f then
            f:write(tostring(os.time()))
            f:close()
        end
        hs.alert.show("Cards shuffled")
    end
end)

-- URL handler for iching://cast
hs.urlevent.bind("iching", function(eventName, params)
    if params.action == "cast" then
        local f = io.open("/tmp/iching-cast", "w")
        if f then
            f:write(tostring(os.time()))
            f:close()
        end
        hs.alert.show("Yarrow stalks cast")
    end
end)

-- URL handler for git://open - open repo in iTerm2
hs.urlevent.bind("git", function(eventName, params)
    if params.action == "open" and params.path then
        local path = hs.http.urlParts(params.path) and params.path or os.getenv("HOME") .. "/repo"
        -- URL decode the path
        path = path:gsub("%%(%x%x)", function(hex)
            return string.char(tonumber(hex, 16))
        end)
        hs.osascript.applescript([[
            tell application "iTerm2"
                activate
                create window with default profile
                tell current session of current window
                    write text "cd ]] .. path .. [["
                end tell
            end tell
        ]])
    end
end)

-- URL handler for finder://path - open Ghostty at path
hs.urlevent.bind("finder", function(eventName, params)
    if params.path then
        local path = params.path:gsub("%%(%x%x)", function(hex)
            return string.char(tonumber(hex, 16))
        end)
        hs.execute("/Applications/Ghostty.app/Contents/MacOS/ghostty --working-directory=" .. path .. " &")
    end
end)

-- Spotify media key control
mediaKeyWatcher = hs.eventtap.new({hs.eventtap.event.types.systemDefined}, function(event)
    local data = event:systemKey()
    if data and data.down then
        if data.key == "PLAY" then
            hs.spotify.playpause()
            return true
        elseif data.key == "PREVIOUS" then
            hs.spotify.previous()
            return true
        elseif data.key == "NEXT" then
            hs.spotify.next()
            return true
        end
    end
    return false
end)
mediaKeyWatcher:start()

-- Auto-manage vaporwave overlay based on power state
-- Runs when plugged in + battery > 50%, stops otherwise
-- Uses globals to prevent GC, checks process existence before starting

-- VW overlay managed via LaunchAgent and `vw` CLI, not Hammerspoon
-- Auto-start removed: hs.task.new triggers macOS permission dialogs on every launch
hs.alert.show("Hammerspoon loaded")
