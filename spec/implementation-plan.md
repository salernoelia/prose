# Prose Implementation Plan

A commit-by-commit checklist for building Prose from the current Tauri scaffold to a
feature-complete reader. Read `spec/software-requirements.typ` and `spec/architecture.md`
first; this file only sequences the work, it does not redefine it.

## How to use this

- Each phase is a coherent slice. Each `### Commit` is one reviewable, mergeable unit that
  leaves the build green (compiles, `bun run build` passes, tests pass).
- Tick boxes as you finish them. A commit is done only when every box under it is ticked.
- Requirement tags in parentheses (FR-..., NFR-...) trace work back to the spec.
- Suggested commit messages use Conventional Commits. Adjust as you like.

## Sequencing principle

Foundation and the typed boundary come first, then a thin end-to-end slice proves the whole
stack, then features land one vertical at a time. Order chosen so each commit depends only on
earlier ones:

1. Tooling and test harness (both languages).
2. Domain model and ports (the core "API", pure, no I/O).
3. IPC contract scaffolding and one proven round-trip.
4. Persistence behind the repository port.
5. Settings vertical (smallest full slice: Rust authority to reactive UI store).
6. Library import and catalog.
7. Custom protocol for streaming book bytes.
8. ePub reading, then PDF reading.
9. Reading position and progress.
10. Reading customization and theming.
11. Annotations and dictionary.
12. WebDAV sync.
13. Mobile, performance hardening, packaging.

---

## Phase 0: Foundation and tooling

Goal: a clean skeleton, both test harnesses running, CI green, before any feature code.

### Commit 0.1: Repo hygiene and project metadata

- [x] Replace placeholder `authors`, `description` in `src-tauri/Cargo.toml` and `tauri.conf.json`.
- [x] Set a stable window title and sensible default window size in `tauri.conf.json`.
- [x] Add `rustfmt.toml` and `clippy` config; run `cargo fmt` and `cargo clippy -- -D warnings` clean.
- [x] Add Prettier + ESLint for the frontend (Vue 3 + TS preset); format the existing files.
- [x] Add an `.editorconfig`.
- [x] `chore: project metadata, formatters, linters`

### Commit 0.2: Rust test harness and module skeleton

- [x] Create empty module tree per architecture section 3: `domain/`, `adapters/`, `ipc/`,
      `state.rs`, `protocol.rs`, each with a `mod.rs` and a `//!` doc comment stating its role.
- [x] Wire the modules into `lib.rs` (declared, currently empty).
- [x] Add `thiserror` to `Cargo.toml`; create `domain/error.rs` with an empty `DomainError` enum.
- [x] Add a trivial domain unit test so `cargo test` runs.
- [x] `chore(core): module skeleton and test harness`

### Commit 0.3: Frontend test harness and folder skeleton

- [x] Add Vitest + `@vue/test-utils` + `@testing-library/vue` + `jsdom`; add `bun run test`.
- [x] Create the frontend folders per architecture section 5: `ipc/`, `readers/`, `stores/`,
      `composables/`, `views/`, `components/`, each with an index or placeholder.
- [x] Add one trivial component test so `bun run test` runs green.
- [x] `chore(ui): vitest harness and folder skeleton`

### Commit 0.4: Continuous integration

- [x] CI job: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` (in `src-tauri`).
- [x] CI job: `bun install`, `bun run build` (type-check), `bun run test`, lint.
- [x] Cache cargo and bun; run on push and PR.
- [x] `ci: rust and frontend pipelines`

### Commit 0.5: Strip the scaffold demo

- [x] Remove the `greet` command and the demo UI from `App.vue` and `lib.rs`.
- [x] Reduce `App.vue` to a bare shell (no `invoke`), confirm app still launches.
- [x] `chore: remove scaffold demo`

---

## Phase 1: Domain core (the API of the core)

Goal: model and ports exist and are unit-tested with fakes. No `tauri`, `rusqlite`, `reqwest`,
`std::fs`, or `tokio::fs` anywhere in `domain/` (architecture section 3). Pure logic first.

### Commit 1.1: Core model types

- [x] `domain/model.rs`: `BookId` (content hash, `from_content(&[u8])`), `Format` enum
      (`Epub`, `Pdf`), `BookMetadata` (title, author, cover ref), `Book`.
- [x] `Locator` (format-neutral, opaque payload plus a `progression: f32` fraction; architecture section 6).
- [x] `Progress`, `Bookmark`, `Highlight`, `ReadingStyle`, `Settings` (with `schema_version`,
      every field defaulted; architecture section 7).
- [x] Derive `serde`, `Clone`, `PartialEq`; unit tests for `BookId::from_content` determinism
      and `Settings::default`.
- [x] `feat(core): domain model types`

### Commit 1.2: Ports

- [x] `domain/ports.rs`: `BookRepository`, `ReaderAdapter`, `RemoteStore`, `CredentialStore`,
      `Clock` traits (all `Send + Sync`), return `Result<_, DomainError>`.
- [x] Flesh out `DomainError` variants (not found, invalid format, conflict, storage, remote, etc.).
- [x] In-memory fakes for each port under `#[cfg(test)]` (or a `domain/testing` module) for service tests.
- [x] `feat(core): ports and test fakes`

