# Prose Architecture and Engineering Guidelines

This document defines how Prose is built. It is the companion to the requirements in
`spec/software-requirements.typ`: the spec says _what_ the product does, this says _how_
the code is organized so it stays maintainable, flexible, robust, and fast.

Read this before adding a feature. When a rule here conflicts with convenience, follow the
rule or change the rule on purpose with a note here.

---

## 1. The one rule

> Business logic lives in the Rust domain core. The WebView renders and captures input.
> Everything crosses the boundary through a single typed IPC layer.

Every other guideline is a consequence of this. The domain core is pure and platform
independent (NFR-M-01), so it compiles once and behaves identically on all five targets,
and it is unit testable without a UI, filesystem, or network (NFR-M-04).

---

## 2. What lives where

Prose has two runtimes: the **Rust core** (one process, shared across all platforms) and
the **WebView** (the system WebKit/WebView2 running Vue). The split is deliberate.

| Concern                                 | Owner                             | Why                                                   |
| --------------------------------------- | --------------------------------- | ----------------------------------------------------- |
| Library catalog, metadata, IDs          | Rust                              | Single source of truth, fast queries, atomic writes   |
| Book file storage and import            | Rust                              | Filesystem access, content hashing, offline guarantee |
| Reading position, bookmarks, highlights, sessions | Rust                    | Persisted, synced, conflict-resolved in one place     |
| Settings persistence                    | Rust                              | One authority, atomic, emitted to every window        |
| WebDAV sync engine                      | Rust                              | Background task, TLS, resumability, conflict rules    |
| Credential storage                      | Rust                              | OS keychain via a single port                         |
| Rendering reflowable ePub               | WebView (foliate-js)              | DOM is the rendering engine                           |
| Rendering PDF pages                     | WebView (pdf.js)                  | Canvas rendering, partial loads                       |
| UI, navigation, gestures                | WebView (Vue, PrimeVue, Tailwind) | Native WebView input, responsive layout               |
| Capturing the current locator           | WebView, sent to Rust             | The renderer knows the position; Rust stores it       |

The mental model: **Rust decides and remembers, the WebView shows and reacts.** The reader
"opens" a book in the UI, but the bytes are served and the position is stored by Rust.

Do not duplicate domain logic in TypeScript. The frontend may hold a reactive _copy_ of
state for display, but the authority is always Rust.

---

## 3. Rust module layout

The core follows ports and adapters. Dependencies point inward only: `adapters` and `ipc`
depend on `domain`; `domain` depends on nothing app specific.

```
src-tauri/src/
  main.rs              bin entry, calls prose_lib::run()
  lib.rs               run(): builds adapters, injects them, registers commands + protocol
  state.rs             AppState: owns the wired services, lives in Tauri State

  domain/              THE CORE. No tauri, no reqwest, no std::fs, no tokio fs.
    mod.rs
    model.rs           Book, BookId, Locator, Progress, Bookmark, Highlight, Settings
    ports.rs           traits: BookRepository, ReaderAdapter, RemoteStore,
                       CredentialStore, Clock
    library.rs         LibraryService
    reading.rs         ReadingService  (position + progress logic)
    annotation.rs      AnnotationService
    sync.rs            SyncService      (conflict resolution lives here)
    settings.rs        SettingsService
    error.rs           DomainError (thiserror)

  adapters/            DRIVEN adapters: concrete implementations of the ports above.
    mod.rs
    storage/           SQLite + filesystem implementation of BookRepository etc.
    readers/
      epub.rs          ReaderAdapter for ePub 2/3 (metadata, resource listing)
      pdf.rs           ReaderAdapter for PDF (metadata, page count)
    webdav.rs          RemoteStore over reqwest_dav
    credentials.rs     CredentialStore over keyring

  ipc/                 DRIVING adapter: the only Rust that knows about Tauri commands.
    mod.rs             registers handlers
    library.rs         #[tauri::command] fns -> call LibraryService
    reading.rs
    annotation.rs
    sync.rs
    settings.rs
    dto.rs             serde request/response structs (NOT domain types)
    event.rs           event name constants + payload structs
    error.rs           AppError: the serializable boundary error

  protocol.rs          prose:// URI scheme: streams book resources to the renderer
```

