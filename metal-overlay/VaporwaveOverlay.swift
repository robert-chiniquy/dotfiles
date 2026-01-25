// VaporwaveOverlay - Metal shader window overlay for macOS
// Renders vaporwave effects over unfocused windows only
// Use --fullscreen flag to cover entire screen

import Cocoa

// Global flag for fullscreen mode
var fullscreenMode = CommandLine.arguments.contains("--fullscreen")
import Metal
import MetalKit
import QuartzCore
import CoreGraphics
import ScreenCaptureKit

// Debug logging to file
let debugLogPath = "/tmp/vaporwave-debug.log"
func debugLog(_ message: String) {
    let timestamp = DateFormatter.localizedString(from: Date(), dateStyle: .none, timeStyle: .medium)
    let line = "[\(timestamp)] \(message)\n"
    if let handle = FileHandle(forWritingAtPath: debugLogPath) {
        handle.seekToEndOfFile()
        handle.write(line.data(using: .utf8)!)
        handle.closeFile()
    } else {
        FileManager.default.createFile(atPath: debugLogPath, contents: line.data(using: .utf8), attributes: nil)
    }
    print(message)  // Also print to console
}

// Shared shader resources - compiled ONCE at startup
class ShaderResources {
    static let shared = ShaderResources()

    let device: MTLDevice
    let pipelineState: MTLRenderPipelineState
    let startTime: CFAbsoluteTime

    private init?() {
        guard let dev = MTLCreateSystemDefaultDevice() else { return nil }
        self.device = dev
        self.startTime = CFAbsoluteTimeGetCurrent()

        // Compile shader ONCE
        let shaderPath = Bundle.main.path(forResource: "VaporwaveShader", ofType: "metal")
            ?? (FileManager.default.currentDirectoryPath + "/VaporwaveShader.metal")

        guard let shaderSource = try? String(contentsOfFile: shaderPath, encoding: .utf8),
              let library = try? dev.makeLibrary(source: shaderSource, options: nil),
              let vertexFunc = library.makeFunction(name: "vaporwave_vertex"),
              let fragmentFunc = library.makeFunction(name: "vaporwave_fragment") else {
            print("Failed to compile shader from: \(shaderPath)")
            return nil
        }

        let descriptor = MTLRenderPipelineDescriptor()
        descriptor.vertexFunction = vertexFunc
        descriptor.fragmentFunction = fragmentFunc
        descriptor.colorAttachments[0].pixelFormat = .bgra8Unorm
        descriptor.colorAttachments[0].isBlendingEnabled = true
        descriptor.colorAttachments[0].sourceRGBBlendFactor = .sourceAlpha
        descriptor.colorAttachments[0].destinationRGBBlendFactor = .oneMinusSourceAlpha
        descriptor.colorAttachments[0].sourceAlphaBlendFactor = .one
        descriptor.colorAttachments[0].destinationAlphaBlendFactor = .oneMinusSourceAlpha

        guard let pipeline = try? dev.makeRenderPipelineState(descriptor: descriptor) else {
            print("Failed to create pipeline")
            return nil
        }
        self.pipelineState = pipeline
        print("Shader compiled successfully")
    }
}

class ShaderRenderer: NSObject, MTKViewDelegate {
    let commandQueue: MTLCommandQueue
    var opacity: Float = 0.0  // Controlled by overlay window
    var isPaused: Bool = false
    weak var overlayWindow: OverlayWindow?
    var windowTexture: MTLTexture?
    var textureLoader: MTKTextureLoader?
    var lastTextureUpdate: CFAbsoluteTime = 0
    let textureUpdateInterval: CFAbsoluteTime = 0.5  // Update texture every 0.5 seconds
    var animationStartTime: CFAbsoluteTime = CFAbsoluteTimeGetCurrent()  // Reset on focus change

    init?(device: MTLDevice) {
        guard let queue = device.makeCommandQueue() else { return nil }
        self.commandQueue = queue
        self.textureLoader = MTKTextureLoader(device: device)
        super.init()
    }

    var isCapturing = false
    var isFullscreenCapture = false  // Set to true for fullscreen mode
    var screenIndex: Int = 0  // Which screen to capture in fullscreen mode

