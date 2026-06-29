#!/usr/bin/env python3
"""Rebuild src-tauri/icons/icon.icns to follow Apple's macOS icon grid.

`tauri icon` resizes the source straight into the .icns, producing a hard-edged,
full-bleed square. macOS never masks app icons, so that reads as an oversized
square in the Dock and Finder. macOS expects the shape baked in: a rounded
squircle at ~82% scale, centered with transparent padding and a soft shadow.

iOS and Android are untouched: those platforms mask full-bleed art themselves.
"""

import os
import subprocess
import sys

from PIL import Image, ImageDraw, ImageFilter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "assets", "app-icon.png")
OUT = os.path.join(ROOT, "src-tauri", "icons", "icon.icns")

CANVAS = 1024            # Apple macOS icon grid, expressed at 1024px
BODY = 824               # icon body fills ~82% of the canvas
RADIUS = 185            # corner radius (~824 * 0.2247)
MARGIN = (CANVAS - BODY) // 2

src = Image.open(SRC).convert("RGBA").resize((BODY, BODY), Image.LANCZOS)

mask = Image.new("L", (BODY, BODY), 0)
ImageDraw.Draw(mask).rounded_rectangle([0, 0, BODY - 1, BODY - 1], radius=RADIUS, fill=255)

body = Image.new("RGBA", (BODY, BODY), (0, 0, 0, 0))
body.paste(src, (0, 0), mask)

shadow = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
ImageDraw.Draw(shadow).rounded_rectangle(
    [MARGIN, MARGIN + 8, MARGIN + BODY - 1, MARGIN + BODY - 1 + 8],
    radius=RADIUS, fill=(0, 0, 0, 60))
shadow = shadow.filter(ImageFilter.GaussianBlur(12))

master = Image.alpha_composite(Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0)), shadow)
master.paste(body, (MARGIN, MARGIN), body)

iconset = os.path.join(ROOT, "scripts", ".prose.iconset")
os.makedirs(iconset, exist_ok=True)
sizes = {
    "icon_16x16.png": 16, "icon_16x16@2x.png": 32,
    "icon_32x32.png": 32, "icon_32x32@2x.png": 64,
    "icon_128x128.png": 128, "icon_128x128@2x.png": 256,
    "icon_256x256.png": 256, "icon_256x256@2x.png": 512,
    "icon_512x512.png": 512, "icon_512x512@2x.png": 1024,
}
for name, sz in sizes.items():
    master.resize((sz, sz), Image.LANCZOS).save(os.path.join(iconset, name))

try:
    subprocess.run(["iconutil", "-c", "icns", iconset, "-o", OUT], check=True)
finally:
    for name in sizes:
        os.remove(os.path.join(iconset, name))
    os.rmdir(iconset)

print(f"Rebuilt {os.path.relpath(OUT, ROOT)} with the macOS icon grid.", file=sys.stderr)
