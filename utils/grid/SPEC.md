# grid — a fuzzy finder for tabular data

**Status:** design document / implementation prompt. No code yet.
**Language:** Rust. **Install target:** `~/.local/bin/grid` via `setup/install/install-grid.sh`.

---

## 1. What this is

`grid` is to tables what `fzf` is to lines: a terminal picker that reads rows on
stdin, renders them as an aligned, scrollable table with a pinned header,
narrows them with fuzzy search, lets arbitrary keys run arbitrary commands
against the focused row, and prints a selection to stdout.

```
ps aux | grid --header -d ws --nth 11.. \
  --bind 'ctrl-k:execute-silent(kill {2})+reload(ps aux)' \
  --bind 'enter:accept' --field 2
```

The gap it fills: `fzf` has the interaction model but no concept of columns
(`column -t` fakes it, and breaks on live streams). VisiData has real columns
and programmable keys but is an application, not a picker. `csvlens`/`tabiew`
are viewers with no action layer. `grid` is the intersection: **columns +
bindings + stdin + clean stdout**.

### Non-goals

- Not a spreadsheet. No cell editing, no formulas, no persistence.
- Not a query engine. No SQL (`tabiew` does that; pipe through `qsv`/`duckdb` first).
- Not a pager. If you only want to *look* at a CSV, use `csvlens`.
- No plugin runtime, no embedded scripting language. Actions shell out.

---

## 2. Architecture

Four subsystems, three threads.

```
stdin ──▶ [reader thread] ──▶ Store (arena + offsets)
                                 │
                                 ├──▶ [nucleo worker pool] ──▶ matched index
                                 │
main thread: event loop ◀── /dev/tty (keys) ──┐
             │                                │
             └──▶ ratatui render ──▶ /dev/tty (alt screen)

on accept: selection ──▶ real stdout
```

### 2.1 The stdout/tty split (get this right first)

stdin is **data**, so the TUI cannot read keys from it, and stdout is the
**result channel**, so the TUI cannot draw to it. Therefore:

- Open `/dev/tty` read+write at startup. All input events and all rendering go
  through that handle.
- `stdout` stays untouched until exit, when the selection is written to it.
- If `/dev/tty` cannot be opened (cron, CI, nested pipe), fail with a clear
  error unless `--filter <query>` was passed, which is the headless mode:
  filter, print, exit, never touch the terminal.

Getting this backwards is the single most common way tools like this become
unusable inside `$(...)`.

### 2.2 Store: arena + offsets

Rows arrive forever and must not each cost N allocations.

```rust
struct Store {
    buf: String,              // append-only arena of raw input
    fields: Vec<u32>,         // flat field boundaries into buf
    rows: Vec<RowMeta>,       // { field_start: u32, field_count: u16 }
}
```

A cell is `&buf[fields[i]..fields[i+1]]`. One allocation amortised per read
chunk, zero per cell. Targets: 1M rows resident under ~200 MB for typical
80-col-wide input; filter latency under 100 ms at that size.

Rows are never removed; `reload` swaps in a fresh `Store` behind an `ArcSwap`
so the render thread never blocks on the reader.

### 2.3 Parsing

Delimiter modes, selected by `-d/--delimiter`:

| Mode | Value | Behaviour |
|---|---|---|
| whitespace | `ws` | runs of space/tab collapse; the default when input is not CSV-ish |
| tab | `tab` / `\t` | strict single-tab split, the fast path |
| char | any single char | strict split |
| csv | `csv` | RFC 4180 via the `csv` crate — quotes, embedded newlines, escapes |
| regex | `re:<pat>` | last resort, slow path |

Auto-detection when `-d` is absent: sniff the first 64 KiB. Tab present on
every line → `tab`. Balanced quotes + commas → `csv`. Otherwise `ws`.

Ragged rows are legal. Missing trailing fields render empty; extra fields
beyond the header count are kept and addressable by index. `--strict` turns
raggedness into an error.

ANSI escapes in input are parsed into style spans and stripped from the match
haystack, so colours survive but don't corrupt column widths or search
(`--ansi`; off by default because it costs a scan).

### 2.4 Matching