    func updateWindowTexture(device: MTLDevice) {
        guard let window = overlayWindow else { return }
        let now = CFAbsoluteTimeGetCurrent()
        if now - lastTextureUpdate < textureUpdateInterval { return }
        if isCapturing { return }  // Don't start new capture while one is pending
        lastTextureUpdate = now
        isCapturing = true

        let frameWidth = Int(window.frame.width)
        let frameHeight = Int(window.frame.height)

        if isFullscreenCapture {
            // Fullscreen mode: capture entire screen
            Task { @MainActor in
                do {
                    let content = try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true)
                    let screens = NSScreen.screens
                    guard self.screenIndex < screens.count,
                          self.screenIndex < content.displays.count else {
                        self.isCapturing = false
                        return
                    }

                    let display = content.displays[self.screenIndex]
                    let filter = SCContentFilter(display: display, excludingWindows: [])
                    let config = SCStreamConfiguration()
                    config.width = frameWidth
                    config.height = frameHeight
                    config.pixelFormat = kCVPixelFormatType_32BGRA
                    config.showsCursor = false

                    let image = try await SCScreenshotManager.captureImage(contentFilter: filter, configuration: config)

                    do {
                        self.windowTexture = try await self.textureLoader?.newTexture(cgImage: image, options: [
                            .textureUsage: MTLTextureUsage.shaderRead.rawValue,
                            .textureStorageMode: MTLStorageMode.private.rawValue
                        ])
                    } catch {
                        // Silently fail
                    }
                    self.isCapturing = false
                } catch {
                    self.isCapturing = false
                }
            }
        } else {
            // Window mode: capture specific window
            let windowID = window.targetWindowID
            Task { @MainActor in
                do {
                    let content = try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true)
                    guard let scWindow = content.windows.first(where: { $0.windowID == windowID }) else {
                        self.isCapturing = false
                        return
                    }

                    let filter = SCContentFilter(desktopIndependentWindow: scWindow)
                    let config = SCStreamConfiguration()
                    config.width = frameWidth
                    config.height = frameHeight
                    config.pixelFormat = kCVPixelFormatType_32BGRA
                    config.showsCursor = false

                    let image = try await SCScreenshotManager.captureImage(contentFilter: filter, configuration: config)

                    do {
                        self.windowTexture = try await self.textureLoader?.newTexture(cgImage: image, options: [
                            .textureUsage: MTLTextureUsage.shaderRead.rawValue,
                            .textureStorageMode: MTLStorageMode.private.rawValue
                        ])
                    } catch {
                        // Silently fail
                    }
                    self.isCapturing = false
                } catch {
                    self.isCapturing = false
                }
            }
        }
    }

    func updateOpacity() {
        guard let window = overlayWindow else { return }
        // At 30 FPS: 0.001 per frame = 1000 frames = ~33 seconds to full opacity
        let fadeInSpeed: Float = isFullscreenCapture ? 0.001 : 0.008
        let fadeOutSpeed: Float = 0.05

        if window.currentOpacity < window.targetOpacity {
            window.currentOpacity = min(window.currentOpacity + fadeInSpeed, window.targetOpacity)
        } else if window.currentOpacity > window.targetOpacity {
            window.currentOpacity = max(window.currentOpacity - fadeOutSpeed, window.targetOpacity)
        }
        self.opacity = window.currentOpacity
    }

    func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {}

    func draw(in view: MTKView) {
        if isPaused { return }
        guard let resources = ShaderResources.shared else { return }

        // Update fade animation
        updateOpacity()

        // Skip drawing if fully transparent
        if opacity < 0.001 { return }

        // Update window texture periodically for purple detection
        updateWindowTexture(device: resources.device)

        guard let drawable = view.currentDrawable,
              let commandBuffer = commandQueue.makeCommandBuffer() else {
            return
        }

        // Explicit render pass descriptor with clear action
        let descriptor = MTLRenderPassDescriptor()
        descriptor.colorAttachments[0].texture = drawable.texture
        descriptor.colorAttachments[0].loadAction = .clear
        descriptor.colorAttachments[0].storeAction = .store
        descriptor.colorAttachments[0].clearColor = MTLClearColor(red: 0, green: 0, blue: 0, alpha: 0)

        guard let encoder = commandBuffer.makeRenderCommandEncoder(descriptor: descriptor) else {
            return
        }

        var time = Float(CFAbsoluteTimeGetCurrent() - animationStartTime) * 1000.0
        var opacity = self.opacity
        var hasTexture: Float = windowTexture != nil ? 1.0 : 0.0
        var windowSeed: Float = Float(overlayWindow?.targetWindowID ?? 0)

        encoder.setRenderPipelineState(resources.pipelineState)
        encoder.setFragmentBytes(&time, length: MemoryLayout<Float>.size, index: 0)
        encoder.setFragmentBytes(&opacity, length: MemoryLayout<Float>.size, index: 1)
        encoder.setFragmentBytes(&hasTexture, length: MemoryLayout<Float>.size, index: 2)
        encoder.setFragmentBytes(&windowSeed, length: MemoryLayout<Float>.size, index: 3)
        if let texture = windowTexture {
            encoder.setFragmentTexture(texture, index: 0)
        }
        encoder.drawPrimitives(type: .triangleStrip, vertexStart: 0, vertexCount: 4)
        encoder.endEncoding()

        commandBuffer.present(drawable)
        commandBuffer.commit()
    }
}

