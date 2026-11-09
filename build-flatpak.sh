#!/usr/bin/env bash
# Build and package Hello Neighbor as Flatpak
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

APP_ID="org.helloneighbor.HelloNeighbor"
BUILD_DIR="${SCRIPT_DIR}/.flatpak-build"
REPO_DIR="${SCRIPT_DIR}/.flatpak-repo"
BUNDLE_FILE="${SCRIPT_DIR}/${APP_ID}.flatpak"

echo "=== Building Hello Neighbor Flatpak ==="

# Check flatpak-builder
if ! command -v flatpak-builder >/dev/null 2>&1; then
    echo "Error: flatpak-builder is not installed."
    echo "Install it with: sudo pacman -S flatpak-builder"
    exit 1
fi

# Ensure icon exists
if [ ! -f "reference/icon.svg" ]; then
    echo "Error: reference/icon.svg not found."
    exit 1
fi

# Clean previous build artifacts if requested
if [ "${1:-}" = "--clean" ]; then
    echo "Cleaning previous build..."
    rm -rf "$BUILD_DIR" "$REPO_DIR" "$BUNDLE_FILE"
fi

echo "Building flatpak package into $BUILD_DIR..."
flatpak-builder --user --force-clean \
    --repo="$REPO_DIR" \
    "$BUILD_DIR" \
    "${APP_ID}.yml"

echo "Exporting single-file bundle: $BUNDLE_FILE..."
flatpak build-bundle "$REPO_DIR" "$BUNDLE_FILE" "$APP_ID"

echo "=== Build Complete! ==="
echo "Bundle created at: $BUNDLE_FILE"
echo "To install locally for current user:"
echo "  flatpak install --user $BUNDLE_FILE"
echo "Or run directly from build directory:"
echo "  flatpak-builder --run $BUILD_DIR ${APP_ID}.yml hello-neighbor"
