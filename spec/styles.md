# Prose: Elegant Serif Design System & Style Guide

This document defines the user interface style guide for Prose. The interface is designed to evoke the tactile feel of an editorial publication, a literary magazine, or a finely bound book, emphasizing typography over heavy UI controls.

---

## 1. Core Principles

- **Typography as Interface**: The UI uses a Serif font (`Literata` or `Georgia`) for all elements, including menus, buttons, labels, and forms. This creates an immediate connection to reading and literature.
- **Minimal Labeling**: Rely on self-explanatory layouts and clear hierarchies. Avoid repetitive or redundant labels. Use short, singular nouns for controls (e.g., "Theme", "Size").
- **Zero-Icon Aesthetics**: Icons are avoided unless functionally critical (e.g., a close button). Labels and sliders speak for themselves without needing visual metadata like palette, text, or margin icons.
- **Generous Contrast & Whitespace**: UI containers feature ample breathing room. Dividers are hairline thin (`1px` width with low contrast) or omitted entirely in favor of whitespace.

---

## 2. Typography

We use **Literata** as our primary application and reading typeface. It is a serif typeface designed for comfortable screen reading, offering excellent legibility even in small UI sizes (buttons, options, labels).

### Typography Standards

- **UI & Reading Font**: Always use the defined Serif typography hierarchy. Monospace (`mono`) fonts are prohibited in both code layouts and parameter readouts.
- **Left-Bound Text**: Standard text blocks must be left-aligned (`text-left`). Do not use text justification (`text-justify` or fill) for preview blocks or content components.

### Fallbacks

- System Serif (e.g., `Georgia`, `Times New Roman`, `serif`) for platforms where web fonts are loading or disabled.

### UI Typography Token Configuration

```css
@theme {
  --font-serif: 'Literata', 'Lora', 'Georgia', serif;
}

html {
  font-family: var(--font-serif);
}
```

---

## 3. Color Palettes (Editorial Neutrality)

Colors are chosen to represent physical paper types, ink qualities, and comfortable lighting environments.

### Light (Classic Paper)

- **Background**: `#faf9f5` (Warm cream/paper white)
- **Text**: `#18181b` (Deep charcoal ink)
- **Borders**: `#e5e5e0` (Light graphite)
- **Accent**: `#18181b` (Ink black)

### Dark (Nox/Slate)

- **Background**: `#121212` (Vellum black)
- **Text**: `#e4e4e7` (Soft silver)
- **Borders**: `#27272a` (Charcoal gray)
- **Accent**: `#f4f4f5` (Silver white)

### Sepia (Vintage Book - Richer & Darker)

- **Background**: `#d7c49e` (Classic aged book page, deep warm sepia)
- **Text**: `#36291b` (Dark earth/umber ink)
- **Borders**: `#c5b38d` (Aged parchment border)
- **Accent**: `#705235` (Leather-brown accent)

---

## 4. UI Components Specification

### 4.1 Dropdowns & Selects

- **No leading icons**: Dropdown items are simple text lists.
- **Visuals**: Borderless or hairline-bordered fields that blend into the card background. Hover states use a very subtle shade change (`--accent-color-light`).

### 4.2 Sliders & Controls

- **Minimalist Handle**: Small, circular handles that match the primary accent color. No shadows or glowing rings.
- **Range track**: Hairline track that highlights active states in a clean ink-on-paper style.

### 4.3 Dialogs & Drawers

- **Typography headers**: Simple, large serif headings without decorative icons.
- **Whitespace padding**: Large padding gaps (`p-8`) to maintain breathing room and feel spacious.
- **No enclosing cards**: Settings views and control panels must not be enclosed in bordered card components. They flow directly on the application viewport background, blending naturally into the paper-like canvas.

### 4.4 Sidebar Navigation

- **Icon menu toggle**: Use simple functional icons (e.g., `menu` and `close` from Material Symbols) inside a clean, hairline-bordered button for toggling sidebar navigation, rather than text labels.
- **Hiding in Reader Mode**: The menu toggle button must be completely hidden during active reading (`currentView === 'reader'`) to remove distraction from the viewport canvas.
- **Generous Canvas Width**: Page view content has a wider container (`max-w-3xl`) to display typography lines cleanly with optimal readability and generous margins.
- **Fluid transitions**: When open, the sidebar moves out cleanly from the left margin, pushing content slightly or overlaying on narrow viewports with soft contrast.
- **Zero-icon items**: Menu options contain only clear labels ("Library", "Settings", "Sync") without decorative icons.
- **Active state feedback**: Highlight the active section using a subtle font-weight shift (`font-semibold`) and indent (`translate-x-1`) rather than heavy colored blocks or highlights.

### 4.5 Library Catalog Layout

- **Minimalist Search Input**: Single text input field with low contrast placeholder text and a simple border that highlights upon focus. No trailing/leading icons.
- **Textual Filtering Links**: Sort choices are displayed as simple, capital-spaced or capitalized text pills rather than dropdown selects or icons. Active filters are bordered; inactive options are muted and reveal contrast on hover.
- **Editorial Book Listings**: Books are listed as clean typographic segments separated by thin horizontal lines. On hover, the book title shifts slightly (`translate-x-0.5`) to signal interactiveness without introducing heavy animations.

### 4.6 Reader View & Floating Dock Card

- **Responsive Settings Binding**: Margins, size, and leading spacing are bound directly to the reader's typography configuration in real-time.
- **Non-Scrolling Paginated Layout**: The reading canvas utilizes a tall, fixed height (`h-full` stretching within viewport-locked layouts) with overflow hidden to maximize page content area and cover the viewport with minimal bloat. Book content is chunked dynamically into sequential page blocks of roughly 1400 characters.
- **Click-to-Turn Zones**: Viewport-fixed tap targets cover the left and right margins of the viewport (`fixed left-0`/`fixed right-0` and `top-0 bottom-0`). Tapping the left area turns to the previous page (with `w-resize` cursor); tapping the right area turns to the next page (with `e-resize` cursor). The center column of the screen toggles the floating controls menu. These zones cover the full height of the viewport and extend starting from the outer screen edges inwards, spanning all page padding and margins.
- **Adjustable Turn Area Settings**: A 'Page-Turn Zone' slider in the Settings allows readers to customize the click zone width, ranging from 10% to 45% of the viewport width on each side. When adjusted, temporary low-opacity red overlay blocks (`bg-red-500/10` with `border-red-500/30` borders) appear on the left and right sides of the screen to preview the zone boundaries in real-time, automatically vanishing 1.2 seconds after adjustment stops.
- **Floating Bottom Dock Card**: In-book controls are housed inside a highly compact, rounded pill-shaped card (`rounded-full`) with a thin border and low-profile shadow, centered at the bottom.
- **Interactive Hiding Controls**: The floating dock card can be hidden completely to maximize focus. Dock toggling is triggered by clicking anywhere on the center canvas zone, or via the explicit hide icon on the dock. When hidden, a tiny rounded floating menu icon rests at the bottom to restore controls.
- **Compact Icon-Based Actions**: The dock links use simple, clean functional icons (e.g. `arrow_back`, `toc`, `bookmark`, `visibility_off`) rather than text labels, creating a compact and highly mobile-friendly toolbar.
- **Percentage Only Indicator**: The bottom page count indicator footer is omitted. Progress is displayed solely as a numeric percentage (e.g. `35%`) inside the compact bottom dock card, rendered in the app's default Serif typeface. Monospace (`mono`) layout elements are completely prohibited.