Rules:

- **`domain` imports no I/O.** If a domain file needs `use reqwest` or `use rusqlite`, the
  logic is in the wrong layer. Move the I/O behind a port.
- **Ports are traits, defined in `domain/ports.rs`.** Adapters implement them. Services take
  ports as generic params or `Arc<dyn Trait>` so tests inject fakes.
- **A new book format is a new file in `adapters/readers/`** implementing `ReaderAdapter`,
  registered in one place. No domain or UI change (NFR-M-02).
- **One service per aggregate.** Services are the only thing `ipc` calls. Commands stay thin.

Example port and service shape:

```rust
// domain/ports.rs
pub trait BookRepository: Send + Sync {
    fn insert(&self, book: &Book) -> Result<(), DomainError>;
    fn list(&self, query: &LibraryQuery) -> Result<Vec<Book>, DomainError>;
    fn save_progress(&self, id: &BookId, p: &Progress) -> Result<(), DomainError>;
    // ...
}

pub trait ReaderAdapter: Send + Sync {
    fn supports(&self, format: Format) -> bool;
    fn probe(&self, bytes: &[u8]) -> Result<BookMetadata, DomainError>; // title, author, cover
}

// domain/library.rs
pub struct LibraryService<R: BookRepository> {
    repo: R,
    readers: ReaderRegistry,
}

impl<R: BookRepository> LibraryService<R> {
    pub fn import(&self, bytes: Vec<u8>, format: Format) -> Result<Book, DomainError> {
        let meta = self.readers.for_format(format)?.probe(&bytes)?;
        let book = Book::new(BookId::from_content(&bytes), meta);
        self.repo.insert(&book)?;
        Ok(book)
    }
}
```

Tests construct `LibraryService` with an in-memory `BookRepository` and a fake reader. No
files, no network.

---

## 4. The IPC contract (TypeScript <-> Rust)

This is the most touched seam in the codebase, so it has the strictest conventions.

### 4.1 Commands: request/response, TS calls Rust

Use `invoke` for synchronous-feeling request/response with small JSON payloads.

- **One typed wrapper per command in `src/ipc/`. Components never call `invoke` directly.**
  This keeps the boundary testable (mock one module) and the payload shapes in one place.
- **Naming: `domain_verb_noun`, snake_case.** `library_import_book`, `library_list`,
  `reading_save_position`, `annotation_add_bookmark`, `sync_configure`, `settings_patch`.
- **Payloads are flat DTOs, not domain structs.** Define them in `ipc/dto.rs`, mirror them
  in `src/ipc/types.ts`. The boundary is a contract; it should not move when an internal
  domain type is refactored.

```rust
// src-tauri/src/ipc/library.rs
#[tauri::command]
pub async fn library_import_book(
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<BookDto, AppError> {
    let book = state.library.import_from_path(&path).await?;
    Ok(BookDto::from(book))
}
```

```ts
// src/ipc/library.ts  -- the ONLY place invoke() appears for the library
import { invoke } from '@tauri-apps/api/core'
import type { Book } from './types'

export function importBook(path: string): Promise<Book> {
  return invoke('library_import_book', { path })
}

export function listBooks(query: LibraryQuery): Promise<Book[]> {
  return invoke('library_list', { query })
}
```

### 4.2 Events: Rust pushes to TS

Use events for anything asynchronous or streaming: sync progress, import progress, a library
or settings change that any open window must reflect. Do not poll with `invoke`.

- **Naming: `domain:event`.** `sync:progress`, `sync:finished`, `library:changed`,
  `settings:changed`, `import:progress`.
- **Listen in one typed module (`src/ipc/events.ts`), fan out to stores.** Components
  subscribe to a store, not to raw Tauri events.

```ts
// src/ipc/events.ts
import { listen } from '@tauri-apps/api/event'
import type { SyncProgress } from './types'

export const onSyncProgress = (cb: (p: SyncProgress) => void) =>
  listen<SyncProgress>('sync:progress', (e) => cb(e.payload))
```

### 4.3 Large book content: custom protocol, never invoke