### Commit 1.3: LibraryService

- [x] `domain/library.rs`: `LibraryService` over `BookRepository` + a `ReaderRegistry`.
- [x] `import(bytes, format)`: probe metadata, build `Book` with content-hash id, dedupe, insert (FR-LIB-01..03).
- [x] `list(query)` with search/filter/sort over title, author, last-read, progress (FR-LIB-04..06).
- [x] `remove(id)` (FR-LIB-07).
- [x] Unit tests with fake repo and fake reader, including dedup of the same content hash.
- [x] `feat(core): library service`

### Commit 1.4: ReadingService

- [x] `domain/reading.rs`: save/get reading position per book (FR-READ-06).
- [x] Progress percentage from locator progression fraction (FR-READ-07).
- [x] "Furthest position" comparison helper (used later by sync, architecture section 8).
- [x] Unit tests for resume and progress math.
- [x] `feat(core): reading service`

### Commit 1.5: AnnotationService

- [x] `domain/annotation.rs`: add/list/delete bookmarks (FR-NOTE-01); add/list/delete highlights (FR-NOTE-02).
- [x] Unit tests.
- [x] `feat(core): annotation service`

### Commit 1.6: SettingsService

- [x] `domain/settings.rs`: get, patch (partial update of changed fields only), validate, version tolerance.
- [x] Unknown/missing fields fall back to default; tests for forward/backward tolerance (architecture section 7).
- [x] `feat(core): settings service`

### Commit 1.7: SyncService conflict logic (pure)

- [x] `domain/sync.rs`: conflict resolution only, against the `RemoteStore` port and a fake.
- [x] Reading position: keep furthest progression; everything else: last-write-wins by timestamp (FR-SYNC-04).
- [x] Outbox model and idempotent, id-keyed operations defined as pure types (architecture section 8).
- [x] Thorough unit tests for each conflict branch; this is load-bearing.
- [x] `feat(core): sync conflict resolution`

> Checkpoint: `cargo test` covers the core with fakes only. Target NFR-M-04 (>=80% on `domain/`).
> Measured 2026-06-28 with `cargo llvm-cov --lib`: 96.82% line coverage on `domain/`
> (45 tests), well above the 80% target. Per file: annotation 100%, reading 100%,
> testing 99.6%, settings 98.4%, model 96.9%, sync 94.6%, library 94.2%.
>
> verified by user

---

## Phase 2: IPC contract and one proven round-trip

Goal: the typed boundary exists and one command travels UI to Rust and back, establishing the
pattern every later feature copies (architecture section 4).

### Commit 2.1: Error boundary and DTO conventions

- [x] `ipc/error.rs`: `AppError { code, message }`, serializable; `From<DomainError>` mapping.
- [x] `ipc/dto.rs`: first DTOs (`SettingsDto`) as flat structs, not domain types.
- [x] `ipc/event.rs`: event-name constants and payload structs.
- [x] `src/ipc/types.ts`: hand-mirrored TS types (strategy 1 from architecture section 4.5).
- [x] `feat(ipc): error boundary and dto conventions`

### Commit 2.2: AppState and dependency wiring

- [x] `state.rs`: `AppState` holding the wired services behind `Arc<dyn Port>`.
- [x] `lib.rs` `run()`: build adapters (temporary in-memory ones are fine here), inject, manage State.
- [x] The only `expect()` allowed lives in this startup wiring (architecture section 4.4).
- [x] `feat(core): app state and service wiring`

### Commit 2.3: Settings command round-trip (reference slice)

- [x] `ipc/settings.rs`: `settings_get`, `settings_patch` thin commands calling `SettingsService`,
      `settings_patch` emits `settings:changed` (architecture section 7).
- [x] Register handlers in `ipc/mod.rs` and `lib.rs`.
- [x] `src/ipc/settings.ts`: the only place `invoke` appears for settings.
- [x] `src/ipc/events.ts`: typed `onSettingsChanged` listener.
- [x] Vitest test with `invoke` mocked: payload shape and error mapping (architecture section 10).
- [x] `feat(ipc): settings command round-trip`