class OverlayWindow: NSWindow {
    var targetWindowID: CGWindowID = 0
    var mtkView: MTKView?
    var renderer: ShaderRenderer?
    var targetOpacity: Float = 1.0
    var currentOpacity: Float = 0.0  // Start at 0 for fade-in
    var isFadingOut: Bool = false

    init(frame: NSRect, windowID: CGWindowID) {
        super.init(
            contentRect: frame,
            styleMask: .borderless,
            backing: .buffered,
            defer: false
        )

        self.targetWindowID = windowID
        self.isOpaque = false
        self.backgroundColor = NSColor.clear
        self.level = .floating
        self.ignoresMouseEvents = true
        self.collectionBehavior = [.stationary, .fullScreenAuxiliary, .moveToActiveSpace]
        self.hasShadow = false
        self.alphaValue = 1.0

        // Force the window to not have any default content
        self.contentView?.wantsLayer = true
        self.contentView?.layer?.backgroundColor = CGColor.clear
    }

    func cleanup() {
        renderer?.isPaused = true
        mtkView?.isPaused = true
        mtkView?.delegate = nil
        mtkView?.removeFromSuperview()
        mtkView = nil
        renderer = nil
    }
}

struct WindowInfo {
    let id: CGWindowID
    let frame: CGRect
    let ownerPID: pid_t
    let name: String
    let ownerName: String
}

class AppDelegate: NSObject, NSApplicationDelegate {
    var overlayWindows: [CGWindowID: OverlayWindow] = [:]
    var updateTimer: Timer?
    var isReconfiguring = false
    var isInTransition = false  // Brief cooldown during focus/space changes
    var lastTransitionTime: CFAbsoluteTime = 0

    let excludedApps = Set(["SystemUIServer", "Window Server", "Dock", "Spotlight", "Control Center", "Notification Center"])

    // Apps to NEVER overlay (user blacklist) - add app names here
    let blacklistedApps: Set<String> = [
        "System Settings",
        "System Preferences"
    ]

    var fullscreenOverlays: [OverlayWindow] = []

    func applicationDidFinishLaunching(_ notification: Notification) {
        guard ShaderResources.shared != nil else {
            print("Failed to initialize shader resources")
            NSApp.terminate(nil)
            return
        }

        if fullscreenMode {
            print("Fullscreen mode enabled - covering entire screen")
            setupFullscreenOverlay()
            return
        }

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(screenDidChange),
            name: NSApplication.didChangeScreenParametersNotification,
            object: nil
        )

        // React to focus changes immediately
        NSWorkspace.shared.notificationCenter.addObserver(
            self,
            selector: #selector(appDidActivate),
            name: NSWorkspace.didActivateApplicationNotification,
            object: nil
        )

        // React to space changes (Mission Control)
        NSWorkspace.shared.notificationCenter.addObserver(
            self,
            selector: #selector(spaceDidChange),
            name: NSWorkspace.activeSpaceDidChangeNotification,
            object: nil
        )

        updateOverlays()

