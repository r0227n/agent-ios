#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/../MockApp.app"

# Clean previous build
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

# Get iOS Simulator SDK path
SDK_PATH=$(xcrun --sdk iphonesimulator --show-sdk-path)

# Compile for iOS Simulator (arm64 for Apple Silicon, x86_64 for Intel)
# Build for both architectures to support all Macs
echo "Building MockApp for iOS Simulator..."

swiftc \
    -sdk "$SDK_PATH" \
    -target arm64-apple-ios14.0-simulator \
    -o "$OUTPUT_DIR/MockApp-arm64" \
    "$SCRIPT_DIR/main.swift" 2>/dev/null || true

swiftc \
    -sdk "$SDK_PATH" \
    -target x86_64-apple-ios14.0-simulator \
    -o "$OUTPUT_DIR/MockApp-x86_64" \
    "$SCRIPT_DIR/main.swift" 2>/dev/null || true

# Create universal binary if both architectures built
if [[ -f "$OUTPUT_DIR/MockApp-arm64" && -f "$OUTPUT_DIR/MockApp-x86_64" ]]; then
    lipo -create \
        "$OUTPUT_DIR/MockApp-arm64" \
        "$OUTPUT_DIR/MockApp-x86_64" \
        -output "$OUTPUT_DIR/MockApp"
    rm "$OUTPUT_DIR/MockApp-arm64" "$OUTPUT_DIR/MockApp-x86_64"
elif [[ -f "$OUTPUT_DIR/MockApp-arm64" ]]; then
    mv "$OUTPUT_DIR/MockApp-arm64" "$OUTPUT_DIR/MockApp"
elif [[ -f "$OUTPUT_DIR/MockApp-x86_64" ]]; then
    mv "$OUTPUT_DIR/MockApp-x86_64" "$OUTPUT_DIR/MockApp"
else
    echo "Error: Failed to build for any architecture"
    exit 1
fi

# Copy Info.plist
cp "$SCRIPT_DIR/Info.plist" "$OUTPUT_DIR/Info.plist"

# Sign the app (ad-hoc signing for simulator)
codesign --force --sign - "$OUTPUT_DIR/MockApp"

echo "MockApp.app built successfully at: $OUTPUT_DIR"