Use `nucleo` (the matcher extracted from Helix — same one `fzf`-alikes reach
for). Its `Nucleo<T>` driver already does streaming injection and background
worker threads, which is exactly the shape here: the reader thread pushes into
the `Injector` as rows land, the UI polls a tick for updated results.

The haystack per row is the concatenation of the columns named by `--nth`
(default: all columns), joined by `\x1f` so matches can't bleed across a column
boundary. `--with-nth` independently controls what is *displayed*, which is how
you carry a hidden ID column and print it on accept without ever showing it.

Match highlights must be mapped back from haystack offsets to (column, byte
range) so the correct characters bold inside the correct cell.

Search modes, cycled by a binding: `fuzzy` (default), `substring` (`'` prefix
like fzf), `regex` (`--regex`), and `column-scoped` (`name:query` when a header
name is matched — this is the one real ergonomic win over fzf).

### 2.5 Layout

Column widths are the interesting problem: one 4000-char cell must not blow up
the table, and widths must not jitter as a stream arrives.

- Width for column *i* = clamp(p95 of observed display widths, `min_width`,
  `max_width`). p95 rather than max, tracked with a small reservoir/sketch per
  column so it's O(1) per row.
- Recompute on a debounce (every 150 ms, or immediately when a width would
  *grow* past the current allocation) — never per row.
- Once the user starts interacting, freeze widths unless they grow by >20%;
  jittering columns under the cursor are the main reason streaming table UIs
  feel broken.
- Display width is grapheme/east-asian-aware (`unicode-width`), not byte length.
- Overflow: ellipsis by default, `--wrap` for wrapped cells, per-column
  alignment auto-detected (numeric → right).
- Horizontal scroll when total width exceeds the viewport; `--freeze N` pins
  the first N columns during horizontal scroll.

Render with `ratatui`'s `Table` + `crossterm` backend.

---

## 3. CLI surface

Deliberately fzf-shaped. Anyone who knows fzf should be productive without docs.

### Input & structure
```
-d, --delimiter <SPEC>     ws | tab | csv | <char> | re:<pat>   [auto]
    --header               treat first row as the header (pinned, unfilterable)
    --header-lines <N>     pin N leading rows
    --names <a,b,c>        supply header names for headerless input
    --nth <RANGES>         columns to search           [all]
    --with-nth <RANGES>    columns to display          [all]
    --freeze <N>           pin first N columns during horizontal scroll
    --strict               error on ragged rows
    --ansi                 interpret ANSI colour in input
```
Ranges use fzf syntax: `2`, `2..5`, `..3`, `4..`, `-1`, and names: `pid,rss`.

### Selection & output
```
-m, --multi                multi-select
    --field <RANGE>        print only these columns on accept   [whole row]
    --print0               NUL-separate output records
    --print-query          print the query as the first line
    --filter <QUERY>       headless: filter, print, exit (no tty)
    --with-header          include the header row in output
```
Exit codes mirror fzf: `0` accepted, `1` no match, `2` error, `130` aborted.

### Interaction
```
    --bind <KEY:ACTIONS>   repeatable; see §4
    --preview <CMD>        preview command, placeholders expanded
    --preview-window <SPEC>  right:50% | down:30% | hidden
    --sort <COL[:asc|desc]>  initial sort
    --height <N|N%>        inline mode instead of alt-screen
    --query <Q>            initial query
    --no-mouse
```

### Presentation
```
    --fmt <COL:MOD[,MOD]>  column modifiers; see §5
    --color <COL:RULE>     conditional colouring; see §5
    --theme <NAME>
    --min-width / --max-width <COL:N>
```

### Config
```
    --profile <NAME>       load [profile.NAME] from config.toml
    --config <PATH>
```

---

## 4. Bindings and actions

`--bind 'key:action(arg)+action+action'`, same grammar as fzf so muscle memory
transfers. Keys: `enter`, `ctrl-x`, `alt-x`, `f1`..`f12`, `tab`, `btab`,
`ctrl-alt-x`, plus `click`/`double-click` on a column header when mouse is on.

### Action registry