Do **not** push book bytes through `invoke` (it serializes and copies whole files, which
breaks the 2-second open target and wastes memory). Serve book resources through a custom URI
scheme so foliate-js and pdf.js fetch them directly, with HTTP range support for PDF partial
loads.

```
prose://book/{book_id}/{resource_path}
```

Registered in `protocol.rs` with `register_asynchronous_uri_scheme_protocol`, reading from the
stored file and honoring `Range` headers. The renderer just sees a URL.

### 4.4 Errors

- Rust domain uses `thiserror` (`DomainError`). At the boundary, map to `AppError`, a
  serializable struct, never a bare string.
- TS wrappers reject with a typed error the UI can branch on.

```rust
#[derive(serde::Serialize)]
pub struct AppError { pub code: String, pub message: String }
// e.g. { code: "book_not_found", message: "..." }
```

- **No `unwrap()` or `expect()` on a command path.** Propagate `Result`. The only acceptable
  `expect` is in `run()` startup wiring.

### 4.5 Keeping types in sync

The risk in a two-language boundary is silent DTO drift. Two acceptable strategies:

1. **Hand-mirrored types** in `ipc/dto.rs` and `src/ipc/types.ts`, kept in one file each. Simple,
   zero tooling, relies on review.
2. **`tauri-specta`** to generate `types.ts` from the Rust DTOs at build time. Recommended once
   the command surface grows past a handful, because it makes drift a compile error.

Start with (1); adopt (2) when the surface stabilizes. Either way, **DTO definitions live in
exactly one place per language.**

---

## 5. Frontend layer (Vue, PrimeVue, Tailwind)

The WebView UI is Vue 3 with `<script setup>` and TypeScript. It has its own small layered
structure that mirrors the "thin boundary, logic elsewhere" discipline of the core.

```
src/
  main.ts            app bootstrap, PrimeVue + theme registration
  App.vue            shell: layout, root theme attribute
  ipc/               the ONLY place invoke()/listen() appear (see section 4)
  readers/           BookRenderer implementations (see section 6)
  stores/            reactive state, seeded from Rust, updated by events
  composables/       reusable view logic (useLibrary, useReader, useSettings)
  views/             one component per screen (Library, Reader, Settings)
  components/         presentational, PrimeVue-based, no invoke
```

Rule: components are presentational. They read from stores and call composables; only
`src/ipc` touches Tauri. A component that imports `@tauri-apps/api` is a smell.

### 5.1 Component library: PrimeVue (primary)

PrimeVue is the primary UI component library. Build screens from PrimeVue components rather
than hand-rolling interactive widgets:

- Library grid and list: `DataView` (its grid and list layouts) backed by the library store.
- Dialogs, menus, toasts, sliders, selects, buttons: the matching PrimeVue components.
- Reader chrome (toolbars, the TOC drawer, the settings panel): `Drawer`, `Menu`, `Slider`,
  `Select`, and friends.

Do not introduce a second component library. If a needed widget does not exist in PrimeVue,
build it as a local component styled with Tailwind, not by pulling in another framework.

### 5.2 Styling: Tailwind CSS (utility layer)

Tailwind CSS v4 (wired through `@tailwindcss/vite`) is the utility layer for layout, spacing,
and composition around PrimeVue components. Division of labor:

- PrimeVue owns behavior-bearing widgets and their internal styling, driven by design tokens.
- Tailwind owns layout, spacing, flex and grid, and one-off presentational styling.
- No third styling system and no scattered bespoke CSS files. Prefer Tailwind utilities in the
  template over standalone style sheets.

Use the `tailwindcss-primeui` plugin so PrimeVue design tokens are available as Tailwind
utilities and layer ordering stays correct, so PrimeVue components are not clobbered by
Tailwind's preflight.

### 5.3 Theming: one switch, three sources

The three reading themes (light, dark, sepia from FR-CUST-04) are driven from a single
Rust-owned setting (see section 7). PrimeVue styled mode with `@primeuix/themes` is the token
source; the shell applies the active theme by setting one attribute on the root element.

