# Prose

Cross-platform, local-first reader for ePub 2, ePub 3, and PDF. Tauri 2 (Rust core) plus
Vue 3 + TypeScript in the system WebView, with PrimeVue as the UI component library and
Tailwind CSS for styling.

## Read first

- `spec/software-requirements.typ` defines what Prose does (requirements, scope, NFRs).
- `spec/architecture.md` defines how it is built (patterns, module layout, the TS <-> Rust
  IPC contract, reader adapters, settings, sync). Follow it before adding code.
- `spec/implementation-plan.md` for the implementation progress. Tick off implemented and verified parts.

## The one rule

Business logic lives in the Rust domain core (`src-tauri/src/domain`), behind ports. The
WebView renders and captures input. Everything crosses the boundary through the typed IPC
layer (`src-tauri/src/ipc` and `src/ipc`). Do not call `invoke` from components, do not push
book bytes through `invoke`, do not reimplement domain logic in TypeScript.

## Commands

```
bun install
bun run tauri dev        # run the app
bun run build            # type-check + build frontend
bun run tauri build      # package the app
```

## House style

No em dashes, no emojis, specific language, no duplication. Match the surrounding code.
