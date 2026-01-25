# Vaporwave Metal Overlay

A macOS Metal shader overlay that adds vaporwave visual effects to windows.

Based on the `underwater-glitchy.glsl` shader from Ghostty.

## Features

- Curved light rays with vaporwave color palette rotation
- Purple content detection with drip/echo effects
- Moire scanlines
- Window mode (background windows only) or fullscreen mode
- Auto-start via LaunchAgent

## Permissions

### Screen Recording Permission

The overlay uses ScreenCaptureKit to detect purple content in windows. This requires Screen Recording permission.

**If you see repeated permission popups:**

1. Open **System Settings > Privacy & Security > Screen Recording**
2. Find "VaporwaveOverlay" in the list
3. **Remove it** (click minus or toggle off and remove)
4. Run `make deploy` from `~/repo/dotfiles`
5. Grant permission when prompted

The app is now code-signed (ad-hoc), so permission should persist across rebuilds.

**Why this happens:**
- Unsigned apps get a new identity each rebuild
- macOS treats each rebuild as a "new" app requesting permission
- Code signing preserves the app identity

## Usage

From `~/repo/dotfiles`:

```bash
make deploy    # Build, install, restart
make kill      # Stop and disable auto-start
make enable    # Enable auto-start (window mode)
make disable   # Disable auto-start
make status    # Check if running
```

Shell alias for fullscreen toggle:
```bash
vw             # Toggle fullscreen mode on/off
```

## Modes

- **Window mode** (default, via LaunchAgent): Overlays unfocused windows only
- **Fullscreen mode** (`--fullscreen` flag): Covers entire screen, fades in slowly

## Files

- `VaporwaveOverlay.swift` - Main app, window management, screen capture
- `VaporwaveShader.metal` - GPU shader for visual effects
- `Makefile` - Build and install targets
- `../LaunchAgents/com.rch.vaporwave-overlay.plist` - Auto-start config (symlinked)

## Customization

Edit `VaporwaveShader.metal` to modify:
- `VAPORWAVE_PALETTE` - color palette
- Ray positions and directions
- Purple detection sensitivity
- Effect intensities
- Animation speeds