- Register PrimeVue with a preset and a `darkModeSelector` pointing at the root attribute.
- Point Tailwind's dark variant at the same selector, so one toggle flips both.
- On `settings:changed`, `App.vue` sets the root attribute; PrimeVue tokens, Tailwind dark
  utilities, and the ePub renderer style all react to that single change.

```ts
// src/main.ts
import Aura from '@primeuix/themes/aura'
app.use(PrimeVue, {
  theme: { preset: Aura, options: { darkModeSelector: '[data-theme="dark"]' } },
})
```

```css
/* src/assets/main.css */
@custom-variant dark ([data-theme="dark"] &);
```

Sepia is a third token set (a preset variation or token override) selected by the same root
attribute. Theme is the one setting the Vue shell consumes directly, since it styles the whole
application, not just the reading view.

---

## 6. Adaptable readers

A "reader" exists on both sides, and the two layers are distinct.

**Rust `ReaderAdapter` (metadata and resources):** parses the file enough to extract title,
author, and cover, reports the format, and lists resources for the protocol handler. One
adapter per format in `adapters/readers/`, selected by a small registry. Candidates: the
`epub` crate for ePub, `lopdf`/`pdf` for PDF.

**TS `BookRenderer` (pixels):** renders content in the WebView and reports the current locator.
One implementation per format, selected by the format Rust reports.

```ts
// src/readers/Renderer.ts
export interface BookRenderer {
  load(source: string): Promise<void> // a prose:// URL
  goToLocator(locator: Locator): Promise<void>
  next(): void
  prev(): void
  onLocationChange(cb: (l: Locator) => void): void
  applyStyle(style: ReadingStyle): void // ePub only; PDF is fixed-layout
}
// EpubRenderer wraps foliate-js, PdfRenderer wraps pdf.js
```

Adding a format = one Rust adapter + one TS renderer + one registry entry on each side. Nothing
in the domain core or the UI shell changes. That is the test of whether this layer is right.

The `Locator` type is the shared currency of position. Keep it format neutral at the domain
boundary (for ePub a CFI or progression fraction, for PDF a page index plus offset), serialized
as an opaque structure the renderer produced and the renderer consumes. The domain stores and
compares it; it does not interpret it, except for the "furthest position" comparison which uses
the progression fraction.

---

## 7. Settings

Settings are domain state with one authority (Rust) and reactive copies in the UI.

Flow:

1. On startup the UI calls `settings_get` once and seeds a reactive store.
2. A change in the UI calls `settings_patch` with only the changed fields.
3. Rust validates, persists atomically, and emits `settings:changed`.
4. Every window updates its store from the event, so all windows stay consistent.

Rules:

- **Settings struct is versioned** (`schema_version`) and every field has a default. A missing
  or unknown field falls back to default, so an older or newer file never crashes the app
  (robustness, forward and backward tolerance).
- **Reading-view settings** (font family/size, line spacing, margins) are applied by the
  `EpubRenderer` via `applyStyle`; they have no effect on PDF.
- **Theme** (light/dark/sepia) applies to both the reading view and the app shell (FR-CUST-04),
  so it is the one setting the Vue shell also consumes directly.
- Settings persist through the same `BookRepository`-style port and store as the rest of the
  data, so there is one atomic local store, not a scattered config file.

---

## 8. Persistence and local-first sync

### 7.1 Local store

Use **SQLite** as the single local store for catalog, progress, annotations, settings, and
sync state. Book files and cover images live on disk, referenced by path. Rationale:

- Transactions give atomic writes for free (NFR-R-02).
- Indexed queries load 1,000 books in well under a second (NFR-P-04).
- One file to back up, one place for migrations.

It sits behind the repository ports, so tests use an in-memory or temp-file database and the
store is replaceable. Candidate crate: `rusqlite` (bundled SQLite) or `sqlx`.

Book identity is a **content hash** (`BookId::from_content`), so the same book imported on two
devices converges to one ID and sync does not duplicate it.

### 7.2 Sync engine

Sync is a background task in `SyncService`, started after configuration, never blocking reading
(FR-SYNC-05). Each syncable record carries `last_modified` (timestamp) and an origin device id.

Conflict resolution (FR-SYNC-04), implemented and unit tested in `domain/sync.rs`:

