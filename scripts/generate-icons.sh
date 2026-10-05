#!/bin/bash
set -e

# Make sure we are in the project root directory
cd "$(dirname "$0")/.."

# Check if icon source exists
ICON_SRC="${1:-assets/app-icon-2.png}"
if [ ! -f "$ICON_SRC" ]; then
  echo "Error: $ICON_SRC not found."
  echo "Please place a full-bleed 1024x1024 square PNG (no rounded corners, no transparency) at $ICON_SRC"
  exit 1
fi

echo "Generating desktop icons from $ICON_SRC..."
bunx tauri icon "$ICON_SRC"

echo "Generating mobile icons from $ICON_SRC..."
bunx tauri icon "$ICON_SRC" --output temp-icons

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
  python3 scripts/macos-icns.py "$ICON_SRC"
fi

echo "Icons generated successfully!"