> Checkpoint: the full stack works end to end for one command. Every later vertical mirrors this.
>
> verified by user

---

## Phase 3: Persistence

Goal: the real local store behind the repository port (architecture section 8).

### Commit 3.1: SQLite store and migrations

- [x] Add `rusqlite` (bundled) to `Cargo.toml`.
- [x] `adapters/storage/`: schema and a migration runner; tables for books, progress, bookmarks,
      highlights, settings, sync state.
- [x] Resolve the app data directory via Tauri path APIs; open/create the db there.
- [x] Integration test against a temp-file db (architecture section 10).
- [x] `feat(storage): sqlite store and migrations`

### Commit 3.2: Repository implementation

- [x] Implement `BookRepository` (and settings persistence) over SQLite with atomic transactions (NFR-R-02).
- [x] Swap the in-memory adapter in `run()` for the SQLite one; settings now persist across launches.
- [x] Integration tests for insert/list/query/remove and atomicity.
- [x] `feat(storage): repository over sqlite`

### Commit 3.3: Settings UI store and panel

- [x] `stores/settings.ts`: seed from `settings_get` on startup, update on `settings:changed`.
- [x] `composables/useSettings.ts`; a minimal `views/Settings.vue` exercising one real setting.
- [x] `feat(ui): settings store and panel`

> Checkpoint: settings persist to SQLite and round-trip through the typed boundary, reactively.
>
> verified by user

---

## Phase 4: Library

Goal: import books, see them, manage them (FR-LIB).

### Commit 4.1: ReaderAdapter registry and ePub metadata

- [x] Add the `epub` crate.
- [x] `adapters/readers/epub.rs`: implement `ReaderAdapter::probe` (title, author, cover) and `supports` (FR-LIB-02).
- [x] `ReaderRegistry` selecting an adapter by `Format`.
- [x] Fixture-based tests with a small sample ePub in `tests/fixtures` (architecture section 10).
- [x] `feat(readers): epub metadata adapter`

### Commit 4.2: PDF metadata adapter

- [x] Add `lopdf`/`pdf`; `adapters/readers/pdf.rs`: probe title/author/page count, render or extract a cover thumbnail.
- [x] Register in the registry; fixture tests with a sample PDF.
- [x] `feat(readers): pdf metadata adapter`

### Commit 4.3: Book file storage on import

- [x] On import, copy the source file into app storage and store cover images on disk, referenced by path (architecture section 8).
- [x] Content hashing of large files runs on `spawn_blocking` (architecture section 9).
- [x] `feat(storage): persist book files and covers`

### Commit 4.4: Library import and list commands

- [x] `ipc/library.rs`: `library_import_book(path)`, `library_list(query)`, `library_remove(id)`;
      emit `library:changed` and `import:progress` (architecture section 4.2).
- [x] `src/ipc/library.ts` typed wrappers; extend `events.ts`.
- [x] Use Tauri dialog plugin for the native file picker; add the capability/permission.
- [x] Vitest payload-shape tests.
- [x] `feat(ipc): library import and list commands`

### Commit 4.5: Library view (grid and list)

- [x] `stores/library.ts`, `composables/useLibrary.ts`.
- [x] `views/Library.vue` using PrimeVue `DataView` grid and list layouts, switchable (FR-LIB-04, architecture section 5.1).
- [x] Search and filter by title/author (FR-LIB-05); sort by title/author/last-read/progress (FR-LIB-06).
- [x] Remove-with-confirmation dialog (FR-LIB-07).
- [x] Component tests for switching, search, sort.
- [x] `feat(ui): library grid and list view`

> Checkpoint: import a real ePub and PDF, see covers, search, sort, remove. FR-LIB done.
> Manual validation (2026-06-28): imported `book_sample.epub` via the Library view native picker and verified it appears in catalog, is searchable/sortable, and can be removed.

> verified by user

---

## Phase 5: Custom protocol (book bytes, never invoke)

Goal: stream book resources to the renderer with range support (architecture section 4.3).

### Commit 5.1: prose:// protocol handler

- [x] `protocol.rs`: register `prose://book/{book_id}/{resource_path}` with
      `register_asynchronous_uri_scheme_protocol`.
- [x] Read from the stored file; honor `Range` headers for PDF partial loads (architecture section 4.3, 9).
- [x] Resource listing comes from the `ReaderAdapter`; 404/scoped access so only library books resolve.
- [x] Register the protocol in `run()`; add a test or manual check that a known resource streams.
- [ x] `feat(protocol): prose:// resource streaming`
> verified by user

