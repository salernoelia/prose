# Tauri + Vue + TypeScript

This template should help get you started developing with Vue 3 and TypeScript in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## App icons

All platform icons are generated from two sources with a single command:

```
bun run generate-icons
```

- `assets/app-icon.png` is the full-bleed 1024x1024 source for desktop.
- `assets/app-icon-mobile.png` is the full-bleed 1024x1024 source for iOS and Android (those platforms apply their own mask, so the art must be square with no rounding or transparency).

The command runs `tauri icon` for both, then rebuilds `src-tauri/icons/icon.icns` on Apple's macOS icon grid (rounded squircle, padding, and shadow) because macOS does not mask app icons. The macOS step runs only on macOS and needs Python with Pillow (auto-installed if missing).
