#let project-name = "Prose"
#let version = "0.0.1"
#let authors = ("Elia Salerno",)
#let today = datetime.today().display("[day]. [month repr:long] [year]")

// Typography & Paragraphs
#set text(font: "Helvetica Neue", size: 10pt, lang: "en")
#set par(leading: 0.65em)
#set heading(numbering: "1.1")

// Metadata
#set document(
  title: project-name + " Software Requirements Specification",
  author: authors,
)

// ==========================================
// PAGE LAYOUT, HEADERS, & FOOTERS
// ==========================================
#set page(
  paper: "a4",
  margin: (x: 2.5cm, y: 2.5cm),
  header: context {
    if counter(page).get().first() > 1 [
      #set text(size: 9pt, fill: luma(120))
      #grid(
        columns: (1fr, 1fr),
        align(left)[#project-name SRS],
        align(right)[#today, Version #version],
      )
      #line(length: 100%, stroke: 0.5pt + luma(180))
    ]
  },
  footer: context {
    if counter(page).get().first() > 1 [
      #line(length: 100%, stroke: 0.5pt + luma(180))
      #set text(size: 9pt, fill: luma(120))
      #align(center)[#counter(page).display()]
    ]
  },
)

// Heading Styling Rules
#show heading.where(level: 1): it => {
  v(1em)
  text(size: 16pt, weight: "bold")[#it]
  v(0.5em)
}
#show heading.where(level: 2): it => {
  v(0.8em)
  text(size: 13pt, weight: "bold")[#it]
  v(0.3em)
}

// ==========================================
// DYNAMIC COMPONENT MACROS
// ==========================================
#let req-counters = state("req-counters", (:))

#let req-table(prefix, descriptions) = {
  req-counters.update(counters => {
    counters.insert(prefix, 0)
    counters
  })
  table(
    columns: (auto, 1fr),
    stroke: 0.5pt + luma(200),
    fill: (_, row) => if row == 0 { luma(235) } else if calc.odd(row) { luma(250) } else { none },
    inset: 8pt,
    [*ID*], [*Description*],
    ..descriptions.enumerate().map(((i, desc)) => {
      let num = str(i + 1)
      if num.len() < 2 { num = "0" + num }
      ([#prefix\-#num], desc)
    }).flatten()
  )
}

#let srs-table(columns, headers, ..cells) = {
  table(
    columns: columns,
    stroke: 0.5pt + luma(200),
    fill: (_, row) => if row == 0 { luma(235) } else if calc.odd(row) { luma(250) } else { none },
    inset: 8pt,
    ..headers.map(h => [*#h*]),
    ..cells.pos()
  )
}

// ==========================================
// TITLE PAGE
// ==========================================
#page(
  margin: (x: 2.5cm, y: 3cm),
  header: none,
  footer: none,
)[
  #set text(size: 9pt, fill: luma(120))
  #grid(
    columns: (1fr, 1fr),
    align(left)[#project-name SRS],
    align(right)[#today, Version #version],
  )

  #v(1fr)

  #align(center)[
    #v(2em)
    #text(size: 32pt, weight: "light")[Software Requirements\ Specification]
    #v(1em)
    #text(size: 20pt, weight: "bold", fill: luma(60))[#project-name]
  ]

  #v(2fr)

  #text(size: 10pt)[
    *Authors:*\
    #authors.join(", ")
  ]
]

// ==========================================
// DOCUMENT CONTROL / REVISIONS
// ==========================================
#heading(numbering: none, outlined: false)[Versions]

#srs-table(
  (auto, 2fr, auto),
  ("Date", "Changes", "Version"),
  [#today], [Initial requirements specification], [#version],
)

#pagebreak()

// Table of Contents
#outline(indent: auto, depth: 2)

#pagebreak()

// ==========================================
// 1. INTRODUCTION
// ==========================================
= Introduction

== Purpose
Prose is a lightweight, cross-platform reader for ePub 2, ePub 3, and PDF books. A reader can open and read these books, customize the reading experience, and manage a personal library. Prose is local-first: the full library and every reading feature work offline. An optional WebDAV server lets the reader source books and synchronize reading position, bookmarks, highlights, settings, and book files across devices. The design priorities are speed, a minimal and usable interface, and a clean, highly testable architecture.

== Scope
*In scope*
- Reading ePub 2, ePub 3, and PDF books.
- A locally stored, offline library populated by local file import, presented as a cover grid and a detail list, with search, filter, and sort.
- Reading customization for reflowable ePub content: font family and size, line spacing and margins, and light, dark, and sepia themes.
- Table-of-contents navigation, bookmarks, highlights, and offline dictionary lookup.
- Optional WebDAV sourcing of books and synchronization of reading position, bookmarks, highlights, settings, and book files.
- Distribution for macOS, Windows, Linux, iOS, and Android from a single shared codebase.

