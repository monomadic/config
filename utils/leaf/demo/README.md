# leaf - Features Demo

**leaf** is a terminal-based Markdown previewer with syntax highlighting, LaTeX rendering, Mermaid diagrams, theme support, and interactive navigation.

## Interactive Features

### Watch Mode

**leaf** can monitor your file for changes and automatically reload the preview. The status bar shows a "reloaded" flash indicator when the file is updated. Scroll position is preserved across reloads.

- Press `w` or `Ctrl+W` to toggle watch mode
- Press `r` or `Ctrl+R` to manually reload the file
- Use `--watch` flag on startup: `leaf --watch file.md`

Source: [demo-watch-mode.md](sources/demo-watch-mode.md)

![Watch Mode](images/demo-watch-mode.png)

### Open in Editor

Quickly open the current file in your preferred external editor without leaving **leaf**. When you return, the file is automatically reloaded with the latest changes.

- Press `Ctrl+E` to open in the configured editor
- Press `Shift+E` to pick an editor (nano, vim, code, subl, emacs)
- Use `--editor` flag on startup: `leaf --editor vim file.md`

Source: [demo-open-editor.md](sources/demo-open-editor.md)

![Open in Editor](images/demo-open-editor.png)

### File Picker

Browse and open Markdown files without leaving **leaf**. Two modes are available: a fuzzy finder for quick search by filename, and a directory browser for navigating the file tree.

- Press `Ctrl+P` for fuzzy file picker
- Press `Shift+P` for directory browser
- Use `--picker` flag on startup: `leaf --picker`

Source: [demo-file-picker.md](sources/demo-file-picker.md)

![File Picker](images/demo-file-picker.png)

### Table of Contents

A sidebar displays the document's heading structure for quick navigation. **leaf** intelligently normalizes heading levels and highlights the currently visible section.

- Press `t` to toggle the TOC sidebar
- Press `1-9` to jump directly to a heading

Source: [demo-toc-sidebar.md](sources/demo-toc-sidebar.md)

![Table of Contents](images/demo-toc-sidebar.png)

### Search

Full-text search with visual highlighting of all matches. Navigate between results with match counter feedback in the status bar.

- Press `/` or `Ctrl+F` to start searching
- Press `n` to jump to next match
- Press `Shift+N` to jump to previous match

Source: [demo-search.md](sources/demo-search.md)

![Search](images/demo-search.png)

**And also:**
- **Auto-update**: run `leaf --update` to check and install the latest version from GitHub with SHA256 verification
- **Stdin support**: pipe Markdown directly with `echo '# Hello' | leaf` (up to 8 MB)
- **Status bar**: displays filename, watch indicator, search status, and scroll percentage
- **Help modal**: press `?` to view all keyboard shortcuts organized by category

## Markdown Rendering

### LaTeX / Math

**leaf** renders mathematical formulas directly in the terminal — a rare feature for a terminal viewer. Inline math with `$...$` and display math with `$$...$$` are converted to Unicode symbols.

Supported: fractions, superscripts, subscripts, Greek letters, roots, and more. Code blocks with `latex` or `tex` language are also rendered.

Source: [demo-latex-render.md](sources/demo-latex-render.md)

![LaTeX Rendering](images/demo-latex-render.png)

### Mermaid Diagrams

**leaf** renders Mermaid diagrams as visual ASCII art directly in the terminal. Flowcharts, sequence diagrams, and pie charts are converted from their text definitions into box-drawing representations. Unsupported diagram types fall back to syntax-colored source display.

Source: [demo-mermaid-render.md](sources/demo-mermaid-render.md)

![Mermaid Diagrams](images/demo-mermaid-render.png)

### Code Blocks with Syntax Highlighting

Fenced code blocks are displayed in a decorative frame with language label and line numbers. Syntax highlighting supports 40+ languages (Rust, Python, JavaScript, TypeScript, Go, C/C++, Java, and more) via the syntect library.

Code blocks adapt to the terminal width with intelligent wrapping.

Source: [demo-code-syntax.md](sources/demo-code-syntax.md)

![Code Blocks](images/demo-code-syntax.png)

### Tables

Markdown tables are rendered with Unicode box-drawing borders and column alignment (left, center, right). Cells support inline code and LaTeX math. Tables adapt to the available terminal width.

Source: [demo-table-render.md](sources/demo-table-render.md)

![Tables](images/demo-table-render.png)

### Theme Picker

Choose from 4 built-in themes with live preview as you navigate. Each theme applies to the UI, Markdown elements, and syntax highlighting simultaneously.

- Press `Shift+T` to open the theme picker
- Available themes: Arctic, Forest, Ocean Dark, Solarized Dark
- Press `Esc` to cancel and restore the original theme

Source: [demo-theme-picker.md](sources/demo-theme-picker.md)

![Theme Picker](images/demo-theme-picker.png)

### Navigation

Navigate documents with keyboard shortcuts or mouse. The scrollbar on the right supports click and drag for quick positioning.

- `j`/`k` or arrow keys: scroll line by line
- `d`/`u` or PageDown/PageUp: scroll by page
- `g`/`Shift+G` or Home/End: jump to top/bottom
- Mouse wheel: scroll 3 lines per tick
- Scrollbar: click and drag for fast navigation

Source: [demo-navigation.md](sources/demo-navigation.md)

![Navigation](images/demo-navigation.png)

**And also:**
- **Frontmatter**: YAML metadata rendered as a full-width table (horizontal for few keys, vertical for 5+)
- **Blockquotes**: multi-level nesting with vertical markers, italic text, can contain lists and code
- **Nested lists**: Unicode bullets per level (bullet, circle, triangle), distinct colors, ordered lists with numbering
- **Headings**: H1 underlined with double lines, H2 with single lines, H3 bold, H4+ with distinct colors
- **Inline formatting**: **bold**, *italic*, ~~strikethrough~~, `inline code`, links with icon