        updateTimer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            self?.updateOverlays()
        }

        print("Vaporwave overlay running.")
    }

    func setupFullscreenOverlay() {
        guard let resources = ShaderResources.shared else {
            print("No shader resources")
            NSApp.terminate(nil)
            return
        }

        let device = resources.device

        // Create overlay for EVERY screen
        for (index, screen) in NSScreen.screens.enumerated() {
            let frame = screen.frame

            // Use index as windowID for each screen's overlay
            let window = OverlayWindow(frame: frame, windowID: CGWindowID(index))

            guard let contentView = window.contentView else {
                print("No content view for screen \(index)")
                continue
            }

            contentView.wantsLayer = true
            contentView.layer?.backgroundColor = CGColor.clear

            let mtkView = MTKView(frame: contentView.bounds, device: device)
            mtkView.device = device
            mtkView.clearColor = MTLClearColor(red: 0, green: 0, blue: 0, alpha: 0)
            mtkView.colorPixelFormat = .bgra8Unorm
            mtkView.framebufferOnly = true
            mtkView.preferredFramesPerSecond = 30
            mtkView.autoresizingMask = [.width, .height]

            if let metalLayer = mtkView.layer as? CAMetalLayer {
                metalLayer.isOpaque = false
                metalLayer.backgroundColor = CGColor.clear
                metalLayer.pixelFormat = .bgra8Unorm
            }

            guard let renderer = ShaderRenderer(device: device) else {
                print("Failed to create renderer for screen \(index)")
                continue
            }

            mtkView.delegate = renderer
            window.mtkView = mtkView
            window.renderer = renderer
            renderer.overlayWindow = window
            renderer.opacity = 0.0  // Start invisible
            renderer.isFullscreenCapture = true  // Enable screen capture
            renderer.screenIndex = index  // Capture this screen
            window.targetOpacity = 1.0  // Fade in to full
            window.currentOpacity = 0.0  // Start at 0 for fade-in

            contentView.addSubview(mtkView)
            window.setFrame(frame, display: true)
            window.orderFrontRegardless()

            fullscreenOverlays.append(window)
            print("Fullscreen overlay active on screen \(index): \(frame)")
        }

        // React to screen changes
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(fullscreenScreenDidChange),
            name: NSApplication.didChangeScreenParametersNotification,
            object: nil
        )

        print("Fullscreen overlays active on \(NSScreen.screens.count) screens")
    }

    @objc func fullscreenScreenDidChange(_ notification: Notification) {
        print("Screen changed - stopping fullscreen overlays")
        for overlay in fullscreenOverlays {
            overlay.cleanup()
            overlay.orderOut(nil)
        }
        fullscreenOverlays.removeAll()
        NSApp.terminate(nil)
    }

    @objc func screenDidChange(_ notification: Notification) {
        debugLog("Screen configuration changed - fading out all overlays")
        isReconfiguring = true

        // Fade out all overlays instead of destroying immediately
        for (_, overlay) in overlayWindows {
            overlay.targetOpacity = 0.0
            overlay.isFadingOut = true
        }

        // Wait longer for display configuration to settle, then rebuild
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) { [weak self] in
            guard let self = self else { return }
            // Now clean up fully faded overlays
            for (windowID, overlay) in self.overlayWindows {
                overlay.cleanup()
                overlay.orderOut(nil)
                self.overlayWindows.removeValue(forKey: windowID)
            }
            self.isReconfiguring = false
            self.updateOverlays()
        }
    }

    @objc func spaceDidChange(_ notification: Notification) {
        debugLog("Space changed - fading out all overlays")
        startTransition()

        // Immediately fade out all overlays when space changes
        for (_, overlay) in overlayWindows {
            overlay.targetOpacity = 0.0
            overlay.isFadingOut = true
        }

        // Short delay to let system settle, then update
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.4) { [weak self] in
            self?.updateOverlays()
        }
    }

    @objc func appDidActivate(_ notification: Notification) {
        debugLog("App activated - updating overlays")
        startTransition()

        // Reset animation time on all overlays so rays start fresh
        let now = CFAbsoluteTimeGetCurrent()
        for (_, overlay) in overlayWindows {
            overlay.renderer?.animationStartTime = now
        }

        // Fade out ALL overlays immediately - updateOverlays will revive the correct ones
        // This prevents the focused window from ever having an overlay during transition
        for (_, overlay) in overlayWindows {
            overlay.targetOpacity = 0.0
            overlay.isFadingOut = true
        }

        // Short delay, then full update (which will revive non-focused window overlays)
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) { [weak self] in
            self?.updateOverlays()
        }
    }

    func getWindowList() -> [WindowInfo] {
        var windows: [WindowInfo] = []

        let options: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
        guard let windowList = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] else {
            return windows
        }

        guard let primaryScreen = NSScreen.screens.first else { return windows }
        let primaryHeight = primaryScreen.frame.height

        for windowDict in windowList {
            guard let windowID = windowDict[kCGWindowNumber as String] as? CGWindowID,
                  let boundsDict = windowDict[kCGWindowBounds as String] as? [String: CGFloat],
                  let ownerPID = windowDict[kCGWindowOwnerPID as String] as? pid_t else {
                continue
            }

            let x = boundsDict["X"] ?? 0
            let y = boundsDict["Y"] ?? 0
            let width = boundsDict["Width"] ?? 0
            let height = boundsDict["Height"] ?? 0

            if width < 50 || height < 50 { continue }

            let layer = windowDict[kCGWindowLayer as String] as? Int ?? 0
            if layer != 0 { continue }  // Only normal layer (0) windows

            let ownerName = windowDict[kCGWindowOwnerName as String] as? String ?? ""

            if excludedApps.contains(ownerName) { continue }
            if blacklistedApps.contains(ownerName) { continue }  // User blacklist
            if ownerName == "vaporwave-overlay" { continue }
            if ownerPID == ProcessInfo.processInfo.processIdentifier { continue }

            let windowName = windowDict[kCGWindowName as String] as? String ?? ""
            let convertedY = primaryHeight - y - height
            let frame = CGRect(x: x, y: convertedY, width: width, height: height)

            windows.append(WindowInfo(
                id: windowID,
                frame: frame,
                ownerPID: ownerPID,
                name: windowName,
                ownerName: ownerName
            ))
        }

        return windows
    }

    func getFrontmostPID() -> pid_t {
        return NSWorkspace.shared.frontmostApplication?.processIdentifier ?? 0
    }

    func getActiveScreenIndex() -> Int? {
        // Find which screen has the focused/frontmost window
        let screens = NSScreen.screens
        guard let primaryScreen = screens.first else { return nil }
        let primaryHeight = primaryScreen.frame.height

        let options: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
        guard let windowList = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] else {
            return nil
        }

        let frontPID = getFrontmostPID()
        let ourPID = ProcessInfo.processInfo.processIdentifier

        for windowDict in windowList {
            guard let ownerPID = windowDict[kCGWindowOwnerPID as String] as? pid_t,
                  let boundsDict = windowDict[kCGWindowBounds as String] as? [String: CGFloat] else {
                continue
            }

            if ownerPID == ourPID { continue }
            if ownerPID != frontPID { continue }  // Only look at frontmost app's windows

            let layer = windowDict[kCGWindowLayer as String] as? Int ?? 0
            if layer != 0 { continue }

            let x = boundsDict["X"] ?? 0
            let y = boundsDict["Y"] ?? 0
            let w = boundsDict["Width"] ?? 100
            let h = boundsDict["Height"] ?? 100
            let cocoaY = primaryHeight - y - h
            let windowCenter = CGPoint(x: x + w/2, y: cocoaY + h/2)

            for (index, screen) in screens.enumerated() {
                if screen.frame.contains(windowCenter) {
                    return index
                }
            }
        }
        return nil
    }

    func getScreenIndexForWindow(_ windowFrame: CGRect) -> Int? {
        let screens = NSScreen.screens
        let windowCenter = CGPoint(x: windowFrame.midX, y: windowFrame.midY)
        for (index, screen) in screens.enumerated() {
            if screen.frame.contains(windowCenter) {
                return index
            }
        }
        return nil
    }

    func startTransition() {
        isInTransition = true
        lastTransitionTime = CFAbsoluteTimeGetCurrent()

        // Auto-clear transition state after a short delay
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) { [weak self] in
            self?.isInTransition = false
        }
    }

    func updateOverlays() {
        guard Thread.isMainThread else {
            DispatchQueue.main.async { [weak self] in self?.updateOverlays() }
            return
        }

        guard let resources = ShaderResources.shared else { return }
        if isReconfiguring { return }
        guard !NSScreen.screens.isEmpty else { return }

        let frontPID = getFrontmostPID()
        let activeScreenIndex = getActiveScreenIndex()  // Screen with focused window
        let windows = getWindowList()

        debugLog("Active screen: \(activeScreenIndex ?? -1), frontPID: \(frontPID)")

        var targetIDs: Set<CGWindowID> = []
        var windowFrames: [CGWindowID: CGRect] = [:]

        for win in windows {
            // Skip windows from the frontmost app (by PID)
            if win.ownerPID == frontPID { continue }
            // CRITICAL: Skip ALL windows on the active screen (where user is focused)
            if let activeIdx = activeScreenIndex,
               let winScreenIdx = getScreenIndexForWindow(win.frame),
               winScreenIdx == activeIdx {
                continue
            }
            if win.frame.width <= 0 || win.frame.height <= 0 { continue }
            if win.frame.origin.x.isNaN || win.frame.origin.y.isNaN { continue }
            targetIDs.insert(win.id)
            windowFrames[win.id] = win.frame
        }

        let currentIDs = Set(overlayWindows.keys)
        let activeIDs = Set(overlayWindows.filter { !$0.value.isFadingOut }.keys)
        let toRemove = activeIDs.subtracting(targetIDs)
        let toAdd = targetIDs.subtracting(currentIDs)  // Only add if not present at all
        let toUpdate = targetIDs.intersection(activeIDs)

        // Revive fading-out windows that should be visible again
        for windowID in targetIDs.intersection(currentIDs.subtracting(activeIDs)) {
            if let overlay = overlayWindows[windowID] {
                overlay.targetOpacity = 1.0
                overlay.isFadingOut = false
            }
        }

        for windowID in toRemove {
            if let overlay = overlayWindows[windowID] {
                // Start fade out instead of immediate removal
                overlay.targetOpacity = 0.0
                overlay.isFadingOut = true
            }
        }

        // Remove fully faded overlays
        let fadedOut = overlayWindows.filter { $0.value.isFadingOut && $0.value.currentOpacity < 0.01 }
        for (windowID, overlay) in fadedOut {
            overlayWindows.removeValue(forKey: windowID)
            overlay.cleanup()
            overlay.orderOut(nil)
        }

        for windowID in toUpdate {
            if let overlay = overlayWindows[windowID],
               let frame = windowFrames[windowID],
               overlay.frame != frame {
                overlay.setFrame(frame, display: false)
            }
        }

        // Limit overlays for debugging
        // IMPORTANT: Don't create new overlays during transitions to avoid flashing
        var created = 0
        if !isInTransition {
            for windowID in toAdd {
                if created >= 10 { break }
                guard let frame = windowFrames[windowID] else { continue }
                createOverlay(windowID: windowID, frame: frame, device: resources.device)
                created += 1
            }
        }

        debugLog("Overlays: \(overlayWindows.count), removed: \(toRemove.count), added: \(created), updated: \(toUpdate.count), inTransition: \(isInTransition), activeScreen: \(activeScreenIndex ?? -1)")
    }

    func createOverlay(windowID: CGWindowID, frame: CGRect, device: MTLDevice) {
        let window = OverlayWindow(frame: frame, windowID: windowID)

        guard let contentView = window.contentView else { return }
        contentView.wantsLayer = true
        contentView.layer?.backgroundColor = CGColor.clear

        let mtkView = MTKView(frame: contentView.bounds, device: device)
        mtkView.device = device
        mtkView.clearColor = MTLClearColor(red: 0, green: 0, blue: 0, alpha: 0)
        mtkView.colorPixelFormat = .bgra8Unorm
        mtkView.framebufferOnly = true
        mtkView.preferredFramesPerSecond = 5
        mtkView.autoresizingMask = [.width, .height]

        // Configure the CAMetalLayer for transparency
        if let metalLayer = mtkView.layer as? CAMetalLayer {
            metalLayer.isOpaque = false
            metalLayer.backgroundColor = CGColor.clear
            metalLayer.pixelFormat = .bgra8Unorm
        }

        guard let renderer = ShaderRenderer(device: device) else {
            print("Failed to create renderer")
            return
        }

        mtkView.delegate = renderer
        window.mtkView = mtkView
        window.renderer = renderer
        renderer.overlayWindow = window
        renderer.opacity = 0.0  // Start transparent for fade-in
        window.targetOpacity = 1.0

        contentView.addSubview(mtkView)
        window.setFrame(frame, display: true)
        window.orderFrontRegardless()

        overlayWindows[windowID] = window
        debugLog("Created overlay for \(windowID) at \(frame)")
    }

    func applicationWillTerminate(_ notification: Notification) {
        updateTimer?.invalidate()
        for (_, window) in overlayWindows {
            window.cleanup()
            window.orderOut(nil)
        }
        overlayWindows.removeAll()
    }
}

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.setActivationPolicy(.accessory)
app.run()
