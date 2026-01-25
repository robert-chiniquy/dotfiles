#!/bin/bash
# Build Vaporwave Overlay Metal app (runtime shader compilation)

set -e
cd "$(dirname "$0")"

echo "Compiling Swift app (shader compiles at runtime)..."
swiftc -O \
    -framework Cocoa \
    -framework Metal \
    -framework MetalKit \
    -framework QuartzCore \
    VaporwaveOverlay.swift \
    -o vaporwave-overlay

echo "Build complete: ./vaporwave-overlay"
echo ""
echo "Run with: ./vaporwave-overlay"
echo "Stop with: Ctrl+C"
