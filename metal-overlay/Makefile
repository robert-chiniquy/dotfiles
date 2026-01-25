# Vaporwave Metal Overlay - Makefile

.PHONY: all clean run log tail app install stop restart deploy

APP_BUNDLE = VaporwaveOverlay.app
APP_CONTENTS = $(APP_BUNDLE)/Contents
APP_MACOS = $(APP_CONTENTS)/MacOS
APP_RESOURCES = $(APP_CONTENTS)/Resources
INSTALL_DIR = $(HOME)/Applications

all: vaporwave-overlay

vaporwave-overlay: VaporwaveOverlay.swift VaporwaveShader.metal
	swiftc -O \
		-framework AppKit \
		-framework Metal \
		-framework MetalKit \
		-framework ScreenCaptureKit \
		VaporwaveOverlay.swift \
		-o vaporwave-overlay

app: vaporwave-overlay
	mkdir -p $(APP_MACOS) $(APP_RESOURCES)
	cp vaporwave-overlay $(APP_MACOS)/
	cp VaporwaveShader.metal $(APP_RESOURCES)/

install: app
	rm -rf $(INSTALL_DIR)/$(APP_BUNDLE)
	cp -R $(APP_BUNDLE) $(INSTALL_DIR)/
	# Ad-hoc code sign to preserve permissions across rebuilds
	codesign --force --deep --sign - $(INSTALL_DIR)/$(APP_BUNDLE)

stop:
	pkill -f VaporwaveOverlay 2>/dev/null || true
	@echo "Stopped"

restart: stop
	sleep 0.5
	open -a $(INSTALL_DIR)/$(APP_BUNDLE)
	@echo "Restarted"

# Build, install, and restart in one command
deploy: install restart

clean:
	rm -f vaporwave-overlay *.air *.metallib /tmp/vaporwave-debug.log
	rm -rf $(APP_BUNDLE)/Contents/MacOS/vaporwave-overlay

run: vaporwave-overlay
	./vaporwave-overlay

log:
	cat /tmp/vaporwave-overlay.log

tail:
	tail -f /tmp/vaporwave-overlay.log