*Out of scope*
- Plugin or extension system.
- LLM or other AI features.
- Ebook formats other than ePub and PDF, and conversion between formats.
- Reading or removing DRM-protected or otherwise encrypted books.
- Full-text search within an open book.
- User-defined collections or shelves.
- Alternative reading layouts such as continuous vertical scrolling.
- Synchronization backends other than WebDAV, and online catalogs such as OPDS.
- Multi-user accounts, sharing, or social features.

== Glossary
#srs-table(
  (10em, 1fr),
  ("Term", "Definition"),
  [ePub], [Open ebook format (versions 2 and 3 in scope) containing reflowable HTML and CSS content.],
  [PDF], [Portable Document Format, a fixed-layout document format.],
  [Reflowable content], [Text that adapts to screen size and to the reader's font and spacing settings. Applies to ePub.],
  [Fixed-layout content], [Pages with a fixed visual layout that do not reflow. Applies to PDF.],
  [WebDAV], [HTTP extension (RFC 4918) for reading and writing files on a remote server.],
  [Local-first], [Approach in which data lives primarily on the device and works fully offline, with optional synchronization.],
  [Reading position], [Saved location within a book at which the reader last stopped.],
  [Ports and adapters], [Architecture that isolates domain logic behind interfaces (ports), with interchangeable implementations (adapters). Also called hexagonal.],
  [Adapter], [Component implementing a port for a specific technology, such as a book format, storage, or remote service.],
  [TLS], [Transport Layer Security, the protocol securing HTTPS connections.],
)

#pagebreak()

// ==========================================
// 2. OVERALL DESCRIPTION
// ==========================================
= Overall Description

== System Environment
Prose is a single application distributed for desktop and mobile, built on Tauri 2 with a Rust core and the operating system's native WebView for the user interface. Supported platforms, following Tauri 2 requirements, are macOS 10.15 or later, Windows 10 or later, Linux distributions providing webkit2gtk 4.1, iOS 13 or later (some native mobile plugins require iOS 14), and Android 7.0 (API level 24) or later. Network access is required only for optional WebDAV synchronization; all other functionality runs offline. Performance targets assume reference hardware: a consumer laptop or mid-range smartphone released in 2020 or later.

== Architecture Overview
Prose follows a ports-and-adapters (hexagonal) architecture. A platform-independent domain core owns the library, reading, annotation, and synchronization logic and exposes ports. Driving adapters (the user interface) and driven adapters (one reader per book format, local storage, the WebDAV client, and the secure credential store) connect to the core through those ports. This keeps the core testable in isolation and lets new book formats or storage backends be added as adapters without changing the core. The binding requirements appear in the Maintainability and Testability section.

== User Classes and Characteristics
#srs-table(
  (auto, 1fr),
  ("Class", "Characteristics"),
  [Reader], [The single end user. Reads and manages their own books on one or more personal devices. No skill is assumed beyond optionally configuring a WebDAV server. Prose has no administrator, multi-user, or shared-account roles.],
)

== External Interfaces
#srs-table(
  (12em, 12em, 1fr),
  ("Interface", "Type", "Purpose"),
  [WebDAV server], [HTTPS, RFC 4918], [Optional source of books and target for synchronization.],
  [Local filesystem], [Operating system file APIs], [Importing books and storing the local library, covers, and settings.],
  [Secure credential store], [OS keychain or keystore], [Storing WebDAV credentials.],
  [System WebView], [OS-provided (WebKit or WebView2)], [Rendering the user interface and reflowable content.],
  [Offline dictionary], [Bundled data set], [Providing word definitions for lookup.],
)

#pagebreak()

// ==========================================
// 3. FUNCTIONAL REQUIREMENTS
// ==========================================
= Functional Requirements

== Library Management
#req-table("FR-LIB", (
  [The system shall import .epub and .pdf files selected from the local filesystem into the library.],
  [The system shall extract the title, author, and cover image from each imported book.],
  [The system shall store imported books and their metadata locally so the library is available offline.],
  [The system shall present the library as a cover grid and as a detail list, switchable by the reader.],
  [The system shall let the reader search and filter the library by title and author.],
  [The system shall let the reader sort the library by title, author, last-read time, and reading progress.],
  [The system shall let the reader remove a book from the library after confirmation.],
))

== Reading and Rendering
#req-table("FR-READ", (
  [The system shall open and render ePub 2 and ePub 3 books as reflowable content.],
  [The system shall open and render PDF documents as fixed-layout pages.],
  [The system shall display the open book's table of contents and navigate to a selected entry.],
  [The system shall move forward and backward through the open book one page at a time.],
  [The system shall let the reader zoom and fit PDF pages, defaulting to fit-to-width.],
  [The system shall store the reading position per book and resume at that position when the book is reopened.],
  [The system shall display the current reading progress as a percentage.],
))