| Action | Effect |
|---|---|
| `accept` | print selection, exit 0 |
| `abort` | exit 130 |
| `execute(cmd)` | suspend TUI, run in a shell, restore |
| `execute-silent(cmd)` | run detached, no screen disruption |
| `become(cmd)` | `exec` into cmd, replacing `grid` |
| `reload(cmd)` | re-run cmd, swap the store, preserve query/cursor by key |
| `refresh` | re-read from the original source if seekable |
| `toggle`, `toggle-all`, `select-all`, `deselect-all` | multi-select |
| `sort-by-column`, `sort-reverse` | sort on the focused column |
| `toggle-column`, `only-column` | show/hide the focused column |
| `yank(SPEC)` | copy expansion of SPEC to the system clipboard |
| `put(TEXT)` | insert text into the query |
| `toggle-preview`, `preview-up`, `preview-down` | |
| `up`, `down`, `page-up`, `page-down`, `first`, `last`, `scroll-left`, `scroll-right` | |
| `change-search-mode(fuzzy\|substring\|regex)` | |

### Placeholder expansion

| Token | Expands to |
|---|---|
| `{}` | whole focused row, raw |
| `{N}` | field N (1-indexed), `{-1}` last |
| `{a..b}` | field range, space-joined |
| `{name}` | field by header name |
| `{+}` `{+N}` | same, but over all selected rows |
| `{q}` | current query |
| `{i}` | row index in the *original* input |
| `{f}` | a temp file containing the selection (for large multi-selects) |

**Every expansion is shell-quoted by default.** `{raw:N}` opts out and is the
only way to inject unquoted text — filenames with spaces and quotes in `ps`
output are the normal case, not the edge case. Expansions never go through a
second round of expansion.

### Config file

`~/.config/grid/config.toml`, merged under CLI flags:

```toml
[bindings]
"ctrl-y" = "yank({})"
"ctrl-s" = "sort-by-column"

[profile.ps]
delimiter = "ws"
header = true
nth = "11.."
bindings = { "ctrl-k" = "execute-silent(kill {2})+reload(ps aux)" }
```

`grid --profile ps` then replaces a long shell alias. Profiles are the intended
distribution mechanism for per-tool recipes.

---

## 5. Column modifiers

This is the "modifiers for text" layer — pure display transforms applied after
parsing and before layout. They never affect the match haystack or the output;
`--field 3` prints the raw cell, not the formatted one. That separation must
hold or the tool becomes unsafe to script against.

A fixed registry, not an expression language:

| Modifier | Example in → out |
|---|---|
| `bytes` | `1073741824` → `1.0 GiB` |
| `num` | `1234567` → `1,234,567` |
| `pct[:dp]` | `0.4271` → `42.7%` |
| `time` | epoch or ISO → `3 days ago` |
| `dur` | `9312` (s) → `2h 35m` |
| `trunc:N[:head\|mid\|tail]` | ellipsised to N cells |
| `path[:N]` | `/a/b/c/d/e.txt` → `…/d/e.txt` |
| `pad:N`, `upper`, `lower`, `trim` | |
| `strip-ansi` | |
| `re:<pat>/<repl>` | one regex substitution |

Composable left to right: `--fmt 5:bytes,pad:10`.

Colouring is separate and predicate-based:

```
--color 'rss:gt(1e9):red'
--color 'status:eq(FAILED):bold-red'
--color 'name:re(^\.):dim'
```

Predicates: `gt`, `lt`, `eq`, `ne`, `re`, `empty`. Deliberately not Turing
complete — anything more belongs in an `awk` upstream of the pipe.

---

## 6. Testing

No CI in this repo, so tests have to be worth running by hand.

- **Parsing:** table-driven over ragged rows, quoted CSV with embedded
  newlines, CRLF, invalid UTF-8 (must not panic — replace and continue), NUL
  bytes, 1-byte and 100 MB inputs.
- **Layout:** golden snapshots against `ratatui`'s `TestBackend`. Cover CJK
  width, combining marks, emoji ZWJ sequences, the p95 clamp under one giant
  outlier cell, and width stability across a simulated stream.
- **Placeholder expansion:** fuzz the quoting. A field containing
  `"; rm -rf /; #` must survive expansion inert. This is the security-relevant
  surface — treat a quoting escape as a P0 bug.
- **End-to-end:** drive a pty (`portable-pty`) with scripted keys, assert on
  stdout and exit code. At minimum: accept, abort, multi-select, reload,
  headless `--filter`.
