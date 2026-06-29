#!/bin/bash
set -e

# Make sure we are in the project root directory
cd "$(dirname "$0")/.."

# Check if mobile icon source exists
MOBILE_SRC="assets/app-icon-mobile.png"
if [ ! -f "$MOBILE_SRC" ]; then
  echo "Error: $MOBILE_SRC not found."
  echo "Please place a full-bleed 1024x1024 square PNG (no rounded corners, no transparency) at assets/app-icon-mobile.png"
  exit 1
fi

echo "Generating desktop icons from assets/app-icon.png..."
bunx tauri icon assets/app-icon.png

echo "Generating mobile icons from $MOBILE_SRC..."
bunx tauri icon "$MOBILE_SRC" --output temp-icons

echo "Copying mobile icons to Tauri assets..."
cp -r temp-icons/ios/ src-tauri/icons/ios/
cp -r temp-icons/android/ src-tauri/icons/android/
cp -r temp-icons/ios/*.png src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset/

echo "Cleaning up temporary files..."
rm -rf temp-icons

# `tauri icon` writes a full-bleed square .icns; macOS does not mask app icons,
# so rebuild it on Apple's icon grid (rounded squircle + padding). macOS only.
if [ "$(uname)" = "Darwin" ]; then
  echo "Applying macOS icon grid to icon.icns..."
  python3 -c "import PIL" 2>/dev/null || python3 -m pip install --quiet --user Pillow
  python3 scripts/macos-icns.py
fi

echo "Icons generated successfully!"