== Reading Customization
#req-table("FR-CUST", (
  [The system shall let the reader choose the reading font family from a bundled set, for reflowable ePub content.],
  [The system shall let the reader adjust the reading font size, for reflowable ePub content.],
  [The system shall let the reader adjust line spacing and page margins, for reflowable ePub content.],
  [The system shall provide light, dark, and sepia themes applied to both the reading view and the application interface.],
  [The system shall store the reader's customization settings and apply them across sessions and books.],
))

== Annotations and Reference
#req-table("FR-NOTE", (
  [The system shall let the reader create, view, and delete bookmarks at the current reading location.],
  [The system shall let the reader highlight selected text and view or delete existing highlights, for content with selectable text.],
  [The system shall let the reader select a word and view its definition from the offline dictionary, for content with selectable text.],
))

== WebDAV Synchronization
#req-table("FR-SYNC", (
  [The system shall let the reader configure a single WebDAV server by URL, username, and password.],
  [The system shall list the .epub and .pdf files in the configured WebDAV folder and download a selected file into the local library.],
  [The system shall synchronize reading position, bookmarks, highlights, customization settings, and book files between the local library and the configured WebDAV server.],
  [The system shall resolve synchronization conflicts by keeping the furthest reading position for reading position, and the most recently modified value by timestamp for all other synchronized data.],
  [The system shall remain fully usable with no WebDAV server configured, and shall perform synchronization in the background without blocking reading.],
  [The system shall apply reading and annotation changes to the local library immediately and upload them to the configured WebDAV server on the next successful connection.],
))

#pagebreak()

// ==========================================
// 4. NON-FUNCTIONAL REQUIREMENTS
// ==========================================
= Non-functional Requirements

== Performance
#req-table("NFR-P", (
  [The application shall reach an interactive state within 3 seconds of launch on reference hardware.],
  [The system shall render the first page of an ePub of up to 10 MB within 2 seconds of opening it.],
  [The system shall complete a page turn within 100 ms.],
  [The system shall load and display a library of up to 1,000 books within 1 second.],
))

== Security and Privacy
#req-table("NFR-S", (
  [All WebDAV communication shall use HTTPS with TLS 1.2 or higher, preferring TLS 1.3.],
  [WebDAV credentials shall be stored only in the operating system secure credential store, never in plaintext.],
  [The application shall send user data to no service other than the reader's configured WebDAV server, and shall include no telemetry or analytics.],
))

== Reliability and Offline Operation
#req-table("NFR-R", (
  [Every reading feature shall function without network connectivity once a book is in the local library.],
  [The system shall write local library, progress, and annotation data atomically so an interrupted operation cannot corrupt the library.],
  [An interrupted synchronization shall be resumable without data loss or duplicate entries.],
))

== Maintainability and Testability
#req-table("NFR-M", (
  [The system shall follow a ports-and-adapters architecture in which the domain core is independent of the user interface, platform, filesystem, and network.],
  [Each supported book format shall be implemented as an adapter behind a single reader port, so adding a format requires no change to the domain core or the user interface.],
  [Local storage and WebDAV synchronization shall each sit behind a dedicated port that can be replaced with a test double.],
  [The domain core shall be covered by automated unit tests that run without a user interface, filesystem, or network, achieving at least 80% line coverage.],
))

== Portability
#req-table("NFR-X", (
  [The system shall run on macOS, Windows, Linux, iOS, and Android from a single shared codebase.],
  [The user interface shall accept both pointer and touch input and adapt to desktop and mobile screen sizes.],
))

#pagebreak()

// ==========================================
// 5. DEFINITION & DEPENDENCIES
// ==========================================
= Definition

== Dependencies
Platform dependencies are fixed by the current project setup. Format, synchronization, dictionary, and credential libraries are candidates pending selection, marked with version "candidate".

#srs-table(
  (12em, auto, auto, 1fr),
  ("Name", "Version", "License", "Used for"),
  [Tauri], [2.x], [Apache-2.0 / MIT], [Cross-platform application shell with a Rust core and the system WebView.],
  [Vue], [3.5.x], [MIT], [User interface.],
  [Vite], [6.x], [MIT], [Frontend build tooling.],
  [TypeScript], [5.6.x], [Apache-2.0], [Frontend language.],
  [foliate-js], [candidate], [MIT], [Rendering reflowable ePub 2 and ePub 3 content.],
  [pdf.js], [candidate], [Apache-2.0], [Rendering fixed-layout PDF pages.],
  [reqwest_dav], [candidate], [MIT / Apache-2.0], [WebDAV access and file synchronization.],
  [keyring], [candidate], [MIT / Apache-2.0], [Storing WebDAV credentials in the OS secure credential store.],
)