- **Reading position:** keep the furthest progression fraction.
- **Everything else:** last write wins by `last_modified` timestamp.

Resumability (NFR-R-03): maintain a local **outbox** of pending changes and a per-file sync
cursor (WebDAV ETag). Operations are idempotent and keyed by stable IDs, so an interrupted sync
re-runs from the cursor without loss or duplicates. A change is applied locally immediately and
uploaded on the next successful connection (FR-SYNC-06).

All network and TLS handling lives in `adapters/webdav.rs` behind the `RemoteStore` port. The
domain sync logic never sees `reqwest`; it works against the port and is fully testable with a
fake remote.

---

## 9. Concurrency and performance

- **Commands that do I/O are `async`.** Never block the WebView thread.
- **CPU-heavy work** (metadata probe, content hashing of a large file) runs on
  `spawn_blocking`, not on the async executor.
- **Stream book content** through the custom protocol with range support; never load a whole
  book into JS memory.
- **Sync runs on its own task** and communicates only through events.
- The performance budgets are in the spec (NFR-P). Treat them as fitness functions: a page turn
  is a renderer-local operation (no IPC round trip on the hot path), position is saved
  asynchronously after the turn, not before it paints.

---

## 10. Testing

| Layer              | How                                        | Target                         |
| ------------------ | ------------------------------------------ | ------------------------------ |
| `domain/*`         | Pure unit tests, all ports faked, no I/O   | ≥80% line coverage (NFR-M-04)  |
| `adapters/storage` | Integration test against a temp SQLite db  | Behavior of the real store     |
| `adapters/webdav`  | Against a mock WebDAV server               | Sync round trips, resumability |
| `adapters/readers` | Fixture ePub/PDF files in `tests/fixtures` | Correct metadata extraction    |
| `src/ipc/*`        | Vitest with `invoke` mocked                | Payload shapes, error mapping  |
| Vue components     | Vitest + Testing Library                   | Rendering and interaction      |

The domain tests are the load-bearing ones. If a piece of logic is hard to test without a file
or a socket, it is in the wrong layer; move it behind a port.

---

## 11. Conventions cheat-sheet

**Do**

- Put new logic in `domain`, behind a port if it touches the outside world.
- Add commands as thin `ipc` wrappers that call a service.
- Keep one typed wrapper per command in `src/ipc`; components import those.
- Return `Result<_, AppError>`; map domain errors at the boundary.
- Name commands `domain_verb_noun`, events `domain:event`.
- Define each DTO once per language.

**Do not**

- Call `invoke` from a Vue component.
- Push book bytes through `invoke`.
- Put `reqwest`, `rusqlite`, or `std::fs` in `domain`.
- Reimplement domain logic in TypeScript.
- `unwrap()` on a command path.
- Add a format by editing the domain core or the UI shell.

**Naming reference**

| Thing        | Convention         | Example                 |
| ------------ | ------------------ | ----------------------- |
| Command      | `domain_verb_noun` | `reading_save_position` |
| Event        | `domain:event`     | `sync:progress`         |
| Rust DTO     | `*Dto` suffix      | `BookDto`               |
| Port (trait) | role noun          | `RemoteStore`           |
| Adapter      | technology noun    | `WebDavRemoteStore`     |
| Book ID      | content hash       | `BookId`                |

---

## 12. Candidate crates and libraries

Consistent with the dependency table in the spec, these are candidates pending final selection,
not hard mandates. Confirm license and maintenance before adopting.

| Need               | Candidate            | Layer                    |
| ------------------ | -------------------- | ------------------------ |
| ePub rendering     | foliate-js           | TS renderer              |
| PDF rendering      | pdf.js               | TS renderer              |
| ePub metadata      | `epub` crate         | Rust reader adapter      |
| PDF metadata       | `lopdf` / `pdf`      | Rust reader adapter      |
| Local store        | `rusqlite` (bundled) | Rust storage adapter     |
| WebDAV             | `reqwest_dav`        | Rust remote adapter      |
| Credentials        | `keyring`            | Rust credential adapter  |
| Error types        | `thiserror`          | Rust domain              |
| TS type generation | `tauri-specta`       | Build tooling (optional) |