---

## Phase 6: Reading (ePub then PDF)

Goal: open and render books, navigate, with a page turn off the IPC hot path (architecture section 9).

### Commit 6.1: BookRenderer interface and reader shell

- [x] `src/readers/Renderer.ts`: the `BookRenderer` interface (architecture section 6). Implemented as `src/readers/types.ts`.
- [x] A renderer registry selecting by the `Format` Rust reports (`src/readers/registry.ts`, lazy-loaded).
- [x] `views/Reader.vue` shell + `composables/useReader.ts`; opens a book by id, resolves its `prose://` base URL (`ipc/protocol.ts`). Shell is the existing `ReaderView.vue`, componentized.
- [ x] `feat(reader): renderer interface and reader shell`

### Commit 6.2: ePub rendering with foliate-js

- [x] Vendor/add foliate-js; `src/readers/EpubRenderer.ts` implementing `BookRenderer` over a `prose://` source.
- [x] `load`, `next`, `prev`, `onLocationChange` (FR-READ-01, FR-READ-04).
- [x] Render the first ePub page within budget (NFR-P-02); page turn is renderer-local (NFR-P-03).
- [ x] `feat(reader): epub rendering`
> verified by user

### Commit 6.3: PDF rendering with pdf.js

- [x] Add pdf.js; `src/readers/PdfRenderer.ts` implementing `BookRenderer`, fetching pages via `prose://` with range (FR-READ-02).
- [x] Zoom and fit, default fit-to-width (FR-READ-05); `applyStyle` is a no-op for PDF (architecture section 6).
- [ x] `feat(reader): pdf rendering`

> verified by user


### Commit 6.4: Table of contents navigation

- [x] Extract the TOC (from the renderer) and render it in a PrimeVue `Drawer` (FR-READ-03, architecture section 5.1).
- [x] Navigate to a selected entry via `goToHref` (the renderer-neutral destination on `TocItem`).
- [x ] `feat(reader): table of contents navigation`

> Checkpoint: open both formats, turn pages, navigate by TOC. Verify NFR-P-02 and NFR-P-03 by hand.
>
> verified by user

---

## Phase 7: Reading position and progress

Goal: persist and resume position; show progress (FR-READ-06, FR-READ-07).

### Commit 7.1: Position commands and capture

- [x] `ipc/reading.rs`: `reading_save_position`, `reading_get_position`; typed `src/ipc/reading.ts`.
- [x] Renderer reports the current `Locator` on change; UI saves it asynchronously after the turn paints (architecture section 9).
- [x] Reopen resumes at the stored position.
- [x] Tests: payload shape; domain resume already covered in 1.4.
- [x] `feat(reading): save and resume position`

### Commit 7.2: Progress display

- [x] Progress percentage in the reader chrome and on library cards, fed by `ReadingService` (FR-READ-07).
- [x] Last-read time updates feed the library sort (FR-LIB-06).
- [x] `feat(reading): progress display`

---

## Phase 8: Reading customization and theming

Goal: ePub typography controls and the three themes (FR-CUST).

### Commit 8.1: Reading style controls

- [x] Extend `Settings` and the settings panel: font family from a bundled set, font size, line spacing, margins (FR-CUST-01..03).
- [x] Bundle the reading fonts; expose them to the ePub renderer.
- [x] `EpubRenderer.applyStyle` applies them live; PDF ignores them (architecture section 7).
- [x] Settings persist and reapply across sessions and books (FR-CUST-05).
- [x] `feat(custom): epub reading style controls`

### Commit 8.2: Themes (light, dark, sepia)

- [x] One Rust-owned `theme` setting drives everything (architecture section 5.3).
- [x] PrimeVue `darkModeSelector` + Tailwind dark variant point at one root attribute; `App.vue` sets it on `settings:changed`.
- [x] Sepia as a third token set selected by the same attribute; the ePub renderer style reacts too (FR-CUST-04).
- [x] `feat(custom): light dark sepia themes`

---

## Phase 9: Annotations and reference

Goal: bookmarks, highlights, dictionary (FR-NOTE).

### Commit 9.1: Bookmarks

- [ ] `ipc/annotation.rs`: add/list/delete bookmark commands; typed `src/ipc/annotation.ts`.
- [ ] UI: bookmark the current location, list in a drawer, delete (FR-NOTE-01).
- [ ] `feat(annotation): bookmarks`

### Commit 9.2: Highlights

- [ ] Capture a text selection range from the ePub renderer; persist as a `Highlight` (FR-NOTE-02).
- [ ] Render existing highlights on load; view and delete; only for content with selectable text.
- [ ] `feat(annotation): highlights`

