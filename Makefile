.PHONY: run overlay clean log deploy stop kill enable disable status

run: overlay

overlay:
	$(MAKE) -C metal-overlay run

# Build, install, and restart the overlay
deploy:
	$(MAKE) -C metal-overlay deploy

# Stop the overlay
stop:
	$(MAKE) -C metal-overlay stop

# Kill immediately and disable auto-start
kill:
	pkill -9 -f vaporwave-overlay 2>/dev/null || true
	launchctl unload ~/Library/LaunchAgents/com.rch.vaporwave-overlay.plist 2>/dev/null || true
	@echo "Killed and disabled"

# Enable auto-start (window mode, background windows only)
enable:
	launchctl load ~/Library/LaunchAgents/com.rch.vaporwave-overlay.plist 2>/dev/null || true
	@echo "Enabled - will auto-start on login"

# Disable auto-start
disable:
	launchctl unload ~/Library/LaunchAgents/com.rch.vaporwave-overlay.plist 2>/dev/null || true
	pkill -f vaporwave-overlay 2>/dev/null || true
	@echo "Disabled"

# Check status
status:
	@launchctl list | grep vaporwave && echo "LaunchAgent: loaded" || echo "LaunchAgent: not loaded"
	@pgrep -f vaporwave-overlay >/dev/null && echo "Process: running" || echo "Process: not running"

clean:
	$(MAKE) -C metal-overlay clean

log:
	@cat /tmp/vaporwave-debug.log 2>/dev/null || echo "No log file yet"
