# Prose

A minimalist, local-first reader for ePub 2, ePub 3, and PDF books.

Prose is built with Tauri 2 and Rust for the application core, paired with a Vue 3 and TypeScript frontend powered by PrimeVue and Tailwind CSS.

## Features

- Local-first storage: Book files, reading progress, highlights, bookmarks, and reading sessions are stored locally in an embedded SQLite database.
- WebDAV sync: Optional cross-device synchronization with last-write-wins conflict resolution and background sync engine.
- Offline dictionary: Fast word definition lookup integrated directly into the reading interface.
- Reading statistics: Track daily reading activity, current and best reading streaks, and library statistics over time.
- Customizable typography: Choose from curated themes, adjust typefaces, font size, line spacing, margins, text alignment, and click zones.
- Cross-platform desktop: macOS, Linux, and Windows support via Tauri 2.

## Architecture

Prose enforces a strict boundary between business logic and UI presentation:

- Core (`src-tauri`): Domain logic, data persistence (rusqlite), ePub and PDF parsing, WebDAV client, and typed IPC endpoints.
- WebView (`src`): Vue 3 SFCs, state management via reactive stores, composables, and custom PDF and ePub renderer adapters.

## Getting Started

### Prerequisites

- [Bun](https://bun.sh/) or [Node.js](https://nodejs.org/) (v18+)
- [Rust toolchain](https://www.rust-lang.org/tools/install) (1.75+)
- Platform dependencies for [Tauri 2](https://v2.tauri.app/start/prerequisites/)

### Installation

Install frontend and project dependencies:

```sh
bun install
```

### Development

Run the application in desktop development mode:

```sh
bun run tauri dev
```

### Testing

Run frontend unit and integration tests:

```sh
bun run test
```

Run Rust core unit tests:

```sh
cd src-tauri && cargo test
```

### Building

Build and type-check the frontend bundle:

```sh
bun run build
```

Package the desktop binary for distribution:

```sh
bun run tauri build
```

### App Icons

Generate platform application icons:

```sh
bun run generate-icons
```

## License

This project is licensed under the [MIT License](LICENSE).