### Commit 9.3: Offline dictionary

- [ ] Bundle a dictionary data set; a `DictionaryService` (and port if it needs bundled-file access) in the core.
- [ ] Select a word, look up the definition, show it in a popover (FR-NOTE-03, external interface in the spec).
- [ ] `feat(reference): offline dictionary lookup`

---

## Phase 10: WebDAV synchronization

Goal: optional sourcing and background sync, resumable, non-blocking (FR-SYNC, NFR-R-03).

### Commit 10.1: Credential store

- [ ] Add `keyring`; `adapters/credentials.rs` implementing `CredentialStore` (NFR-S-02).
- [ ] `feat(sync): os keychain credential store`

### Commit 10.2: WebDAV remote adapter

- [ ] Add `reqwest_dav`; `adapters/webdav.rs` implementing `RemoteStore`, HTTPS/TLS 1.2+ enforced (NFR-S-01).
- [ ] Tests against a mock WebDAV server: list, download, upload, ETag cursor (architecture section 10).
- [ ] `feat(sync): webdav remote adapter`

### Commit 10.3: Configure server and browse remote

- [ ] `ipc/sync.rs`: `sync_configure(url, user, password)` (FR-SYNC-01), `sync_list_remote` to list .epub/.pdf and download one into the library (FR-SYNC-02).
- [ ] Settings UI for the single server; typed `src/ipc/sync.ts`.
- [ ] `feat(sync): configure server and browse remote`

### Commit 10.4: Background sync engine

- [ ] Wire `SyncService` (logic from 1.7) to the real remote; run on its own task, communicate via events only (architecture section 8, 9).
- [ ] Sync position, bookmarks, highlights, settings, and book files (FR-SYNC-03); local changes apply immediately, upload on next connection (FR-SYNC-06).
- [ ] Outbox + per-file ETag cursor make an interrupted sync resumable without loss or duplicates (NFR-R-03).
- [ ] `sync:progress` and `sync:finished` events feed a status indicator; reading never blocks (FR-SYNC-05).
- [ ] App stays fully usable with no server configured (FR-SYNC-05).
- [ ] `feat(sync): background sync engine`

> Checkpoint: configure a real WebDAV server, sync two devices, kill mid-sync, confirm resume.

---

## Phase 11: Mobile, performance, packaging

Goal: meet portability and performance NFRs and ship.

### Commit 11.1: Responsive and touch

- [ ] Layouts adapt to desktop and mobile sizes; pointer and touch input both work (NFR-X-02).
- [ ] Touch gestures for page turns in the reader.
- [ ] `feat(ui): responsive and touch input`

### Commit 11.2: iOS and Android targets

- [ ] `tauri ios init` / `tauri android init`; resolve mobile-only plugin permissions and the dialog/file picker on mobile.
- [ ] Confirm the prose:// protocol and keychain/keystore work on both (NFR-X-01).
- [ ] `chore(mobile): ios and android targets`

### Commit 11.3: Performance pass against the budgets

- [ ] Measure launch-to-interactive (NFR-P-01), ePub first page (NFR-P-02), page turn (NFR-P-03), 1,000-book library load (NFR-P-04).
- [ ] Confirm no IPC round trip on the page-turn hot path; position saved async (architecture section 9).
- [ ] Index SQLite queries for the 1,000-book target; record measured numbers.
- [ ] `perf: meet performance budgets`

### Commit 11.4: Type-generation hardening (optional)

- [ ] If the command surface has grown, adopt `tauri-specta` to generate `src/ipc/types.ts` so DTO drift is a compile error (architecture section 4.5).
- [ ] `chore(ipc): generate ts types from rust dtos`

### Commit 11.5: Release packaging

- [ ] App icons, identifier, version, CSP review in `tauri.conf.json`.
- [ ] `tauri build` for each desktop platform; verify bundles launch.
- [ ] CI release workflow producing artifacts.
- [ ] `build: release packaging and signing`

---

## Cross-cutting rules (apply to every commit)

- No `invoke`/`listen` outside `src/ipc/`; components stay presentational (architecture section 5).
- No book bytes through `invoke`; only the prose:// protocol (architecture section 4.3).
- No `reqwest`, `rusqlite`, or `std::fs` in `domain/` (architecture section 3).
- No `unwrap()`/`expect()` on a command path; the one exception is `run()` startup wiring.
- Each DTO defined once per language; commands `domain_verb_noun`, events `domain:event`.
- House style: no em dashes, no emojis, specific language, match surrounding code.
- Keep `domain/` coverage at or above 80% as services land (NFR-M-04).
