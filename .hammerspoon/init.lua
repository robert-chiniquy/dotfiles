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

-- Debounced focus - only change focus if mouse stays over window for a bit
lastMouseWindow = nil
mouseHoverTime = 0

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
        mouseHoverTime = mouseHoverTime + 0.1
    else
        lastMouseWindow = windowUnderMouse
        mouseHoverTime = 0
    end

    -- Only call focus after hovering 0.3 seconds
    if mouseHoverTime >= 0.3 then
        focusWindowUnderMouse()
        mouseHoverTime = 0
    end
end

focusFollowsMouse = hs.timer.new(1, focusWithDebounce)
focusFollowsMouse:start()

-- Toggle focus follows mouse (cmd+ctrl+f)
focusFollowsMouseEnabled = true
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

-- WiFi watcher
wifiWatcher = nil
lastSSID = hs.wifi.currentNetwork()

local function wifiChanged()
    local newSSID = hs.wifi.currentNetwork()
    if newSSID ~= lastSSID then
        if newSSID then
            hs.alert.show("WiFi: " .. newSSID)
        else
            hs.alert.show("WiFi: Disconnected")
        end
        lastSSID = newSSID
    end
end

wifiWatcher = hs.wifi.watcher.new(wifiChanged)
wifiWatcher:start()

-- Wallpaper rotation (cmd+ctrl+w to rotate)
wallpaperDir = os.getenv("HOME") .. "/Pictures/Wallpapers"
hs.hotkey.bind({"cmd", "ctrl"}, "w", function()
    local files = {}
    local iter, dir_obj = hs.fs.dir(wallpaperDir)
    if iter then
        for file in iter, dir_obj do
            if file:match("%.png$") or file:match("%.jpg$") or file:match("%.jpeg$") then
                table.insert(files, wallpaperDir .. "/" .. file)
            end
        end
        if #files > 0 then
            local randomWallpaper = files[math.random(#files)]
            for _, screen in ipairs(hs.screen.allScreens()) do
                screen:desktopImageURL("file://" .. randomWallpaper)
            end
            hs.alert.show("Wallpaper changed")
        else
            hs.alert.show("No wallpapers in ~/Pictures/Wallpapers")
        end
    else
        hs.alert.show("Create ~/Pictures/Wallpapers folder")
    end
end)

-- Reload config shortcut (cmd+ctrl+r)
hs.hotkey.bind({"cmd", "ctrl"}, "r", function()
    hs.reload()
end)

-- Floating CPU meter (desktop widget, behind windows)
cpuCanvas = hs.canvas.new({x = 20, y = 60, w = 200, h = 120})
cpuCanvas:level(hs.canvas.windowLevels.desktopIcon)
cpuCanvas:behavior(hs.canvas.windowBehaviors.canJoinAllSpaces)

-- Background
cpuCanvas[1] = {
    type = "rectangle",
    action = "fill",
    fillColor = {red = 0.04, green = 0.04, blue = 0.07, alpha = 0.85},
    roundedRectRadii = {xRadius = 8, yRadius = 8}
}

-- Border
cpuCanvas[2] = {
    type = "rectangle",
    action = "stroke",
    strokeColor = {red = 0.67, green = 0, blue = 0.97, alpha = 0.8},
    strokeWidth = 2,
    roundedRectRadii = {xRadius = 8, yRadius = 8}
}

-- Title
cpuCanvas[3] = {
    type = "text",
    text = "CPU",
    textColor = {red = 0.36, green = 0.93, blue = 1, alpha = 1},
    textFont = "Menlo",
    textSize = 12,
    frame = {x = 10, y = 8, w = 180, h = 20}
}

-- CPU percentage text
cpuCanvas[4] = {
    type = "text",
    text = "0%",
    textColor = {red = 1, green = 0, blue = 0.97, alpha = 1},
    textFont = "Menlo",
    textSize = 28,
    frame = {x = 10, y = 30, w = 180, h = 40}
}

-- Bar background
cpuCanvas[5] = {
    type = "rectangle",
    action = "fill",
    fillColor = {red = 0.2, green = 0.2, blue = 0.25, alpha = 1},
    frame = {x = 10, y = 80, w = 180, h = 12},
    roundedRectRadii = {xRadius = 4, yRadius = 4}
}

-- Bar fill
cpuCanvas[6] = {
    type = "rectangle",
    action = "fill",
    fillColor = {red = 1, green = 0, blue = 0.97, alpha = 1},
    frame = {x = 10, y = 80, w = 0, h = 12},
    roundedRectRadii = {xRadius = 4, yRadius = 4}
}

-- Load average text
cpuCanvas[7] = {
    type = "text",
    text = "load: ...",
    textColor = {red = 0.98, green = 0.72, blue = 0.15, alpha = 1},
    textFont = "Menlo",
    textSize = 10,
    frame = {x = 10, y = 100, w = 180, h = 16}
}

cpuCanvas:show()

-- Update CPU meter
local function updateCpuMeter()
    local cpu = hs.host.cpuUsage()
    local total = cpu.overall.active
    local pct = math.floor(total)

    -- Update percentage text
    cpuCanvas[4].text = pct .. "%"

    -- Update bar width (max 180)
    local barWidth = math.floor(180 * total / 100)
    cpuCanvas[6].frame = {x = 10, y = 80, w = barWidth, h = 12}

    -- Color gradient based on usage
    if total < 50 then
        cpuCanvas[6].fillColor = {red = 0.36, green = 0.93, blue = 1, alpha = 1}  -- cyan
    elseif total < 80 then
        cpuCanvas[6].fillColor = {red = 0.98, green = 0.72, blue = 0.15, alpha = 1}  -- gold
    else
        cpuCanvas[6].fillColor = {red = 1, green = 0, blue = 0.97, alpha = 1}  -- pink
    end

    -- Load average
    local loadavg = hs.execute("sysctl -n vm.loadavg | awk '{print $2, $3, $4}'")
    cpuCanvas[7].text = "load: " .. loadavg:gsub("\n", "")
end

cpuMeterTimer = hs.timer.new(10, updateCpuMeter)
cpuMeterTimer:start()
updateCpuMeter()

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

hs.alert.show("Hammerspoon loaded")