- **Bench:** `criterion` on parse throughput and filter latency at 10k / 100k /
  1M rows, so the perf targets in §2.2 stay honest.

`cargo test` and `cargo clippy -- -D warnings` are the gate.

---

## 7. Prioritised task list

Each milestone is independently useful — stop at any point and still have a
working tool.

### M0 — Skeleton (foundation, blocks everything)
1. `utils/grid/` crate; `clap` arg parsing for the §3 surface (parse only, most flags unimplemented).
2. `/dev/tty` acquisition, alt-screen enter/exit, panic hook + SIGINT/SIGTERM handler that always restores the terminal. **Do this before any rendering** — a tool that leaves a wrecked terminal on panic never gets a second use.
3. Read stdin to `Store`; whitespace + tab split; `RowMeta` arena.
4. `setup/install/install-grid.sh` following the existing installer pattern.

### M1 — Viewer (first usable release)
5. `ratatui` table render: pinned header, cursor row, vertical scroll.
6. Width computation with p95 clamp, `unicode-width` measurement, ellipsis overflow, numeric right-align.
7. Horizontal scroll + `--freeze`.
8. `--header`, `--header-lines`, `--names`, explicit `-d`.
9. Basic navigation keys (arrows, `jk`, page, home/end, `q`).

*Checkpoint: `csvlens` parity.*

### M2 — Finding (the reason the tool exists)
10. Wire `nucleo`; query input line; incremental filtering.
11. `--nth` / `--with-nth` range and name parsing; haystack construction with `\x1f` separators.
12. Match highlight mapping back to (column, byte range).
13. `--query`, and `--filter` headless mode with no tty.
14. `accept` / `abort` + exit codes + `--field`, `--print0`, `--multi`.

*Checkpoint: usable in `$(...)`. This is the minimum shippable tool.*

### M3 — Acting (the differentiator)
15. Binding grammar parser: `key:action+action`, `--bind` repeatable.
16. Placeholder expansion with shell quoting; `{raw:}` opt-out; the fuzz test from §6.
17. `execute`, `execute-silent`, `become` with correct terminal suspend/restore.
18. `reload` with cursor/query preservation.
19. `toggle`/`select-all`, `yank`, `sort-by-column`, `toggle-column`.

*Checkpoint: replaces the `column -t | fzf --bind` idiom entirely.*

### M4 — Streaming
20. Reader thread + `Injector`; render while input is still arriving.
21. Debounced width recompute; freeze-on-interaction heuristic.
22. Row-count / "still reading" indicator in the status line.
23. Backpressure and a `--max-rows` cap so `yes | grid` doesn't OOM the machine.

### M5 — Presentation
24. `--fmt` registry (§5) and the modifier parser.
25. `--color` predicates.
26. CSV mode via the `csv` crate; delimiter auto-detection.
27. `--ansi` input parsing.
28. Themes; `--height` inline mode.

### M6 — Preview
29. `--preview` with async command execution, `--preview-window` placement, scrolling, and correct cancellation when the cursor moves faster than the command returns.

### M7 — Config
30. `~/.config/grid/config.toml`, `[bindings]`, `[profile.*]`, merge precedence.
31. Ship profiles for `ps`, `docker ps`, `git log --format`, `brew list --versions`.

### M8 — Polish
32. `--regex` and `column:query` scoped search.
33. Mouse: click to focus, click header to sort, wheel scroll.
34. `README.md` with a recipes section; add `grid` to `Brewfile.optional` notes if it ever gets a formula.
35. Bench suite; verify the §2.2 targets.

### Deferred (explicitly out of scope for v1)
- Computed columns / expression language.
- Any SQL.
- Cell editing or writing back to the source.
- Multi-table joins.
- A non-shell plugin API.

---

## 8. Risks

| Risk | Mitigation |
|---|---|
| Quoting bug in placeholder expansion → arbitrary command execution from hostile input | Quote by default, fuzz it, make `{raw:}` visibly opt-in |
| Terminal left wrecked on panic | Restore guard installed in M0, before any drawing |
| Width jitter makes streaming mode feel broken | Debounce + freeze-on-interaction (M4.21) |
| Scope creep into VisiData territory | §1 non-goals and the deferred list are binding |
| `nucleo` API churn | Pin the version; the matcher boundary is small enough to swap |
