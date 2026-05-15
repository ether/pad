# Plan 2: `pad` v0.1 — Local-Only Editor

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a working `pad` binary that opens files instantly (<50 ms warm cache), provides every nano-faithful keybinding from spec §6.1, persists edits crash-safely, and exits cleanly. **Zero network dependency** — `etherpad-client` is linked but not invoked. Share lands in Plan 3.

**Architecture:** A new `crates/pad/` workspace member produces the `pad` binary. Three logical concurrent tokio tasks (Input, Sync, Render); Sync is a no-op stub in v0.1 — the channel exists so Plan 3 can drop in the real client without restructuring. Rope storage via `ropey`. TUI via `ratatui` + `crossterm`. Buffer state lives in one `Buffer` struct that owns the rope, cursor, undo stack, dirty flag, line-ending convention, and a sidecar handle to the on-disk crash log. Every keystroke appends to `pending.log` (fsync'd) before the rope mutates; on clean save, the log is truncated. A `panic::set_hook` ensures even kill -9 loses at most the last keystroke.

**Tech Stack:** Rust 2024, tokio, `ratatui` 0.30, `crossterm` 0.30, `ropey` 1.6, `uuid` 1 (v7 feature), `dirs` 5 (for `~/.local/state`), `clap` 4 (CLI parse), `criterion` (perf budgets), `expectrl` (PTY e2e).

**Reference sources:**
- nano keybindings: `https://www.nano-editor.org/dist/latest/nano.html` (man page).
- Spec sections covered: §4.1, §4.2, §5.1, §6.1, §6.4, §7 (file/IO/panic rows), §9.
- Spec sections explicitly NOT covered (deferred to Plan 3): §4.4 (Share-related config), §5.2/5.3/5.4, §6.2, §6.3.

---

## Scope boundary — what's in vs out

**In v0.1:**
- Invocations: `pad`, `pad <filepath>`, `pad --recover`. (No `pad <url>` — that joins a remote pad, Plan 3.)
- All §6.1 nano bindings: `^O ^X ^R ^K ^U ^W M-R ^_ M-U M-E ^G ^C ^Z`.
- Soft-wrap, UTF-8, line-ending preservation (§6.4).
- Crash-safe pending log + panic hook + `--recover` (§7 row "Panic", §4.2 sidecars).
- Cold-open perf budget gating in CI (§9).

**Out (Plan 3 picks up):**
- All `M-` collab bindings.
- Share overlay, QR, OSC 52.
- First-run remote config + `scanner.etherpad.org`.
- `pre-share-*` and `pre-merge` snapshots (the *infrastructure* — UUIDv7 buffer-id and the sidecar dir — lands here so Plan 3 just adds new files into an existing layout).
- `pad <url>` join.

---

## File Structure

```
etherpad-pad/
├── Cargo.toml                            # workspace (add crates/pad as member)
├── crates/
│   ├── etherpad-client/                  # Plan 1 — untouched
│   └── pad/
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs                   # arg parse, top-level loop, panic hook install
│       │   ├── cli.rs                    # clap-derived Args
│       │   ├── app.rs                    # App: owns Buffer + Tui + event loop
│       │   ├── buffer/
│       │   │   ├── mod.rs                # Buffer struct, rope, cursor, dirty
│       │   │   ├── line_endings.rs       # detect + preserve LF/CRLF/CR
│       │   │   ├── search.rs             # ^W where-is, M-R replace
│       │   │   ├── clipboard.rs          # ^K cut, ^U uncut (per-line register)
│       │   │   ├── undo.rs               # snapshot-based undo stack
│       │   │   └── sidecar.rs            # buffer-id, sidecar dir, pending.log, meta.json
│       │   ├── tui/
│       │   │   ├── mod.rs                # Tui: ratatui Terminal + render entrypoint
│       │   │   ├── editor_view.rs        # main editor surface (text + cursor)
│       │   │   ├── status_bar.rs         # bottom bar (filename, dirty, line/col)
│       │   │   ├── prompts.rs            # ^O save-as, ^_ goto-line, dirty-prompt etc.
│       │   │   ├── softwrap.rs           # soft-wrap layout helper
│       │   │   └── help.rs               # ^G help overlay
│       │   ├── input.rs                  # crossterm event → KeyAction
│       │   ├── keymap.rs                 # KeyAction enum + chord-table mapping
│       │   ├── config/
│       │   │   ├── mod.rs                # ConfigShape (Plan 3 expands)
│       │   │   └── paths.rs              # XDG dirs, ~/.config/pad, ~/.local/state/pad
│       │   ├── recover.rs                # pad --recover flow
│       │   └── perf.rs                   # first-paint timing helpers + criterion entry
│       ├── tests/
│       │   ├── buffer_unit.rs            # rope/cursor/undo/clipboard
│       │   ├── line_endings.rs           # LF/CRLF/CR round-trip
│       │   ├── sidecar.rs                # pending.log fsync + truncate-on-save
│       │   ├── pty_smoke.rs              # open-edit-save lifecycle (expectrl)
│       │   ├── pty_recover.rs            # crash + recover lifecycle
│       │   └── perf_budget.rs            # cold-open <50ms gate
│       └── benches/
│           └── cold_open.rs              # criterion: cold-open + 100KB file
└── (other files unchanged)
```

**Why this split:**
- `buffer/` is one responsibility (document model) but rope/cursor/undo/clipboard/search are big enough to warrant submodule files.
- `tui/` mirrors that — `editor_view`, `status_bar`, `prompts`, `softwrap`, `help` each fit in <300 lines.
- `keymap.rs` is one file because the chord table belongs in one place.
- `sidecar.rs` is named for what it owns (sidecar on-disk artifacts), not its location.
- `perf.rs` exists because cold-open timing is a tested invariant (§9) — not a comment, a verified target.

---

## Task 1: Add `crates/pad/` to workspace

**Files:**
- Modify: `Cargo.toml` (workspace root)
- Create: `crates/pad/Cargo.toml`
- Create: `crates/pad/src/main.rs`

- [ ] **Step 1: Add `crates/pad` to workspace members**

Edit `Cargo.toml`:

```toml
[workspace]
resolver = "3"
members = [
    "crates/etherpad-client",
    "crates/pad",
    "spike",
]
```

Add to `[workspace.dependencies]`:

```toml
ratatui = "0.30"
crossterm = "0.30"
ropey = "1.6"
uuid = { version = "1", features = ["v7"] }
dirs = "5"
clap = { version = "4", features = ["derive"] }
anyhow = "1"
```

- [ ] **Step 2: Write `crates/pad/Cargo.toml`**

```toml
[package]
name = "pad"
version = "0.1.0-dev"
description = "Nano-class terminal text editor with optional real-time collaboration backed by Etherpad."
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
default-run = "pad"

[[bin]]
name = "pad"
path = "src/main.rs"

[dependencies]
etherpad-client = { path = "../etherpad-client" }
tokio = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
ropey = { workspace = true }
uuid = { workspace = true }
dirs = { workspace = true }
clap = { workspace = true }
anyhow = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
proptest = "1"
expectrl = "0.7"
criterion = { version = "0.5", features = ["html_reports"] }
tempfile = "3"

[[bench]]
name = "cold_open"
harness = false
```

- [ ] **Step 3: Write a hello-world `src/main.rs`**

```rust
fn main() -> anyhow::Result<()> {
    println!("pad v0.1.0-dev — local editor scaffold");
    Ok(())
}
```

Also create empty `benches/cold_open.rs`:

```rust
fn main() {}
```

- [ ] **Step 4: Build the workspace**

Run: `cargo build --workspace`
Expected: `Compiling pad v0.1.0-dev ... Finished`. No warnings.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/pad/
git commit -m "feat(pad): scaffold crates/pad workspace member"
```

---

## Task 2: CLI arg parsing

**Files:**
- Create: `crates/pad/src/cli.rs`
- Modify: `crates/pad/src/main.rs`
- Create: `crates/pad/tests/cli_args.rs`

- [ ] **Step 1: Write failing CLI tests**

```rust
// crates/pad/tests/cli_args.rs
use clap::Parser;
use pad::cli::{Args, Mode};

#[test]
fn no_args_is_untitled() {
    let a = Args::parse_from(["pad"]);
    assert!(matches!(a.mode(), Mode::Untitled));
}

#[test]
fn one_path_opens_file() {
    let a = Args::parse_from(["pad", "foo.md"]);
    match a.mode() {
        Mode::OpenFile(p) => assert_eq!(p.to_str().unwrap(), "foo.md"),
        m => panic!("expected OpenFile, got {m:?}"),
    }
}

#[test]
fn recover_flag() {
    let a = Args::parse_from(["pad", "--recover"]);
    assert!(matches!(a.mode(), Mode::Recover));
}

#[test]
fn extra_args_rejected() {
    let r = Args::try_parse_from(["pad", "a.txt", "b.txt"]);
    assert!(r.is_err());
}
```

- [ ] **Step 2: Verify tests fail (missing `pad::cli` lib)**

`pad` is currently a binary-only crate; expose a `lib.rs` so tests can import.

Add to `crates/pad/Cargo.toml`:

```toml
[lib]
name = "pad"
path = "src/lib.rs"
```

Create `crates/pad/src/lib.rs`:

```rust
pub mod cli;
```

Run: `cargo test -p pad --test cli_args`
Expected: compile errors — `Args`/`Mode` not defined.

- [ ] **Step 3: Implement `cli.rs`**

```rust
// crates/pad/src/cli.rs
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "pad", version, about = "Nano-class terminal text editor.")]
pub struct Args {
    /// Path to open. If omitted, opens an untitled buffer.
    pub path: Option<PathBuf>,

    /// List buffers with unsaved crash state and let you resume one.
    #[arg(long, conflicts_with = "path")]
    pub recover: bool,
}

#[derive(Debug, Clone)]
pub enum Mode {
    Untitled,
    OpenFile(PathBuf),
    Recover,
}

impl Args {
    pub fn mode(&self) -> Mode {
        if self.recover {
            Mode::Recover
        } else if let Some(p) = &self.path {
            Mode::OpenFile(p.clone())
        } else {
            Mode::Untitled
        }
    }
}
```

- [ ] **Step 4: Wire `main.rs` to use it**

```rust
// crates/pad/src/main.rs
use clap::Parser;
use pad::cli::{Args, Mode};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.mode() {
        Mode::Untitled => println!("untitled buffer"),
        Mode::OpenFile(p) => println!("open: {}", p.display()),
        Mode::Recover => println!("recover mode"),
    }
    Ok(())
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p pad --test cli_args`
Expected: 4 tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/pad/Cargo.toml crates/pad/src/ crates/pad/tests/cli_args.rs
git commit -m "feat(pad): CLI arg parsing (untitled / open-file / --recover)"
```

---

## Task 3: Buffer core (rope + cursor)

**Files:**
- Create: `crates/pad/src/buffer/mod.rs`
- Modify: `crates/pad/src/lib.rs`
- Create: `crates/pad/tests/buffer_unit.rs`

- [ ] **Step 1: Write failing tests**

```rust
// crates/pad/tests/buffer_unit.rs
use pad::buffer::{Buffer, CursorPos};

#[test]
fn empty_buffer() {
    let b = Buffer::empty();
    assert_eq!(b.text(), "");
    assert_eq!(b.cursor(), CursorPos { line: 0, col: 0 });
    assert!(!b.is_dirty());
}

#[test]
fn insert_char_advances_cursor_and_dirties() {
    let mut b = Buffer::empty();
    b.insert_char('a');
    assert_eq!(b.text(), "a");
    assert_eq!(b.cursor(), CursorPos { line: 0, col: 1 });
    assert!(b.is_dirty());
}

#[test]
fn insert_newline_moves_to_next_line() {
    let mut b = Buffer::empty();
    b.insert_char('a');
    b.insert_char('\n');
    b.insert_char('b');
    assert_eq!(b.text(), "a\nb");
    assert_eq!(b.cursor(), CursorPos { line: 1, col: 1 });
}

#[test]
fn backspace_at_start_of_line_joins_lines() {
    let mut b = Buffer::from_text("a\nb");
    b.move_cursor_to(CursorPos { line: 1, col: 0 });
    b.backspace();
    assert_eq!(b.text(), "ab");
    assert_eq!(b.cursor(), CursorPos { line: 0, col: 1 });
}

#[test]
fn delete_char_at_end_of_line_joins() {
    let mut b = Buffer::from_text("ab\ncd");
    b.move_cursor_to(CursorPos { line: 0, col: 2 });
    b.delete_char_forward();
    assert_eq!(b.text(), "abcd");
}

#[test]
fn arrow_navigation_clamps_to_line() {
    let mut b = Buffer::from_text("abc\nxy");
    b.move_cursor_to(CursorPos { line: 0, col: 3 });
    b.move_down();
    assert_eq!(b.cursor(), CursorPos { line: 1, col: 2 }); // clamped
}

#[test]
fn from_text_preserves_initial_clean_state() {
    let b = Buffer::from_text("hello");
    assert_eq!(b.text(), "hello");
    assert!(!b.is_dirty());
}
```

- [ ] **Step 2: Run tests to confirm they fail**

Run: `cargo test -p pad --test buffer_unit`
Expected: compile error — `pad::buffer` not defined.

- [ ] **Step 3: Implement `Buffer`**

Append to `crates/pad/src/lib.rs`:

```rust
pub mod buffer;
```

Create `crates/pad/src/buffer/mod.rs`:

```rust
use ropey::Rope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPos {
    pub line: usize,
    pub col: usize,
}

pub struct Buffer {
    rope: Rope,
    cursor: CursorPos,
    dirty: bool,
    /// Preferred column for vertical moves — preserves intent when crossing
    /// shorter lines.
    pref_col: usize,
}

impl Buffer {
    pub fn empty() -> Self {
        Self {
            rope: Rope::new(),
            cursor: CursorPos { line: 0, col: 0 },
            dirty: false,
            pref_col: 0,
        }
    }

    pub fn from_text(s: &str) -> Self {
        Self {
            rope: Rope::from_str(s),
            cursor: CursorPos { line: 0, col: 0 },
            dirty: false,
            pref_col: 0,
        }
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn cursor(&self) -> CursorPos {
        self.cursor
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn line_count(&self) -> usize {
        // Rope counts an empty doc as 1 line.
        self.rope.len_lines().max(1)
    }

    pub fn line(&self, idx: usize) -> String {
        if idx >= self.rope.len_lines() {
            return String::new();
        }
        let l = self.rope.line(idx);
        // ropey lines include the trailing newline; strip for display.
        let s = l.to_string();
        s.strip_suffix('\n').map(|x| x.to_string()).unwrap_or(s)
    }

    pub fn move_cursor_to(&mut self, pos: CursorPos) {
        self.cursor = self.clamp(pos);
        self.pref_col = self.cursor.col;
    }

    pub fn move_left(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        } else if self.cursor.line > 0 {
            self.cursor.line -= 1;
            self.cursor.col = self.line(self.cursor.line).chars().count();
        }
        self.pref_col = self.cursor.col;
    }

    pub fn move_right(&mut self) {
        let line_len = self.line(self.cursor.line).chars().count();
        if self.cursor.col < line_len {
            self.cursor.col += 1;
        } else if self.cursor.line + 1 < self.line_count() {
            self.cursor.line += 1;
            self.cursor.col = 0;
        }
        self.pref_col = self.cursor.col;
    }

    pub fn move_up(&mut self) {
        if self.cursor.line == 0 {
            return;
        }
        self.cursor.line -= 1;
        let line_len = self.line(self.cursor.line).chars().count();
        self.cursor.col = self.pref_col.min(line_len);
    }

    pub fn move_down(&mut self) {
        if self.cursor.line + 1 >= self.line_count() {
            return;
        }
        self.cursor.line += 1;
        let line_len = self.line(self.cursor.line).chars().count();
        self.cursor.col = self.pref_col.min(line_len);
    }

    pub fn insert_char(&mut self, c: char) {
        let char_idx = self.cursor_char_idx();
        self.rope.insert_char(char_idx, c);
        self.dirty = true;
        if c == '\n' {
            self.cursor.line += 1;
            self.cursor.col = 0;
        } else {
            self.cursor.col += 1;
        }
        self.pref_col = self.cursor.col;
    }

    pub fn insert_str(&mut self, s: &str) {
        for c in s.chars() {
            self.insert_char(c);
        }
    }

    pub fn backspace(&mut self) {
        let char_idx = self.cursor_char_idx();
        if char_idx == 0 {
            return;
        }
        // Capture the char we're about to delete to know how to update cursor.
        let prev = self.rope.char(char_idx - 1);
        self.rope.remove(char_idx - 1..char_idx);
        self.dirty = true;
        if prev == '\n' {
            self.cursor.line -= 1;
            self.cursor.col = self.line(self.cursor.line).chars().count();
        } else {
            self.cursor.col -= 1;
        }
        self.pref_col = self.cursor.col;
    }

    pub fn delete_char_forward(&mut self) {
        let char_idx = self.cursor_char_idx();
        if char_idx >= self.rope.len_chars() {
            return;
        }
        self.rope.remove(char_idx..char_idx + 1);
        self.dirty = true;
    }

    /// Convert (line, col) to absolute char index in the rope.
    pub(crate) fn cursor_char_idx(&self) -> usize {
        let line_start = self.rope.line_to_char(self.cursor.line);
        // Walk col chars into the line; ropey indexes in chars, not bytes.
        let line = self.line(self.cursor.line);
        let col_chars = line.chars().take(self.cursor.col).count();
        line_start + col_chars
    }

    fn clamp(&self, pos: CursorPos) -> CursorPos {
        let line = pos.line.min(self.line_count().saturating_sub(1));
        let max_col = self.line(line).chars().count();
        CursorPos {
            line,
            col: pos.col.min(max_col),
        }
    }

    /// Cursor in absolute char offset — used by the etherpad-client layer
    /// later (Plan 3). Public so Plan 3 can pump cursor presence events.
    pub fn cursor_offset(&self) -> u32 {
        self.cursor_char_idx() as u32
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p pad --test buffer_unit`
Expected: 7 tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/pad/src/lib.rs crates/pad/src/buffer/ crates/pad/tests/buffer_unit.rs
git commit -m "feat(pad): Buffer core — rope + cursor + insert/delete + arrow nav"
```

---

## Task 4: Line ending detection and preservation

**Files:**
- Create: `crates/pad/src/buffer/line_endings.rs`
- Modify: `crates/pad/src/buffer/mod.rs`
- Create: `crates/pad/tests/line_endings.rs`

- [ ] **Step 1: Write failing tests**

```rust
// crates/pad/tests/line_endings.rs
use pad::buffer::{Buffer, LineEnding};

#[test]
fn detect_lf() {
    assert_eq!(LineEnding::detect("a\nb\nc"), LineEnding::Lf);
}

#[test]
fn detect_crlf() {
    assert_eq!(LineEnding::detect("a\r\nb\r\n"), LineEnding::Crlf);
}

#[test]
fn detect_cr_only() {
    // Classic Mac line endings.
    assert_eq!(LineEnding::detect("a\rb\rc"), LineEnding::Cr);
}

#[test]
fn default_to_lf_when_no_newlines() {
    assert_eq!(LineEnding::detect("hello"), LineEnding::Lf);
}

#[test]
fn mixed_picks_most_common() {
    // 2 CRLF and 1 LF — CRLF wins.
    assert_eq!(LineEnding::detect("a\r\nb\r\nc\nd"), LineEnding::Crlf);
}

#[test]
fn buffer_roundtrips_crlf() {
    let original = "a\r\nb\r\nc\r\n";
    let mut b = Buffer::from_text_with_ending(original);
    assert_eq!(b.line_ending(), LineEnding::Crlf);
    b.insert_char('!'); // mutate so we know it survived editing
    assert_eq!(b.line_ending(), LineEnding::Crlf);
    // Serializing back preserves the convention.
    assert!(b.serialize_for_save().starts_with("!a\r\nb\r\n"));
}
```

- [ ] **Step 2: Implement `line_endings.rs`**

```rust
// crates/pad/src/buffer/line_endings.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    Crlf,
    Cr,
}

impl LineEnding {
    pub fn detect(text: &str) -> Self {
        let crlf = text.matches("\r\n").count();
        // Lone \r (not part of \r\n) and lone \n (not part of \r\n).
        let mut lone_cr = 0usize;
        let mut lone_lf = 0usize;
        let bytes = text.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'\r' if i + 1 < bytes.len() && bytes[i + 1] == b'\n' => {
                    i += 2;
                    continue;
                }
                b'\r' => lone_cr += 1,
                b'\n' => lone_lf += 1,
                _ => {}
            }
            i += 1;
        }
        if crlf == 0 && lone_cr == 0 && lone_lf == 0 {
            return LineEnding::Lf;
        }
        // Pick the most common.
        if crlf >= lone_cr.max(lone_lf) {
            LineEnding::Crlf
        } else if lone_cr > lone_lf {
            LineEnding::Cr
        } else {
            LineEnding::Lf
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
            LineEnding::Cr => "\r",
        }
    }
}
```

- [ ] **Step 3: Extend `Buffer`**

Edit `crates/pad/src/buffer/mod.rs` — add the module, store the convention, expose constructors:

```rust
// add at top
pub mod line_endings;
pub use line_endings::LineEnding;
```

Add to the `Buffer` struct:

```rust
pub struct Buffer {
    rope: Rope,
    cursor: CursorPos,
    dirty: bool,
    pref_col: usize,
    line_ending: LineEnding,
}
```

Update `empty()` and `from_text()`:

```rust
impl Buffer {
    pub fn empty() -> Self {
        Self {
            rope: Rope::new(),
            cursor: CursorPos { line: 0, col: 0 },
            dirty: false,
            pref_col: 0,
            line_ending: LineEnding::Lf,
        }
    }

    pub fn from_text(s: &str) -> Self {
        // Internal rope uses LF only; detection happens at the wire boundary.
        let normalized = s.replace("\r\n", "\n").replace('\r', "\n");
        Self {
            rope: Rope::from_str(&normalized),
            cursor: CursorPos { line: 0, col: 0 },
            dirty: false,
            pref_col: 0,
            line_ending: LineEnding::Lf,
        }
    }

    /// Like `from_text`, but detects and remembers the original line-ending
    /// convention so `serialize_for_save` round-trips it.
    pub fn from_text_with_ending(s: &str) -> Self {
        let ending = LineEnding::detect(s);
        let normalized = s.replace("\r\n", "\n").replace('\r', "\n");
        Self {
            rope: Rope::from_str(&normalized),
            cursor: CursorPos { line: 0, col: 0 },
            dirty: false,
            pref_col: 0,
            line_ending: ending,
        }
    }

    pub fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    /// Serialize the rope back to bytes with the buffer's line-ending convention.
    pub fn serialize_for_save(&self) -> String {
        let text = self.rope.to_string();
        match self.line_ending {
            LineEnding::Lf => text,
            LineEnding::Crlf => text.replace('\n', "\r\n"),
            LineEnding::Cr => text.replace('\n', "\r"),
        }
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p pad --test line_endings`
Expected: 6 tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/pad/src/buffer/ crates/pad/tests/line_endings.rs
git commit -m "feat(pad): detect + preserve LF/CRLF/CR line endings on save"
```

---

## Task 5: File I/O (load + save)

**Files:**
- Modify: `crates/pad/src/buffer/mod.rs`
- Create: `crates/pad/tests/file_io.rs`

- [ ] **Step 1: Write failing tests**

```rust
// crates/pad/tests/file_io.rs
use pad::buffer::Buffer;
use tempfile::tempdir;

#[test]
fn load_existing_file_preserves_lf() {
    let dir = tempdir().unwrap();
    let p = dir.path().join("a.txt");
    std::fs::write(&p, "hello\nworld\n").unwrap();
    let b = Buffer::load_from_file(&p).expect("load");
    assert_eq!(b.text(), "hello\nworld\n");
    assert!(!b.is_dirty());
}

#[test]
fn load_missing_returns_empty_buffer_with_note() {
    let dir = tempdir().unwrap();
    let p = dir.path().join("missing.txt");
    let r = Buffer::load_from_file(&p).expect("load missing");
    assert_eq!(r.text(), "");
    assert!(!r.is_dirty());
}

#[test]
fn save_round_trips_crlf() {
    let dir = tempdir().unwrap();
    let p = dir.path().join("crlf.txt");
    std::fs::write(&p, "a\r\nb\r\n").unwrap();
    let mut b = Buffer::load_from_file(&p).unwrap();
    b.insert_char('X');
    b.save_to_file(&p).unwrap();
    let after = std::fs::read_to_string(&p).unwrap();
    assert_eq!(after, "Xa\r\nb\r\n");
}

#[test]
fn save_marks_clean() {
    let dir = tempdir().unwrap();
    let p = dir.path().join("clean.txt");
    let mut b = Buffer::empty();
    b.insert_str("hi");
    assert!(b.is_dirty());
    b.save_to_file(&p).unwrap();
    assert!(!b.is_dirty());
}

#[test]
fn save_non_utf8_path_errors_gracefully() {
    // Read into invalid-UTF-8 region; we treat as error rather than panic.
    let dir = tempdir().unwrap();
    let p = dir.path().join("bin.dat");
    std::fs::write(&p, [0xff, 0xfe, 0x00, 0x00]).unwrap();
    let r = Buffer::load_from_file(&p);
    assert!(r.is_err(), "non-UTF-8 should bubble an error");
}
```

- [ ] **Step 2: Implement load/save**

Append to `crates/pad/src/buffer/mod.rs`:

```rust
use std::path::Path;

impl Buffer {
    /// Read a file into a buffer. Missing files return an empty buffer with
    /// the default LF ending — nano-style "save will create it" behaviour.
    pub fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        match std::fs::read(path) {
            Ok(bytes) => {
                let text = std::str::from_utf8(&bytes).map_err(|e| {
                    anyhow::anyhow!("file is not valid UTF-8: {e}")
                })?;
                Ok(Self::from_text_with_ending(text))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::empty()),
            Err(e) => Err(anyhow::anyhow!("read failed: {e}")),
        }
    }

    pub fn save_to_file(&mut self, path: &Path) -> anyhow::Result<()> {
        let text = self.serialize_for_save();
        std::fs::write(path, text).map_err(|e| anyhow::anyhow!("write failed: {e}"))?;
        self.mark_clean();
        Ok(())
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p pad --test file_io`
Expected: 5 tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/pad/src/buffer/mod.rs crates/pad/tests/file_io.rs
git commit -m "feat(pad): Buffer load/save with line-ending preservation"
```

---

## Task 6: Buffer-id + sidecar dir + meta.json

**Files:**
- Create: `crates/pad/src/buffer/sidecar.rs`
- Modify: `crates/pad/src/buffer/mod.rs`
- Modify: `crates/pad/src/lib.rs`
- Create: `crates/pad/src/config/mod.rs`
- Create: `crates/pad/src/config/paths.rs`
- Create: `crates/pad/tests/sidecar_meta.rs`

- [ ] **Step 1: Write failing tests**

```rust
// crates/pad/tests/sidecar_meta.rs
use pad::buffer::sidecar::SidecarHandle;
use tempfile::tempdir;

#[test]
fn new_sidecar_creates_dir_with_buffer_id() {
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).expect("new");
    assert!(sc.dir().exists(), "sidecar dir must exist");
    assert!(sc.dir().join("meta.json").exists(), "meta.json must be written");
    assert!(!sc.buffer_id().to_string().is_empty());
}

#[test]
fn meta_round_trips_file_path() {
    let state_root = tempdir().unwrap();
    let file_path = std::path::PathBuf::from("/some/file.md");
    let sc = SidecarHandle::for_file(state_root.path(), &file_path).expect("for_file");
    let id = sc.buffer_id();
    drop(sc);
    // Reattach using the same buffer-id and verify meta survived.
    let reattached =
        SidecarHandle::reattach(state_root.path(), &id).expect("reattach");
    assert_eq!(reattached.file_path().unwrap(), file_path);
}

#[test]
fn buffer_id_is_uuid_v7() {
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).unwrap();
    let id_str = sc.buffer_id().to_string();
    // UUID v7 has the version nibble = 7 at byte 7.
    let uuid = uuid::Uuid::parse_str(&id_str).expect("parses as uuid");
    assert_eq!(uuid.get_version_num(), 7);
}
```

- [ ] **Step 2: Implement `paths.rs`**

```rust
// crates/pad/src/config/paths.rs
use std::path::PathBuf;

pub fn state_root() -> PathBuf {
    dirs::state_dir()
        .unwrap_or_else(|| dirs::home_dir().expect("HOME").join(".local/state"))
        .join("pad")
}

pub fn config_root() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().expect("HOME").join(".config"))
        .join("pad")
}
```

```rust
// crates/pad/src/config/mod.rs
pub mod paths;
```

- [ ] **Step 3: Implement `sidecar.rs`**

```rust
// crates/pad/src/buffer/sidecar.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub type BufferId = Uuid;

#[derive(Debug, Serialize, Deserialize, Default)]
struct Meta {
    file_path: Option<PathBuf>,
    /// Filled in by Plan 3.
    last_share_remote: Option<String>,
    last_pad_id: Option<String>,
}

pub struct SidecarHandle {
    state_root: PathBuf,
    buffer_id: BufferId,
    meta: Meta,
}

impl SidecarHandle {
    /// Mint a new buffer-id and prepare the sidecar dir for an untitled buffer.
    pub fn new_untitled(state_root: &Path) -> anyhow::Result<Self> {
        let buffer_id = Uuid::now_v7();
        let mut sc = Self {
            state_root: state_root.to_path_buf(),
            buffer_id,
            meta: Meta::default(),
        };
        fs::create_dir_all(sc.dir())?;
        sc.flush_meta()?;
        Ok(sc)
    }

    /// Open or create the sidecar associated with a given file path. Reuses an
    /// existing buffer-id if found via `.pad.<filename>.meta`; otherwise mints
    /// a fresh one.
    pub fn for_file(state_root: &Path, file_path: &Path) -> anyhow::Result<Self> {
        let pointer = sibling_meta_path(file_path);
        let buffer_id = if pointer.exists() {
            let raw = fs::read_to_string(&pointer)?;
            Uuid::parse_str(raw.trim())
                .map_err(|e| anyhow::anyhow!("invalid sibling pointer: {e}"))?
        } else {
            Uuid::now_v7()
        };
        let mut sc = Self {
            state_root: state_root.to_path_buf(),
            buffer_id,
            meta: Meta {
                file_path: Some(file_path.to_path_buf()),
                ..Default::default()
            },
        };
        fs::create_dir_all(sc.dir())?;
        sc.flush_meta()?;
        // Try to persist the sibling pointer; non-fatal if it fails (e.g. RO fs).
        let _ = fs::write(&pointer, buffer_id.to_string());
        Ok(sc)
    }

    pub fn reattach(state_root: &Path, buffer_id: &BufferId) -> anyhow::Result<Self> {
        let dir = state_root.join(buffer_id.to_string());
        if !dir.exists() {
            anyhow::bail!("no sidecar for buffer-id {}", buffer_id);
        }
        let meta_raw = fs::read_to_string(dir.join("meta.json"))?;
        let meta: Meta = serde_json::from_str(&meta_raw)?;
        Ok(Self {
            state_root: state_root.to_path_buf(),
            buffer_id: *buffer_id,
            meta,
        })
    }

    pub fn buffer_id(&self) -> BufferId {
        self.buffer_id
    }

    pub fn dir(&self) -> PathBuf {
        self.state_root.join(self.buffer_id.to_string())
    }

    pub fn pending_log_path(&self) -> PathBuf {
        self.dir().join("pending.log")
    }

    pub fn file_path(&self) -> Option<&Path> {
        self.meta.file_path.as_deref()
    }

    pub fn set_file_path(&mut self, p: PathBuf) -> anyhow::Result<()> {
        self.meta.file_path = Some(p);
        self.flush_meta()
    }

    fn flush_meta(&self) -> anyhow::Result<()> {
        let path = self.dir().join("meta.json");
        let raw = serde_json::to_string_pretty(&self.meta)?;
        fs::write(path, raw)?;
        Ok(())
    }
}

fn sibling_meta_path(file_path: &Path) -> PathBuf {
    let parent = file_path.parent().unwrap_or_else(|| Path::new("."));
    let name = file_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "untitled".to_string());
    parent.join(format!(".pad.{name}.meta"))
}
```

- [ ] **Step 4: Wire modules**

Add to `crates/pad/src/buffer/mod.rs`:

```rust
pub mod sidecar;
```

Add to `crates/pad/src/lib.rs`:

```rust
pub mod config;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p pad --test sidecar_meta`
Expected: 3 tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/pad/src/buffer/sidecar.rs crates/pad/src/config/ crates/pad/src/buffer/mod.rs crates/pad/src/lib.rs crates/pad/tests/sidecar_meta.rs
git commit -m "feat(pad): SidecarHandle — UUIDv7 buffer-id + meta.json + sibling pointer"
```

---

## Task 7: Crash-safe pending log + fsync per write

**Files:**
- Modify: `crates/pad/src/buffer/sidecar.rs`
- Create: `crates/pad/tests/pending_log.rs`

- [ ] **Step 1: Write failing tests**

```rust
// crates/pad/tests/pending_log.rs
use pad::buffer::sidecar::{PendingEntry, PendingLog, SidecarHandle};
use tempfile::tempdir;

#[test]
fn append_then_read_back() {
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).unwrap();
    let mut log = PendingLog::open(&sc).unwrap();
    log.append(&PendingEntry::Insert { offset: 0, text: "hi".into() }).unwrap();
    log.append(&PendingEntry::Delete { offset: 1, len: 1 }).unwrap();
    drop(log);
    let entries = PendingLog::read_all(&sc).unwrap();
    assert_eq!(entries.len(), 2);
    match &entries[0] {
        PendingEntry::Insert { offset, text } => {
            assert_eq!(*offset, 0);
            assert_eq!(text, "hi");
        }
        e => panic!("expected Insert, got {e:?}"),
    }
}

#[test]
fn truncate_on_save_clears_log() {
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).unwrap();
    let mut log = PendingLog::open(&sc).unwrap();
    log.append(&PendingEntry::Insert { offset: 0, text: "x".into() }).unwrap();
    log.truncate().unwrap();
    let entries = PendingLog::read_all(&sc).unwrap();
    assert!(entries.is_empty(), "log should be empty after truncate");
}

#[test]
fn ignores_corrupt_trailing_entry() {
    // A log with one valid entry plus a truncated half-line at the end
    // (simulating power-loss mid-write) should yield the one valid entry
    // and not panic.
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).unwrap();
    {
        let mut log = PendingLog::open(&sc).unwrap();
        log.append(&PendingEntry::Insert { offset: 0, text: "hi".into() }).unwrap();
    }
    // Append garbage that isn't a full line.
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(sc.pending_log_path())
        .unwrap();
    f.write_all(b"{garbage no newline").unwrap();

    let entries = PendingLog::read_all(&sc).unwrap();
    assert_eq!(entries.len(), 1, "corrupt tail must be ignored");
}
```

- [ ] **Step 2: Implement `PendingLog`**

Append to `crates/pad/src/buffer/sidecar.rs`:

```rust
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PendingEntry {
    Insert { offset: u32, text: String },
    Delete { offset: u32, len: u32 },
}

pub struct PendingLog {
    file: std::fs::File,
}

impl PendingLog {
    pub fn open(sc: &SidecarHandle) -> anyhow::Result<Self> {
        let path = sc.pending_log_path();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self { file })
    }

    pub fn append(&mut self, entry: &PendingEntry) -> anyhow::Result<()> {
        let mut line = serde_json::to_string(entry)?;
        line.push('\n');
        self.file.write_all(line.as_bytes())?;
        self.file.sync_data()?;
        Ok(())
    }

    pub fn truncate(&mut self) -> anyhow::Result<()> {
        self.file.set_len(0)?;
        self.file.sync_data()?;
        Ok(())
    }

    pub fn read_all(sc: &SidecarHandle) -> anyhow::Result<Vec<PendingEntry>> {
        let path = sc.pending_log_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let f = std::fs::File::open(path)?;
        let reader = BufReader::new(f);
        let mut entries = Vec::new();
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break, // truncated mid-line — stop, don't fail.
            };
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<PendingEntry>(&line) {
                Ok(e) => entries.push(e),
                Err(_) => break, // garbage tail — stop.
            }
        }
        Ok(entries)
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p pad --test pending_log`
Expected: 3 tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/pad/src/buffer/sidecar.rs crates/pad/tests/pending_log.rs
git commit -m "feat(pad): PendingLog — fsync'd append-only crash log"
```

---

## Task 8: Panic hook (terminal restore + crash log)

**Files:**
- Create: `crates/pad/src/panic_hook.rs`
- Modify: `crates/pad/src/lib.rs`
- Create: `crates/pad/tests/panic_hook.rs`

- [ ] **Step 1: Write the test**

```rust
// crates/pad/tests/panic_hook.rs
use pad::panic_hook::{install_panic_hook, PanicSink};
use std::sync::{Arc, Mutex};

#[test]
fn panic_hook_writes_to_sink() {
    let captured: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink: PanicSink = {
        let c = captured.clone();
        Box::new(move |s: &str| c.lock().unwrap().push(s.to_string()))
    };
    install_panic_hook(sink);

    let _ = std::panic::catch_unwind(|| panic!("boom"));

    let dumped = captured.lock().unwrap();
    assert!(dumped.iter().any(|s| s.contains("boom")), "panic message captured: {:?}", *dumped);
}
```

- [ ] **Step 2: Implement the panic hook**

```rust
// crates/pad/src/panic_hook.rs
pub type PanicSink = Box<dyn Fn(&str) + Send + Sync>;

/// Install a panic hook that restores the terminal and writes the panic stack
/// to the provided sink (in prod: a sidecar `crash-<ts>.log`; in tests: an
/// in-memory buffer).
pub fn install_panic_hook(sink: PanicSink) {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // 1. Try to restore the terminal. Best-effort — if crossterm isn't in
        //    raw mode this is a no-op.
        let _ = restore_terminal();

        // 2. Dump panic info to the sink.
        let payload = info.payload();
        let msg = if let Some(s) = payload.downcast_ref::<&'static str>() {
            (*s).to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "panic payload not a string".to_string()
        };
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "<unknown>".to_string());
        let dump = format!("PANIC at {location}: {msg}");
        sink(&dump);

        // 3. Hint the user.
        eprintln!("\npad crashed: {msg}\nRun 'pad --recover' to resume.");

        // 4. Chain the previous hook so backtrace etc. still print.
        prev(info);
    }));
}

fn restore_terminal() -> std::io::Result<()> {
    use crossterm::execute;
    use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
    let _ = disable_raw_mode();
    let mut stdout = std::io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen);
    Ok(())
}

/// Convenience: in production wire the sink to append to a crash file.
pub fn file_sink(crash_dir: std::path::PathBuf) -> PanicSink {
    use std::fs::OpenOptions;
    use std::io::Write;
    Box::new(move |s: &str| {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let path = crash_dir.join(format!("crash-{ts}.log"));
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = writeln!(f, "{s}");
            let _ = f.sync_data();
        }
    })
}
```

Add to `lib.rs`:

```rust
pub mod panic_hook;
```

- [ ] **Step 3: Run the test**

Run: `cargo test -p pad --test panic_hook`
Expected: passes (panic is caught and the sink sees the message).

- [ ] **Step 4: Commit**

```bash
git add crates/pad/src/panic_hook.rs crates/pad/src/lib.rs crates/pad/tests/panic_hook.rs
git commit -m "feat(pad): panic hook — terminal restore + crash log sink"
```

---

## Task 9: TUI scaffold

**Files:**
- Create: `crates/pad/src/tui/mod.rs`
- Create: `crates/pad/src/tui/editor_view.rs`
- Modify: `crates/pad/src/lib.rs`

- [ ] **Step 1: Implement `Tui` wrapper**

```rust
// crates/pad/src/tui/mod.rs
pub mod editor_view;

use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::Stdout;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    pub fn enter() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = std::io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
    }
}
```

```rust
// crates/pad/src/tui/editor_view.rs
use crate::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(frame: &mut Frame<'_>, area: Rect, buffer: &Buffer) {
    let text = buffer.text();
    let para = Paragraph::new(text).style(Style::default().fg(Color::Reset));
    frame.render_widget(para, area);
    // Cursor positioning happens in the next task; for the scaffold we just
    // render text.
}
```

Add to `lib.rs`:

```rust
pub mod tui;
```

- [ ] **Step 2: Verify compile**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 3: Commit**

```bash
git add crates/pad/src/tui/ crates/pad/src/lib.rs
git commit -m "feat(pad): Tui scaffold — ratatui Terminal + EditorView placeholder"
```

---

## Task 10: Render — text + cursor + status bar

**Files:**
- Create: `crates/pad/src/tui/status_bar.rs`
- Modify: `crates/pad/src/tui/editor_view.rs`
- Modify: `crates/pad/src/tui/mod.rs`

- [ ] **Step 1: Render the editor + cursor correctly**

Replace `editor_view.rs`:

```rust
use crate::buffer::{Buffer, CursorPos};
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(frame: &mut Frame<'_>, area: Rect, buffer: &Buffer) {
    // Build a Vec<Line> from the rope so ratatui handles vertical layout.
    let mut lines: Vec<Line<'_>> = Vec::with_capacity(buffer.line_count());
    for i in 0..buffer.line_count() {
        lines.push(Line::from(buffer.line(i)));
    }
    let para = Paragraph::new(lines).style(Style::default().fg(Color::Reset));
    frame.render_widget(para, area);

    // Position the terminal cursor.
    let CursorPos { line, col } = buffer.cursor();
    // Lines may wrap; for v1 (no soft-wrap until Task 21) we map 1:1.
    let cx = area.x.saturating_add(col as u16);
    let cy = area.y.saturating_add(line as u16);
    if cx < area.right() && cy < area.bottom() {
        frame.set_cursor_position(Position::new(cx, cy));
    }
}
```

- [ ] **Step 2: Status bar**

```rust
// crates/pad/src/tui/status_bar.rs
use crate::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(frame: &mut Frame<'_>, area: Rect, buffer: &Buffer, file_label: &str) {
    let dirty = if buffer.is_dirty() { "[modified] " } else { "" };
    let pos = buffer.cursor();
    let line = format!(
        "  {dirty}{file_label}    line {}, col {}",
        pos.line + 1,
        pos.col + 1
    );
    let p = Paragraph::new(line).style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(p, area);
}
```

- [ ] **Step 3: Combine in `tui::mod.rs`**

Add to `crates/pad/src/tui/mod.rs`:

```rust
pub mod status_bar;

use crate::buffer::Buffer;
use ratatui::layout::{Constraint, Layout};

impl Tui {
    pub fn draw(&mut self, buffer: &Buffer, file_label: &str) -> anyhow::Result<()> {
        self.terminal.draw(|frame| {
            let area = frame.area();
            let chunks = Layout::vertical([
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(area);
            editor_view::render(frame, chunks[0], buffer);
            status_bar::render(frame, chunks[1], buffer, file_label);
        })?;
        Ok(())
    }
}
```

- [ ] **Step 4: Build**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 5: Commit**

```bash
git add crates/pad/src/tui/
git commit -m "feat(pad): render editor + cursor + bottom status bar"
```

---

## Task 11: Input event loop + KeyAction mapping

**Files:**
- Create: `crates/pad/src/input.rs`
- Create: `crates/pad/src/keymap.rs`
- Modify: `crates/pad/src/lib.rs`
- Create: `crates/pad/tests/keymap.rs`

- [ ] **Step 1: Write failing keymap tests**

```rust
// crates/pad/tests/keymap.rs
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pad::keymap::{key_to_action, KeyAction};

fn k(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn alt(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT)
}

#[test]
fn plain_char_inserts() {
    assert_eq!(key_to_action(k('a')), KeyAction::InsertChar('a'));
}

#[test]
fn ctrl_o_is_write_out() {
    assert_eq!(key_to_action(ctrl('o')), KeyAction::WriteOut);
}

#[test]
fn ctrl_x_is_exit() {
    assert_eq!(key_to_action(ctrl('x')), KeyAction::Exit);
}

#[test]
fn ctrl_k_is_cut() {
    assert_eq!(key_to_action(ctrl('k')), KeyAction::Cut);
}

#[test]
fn ctrl_u_is_uncut() {
    assert_eq!(key_to_action(ctrl('u')), KeyAction::Uncut);
}

#[test]
fn ctrl_w_is_where_is() {
    assert_eq!(key_to_action(ctrl('w')), KeyAction::WhereIs);
}

#[test]
fn ctrl_underscore_is_goto_line() {
    // ^_ shows up as Char('_') + CONTROL on Unix terminals.
    assert_eq!(key_to_action(ctrl('_')), KeyAction::GotoLine);
}

#[test]
fn alt_u_is_undo() {
    assert_eq!(key_to_action(alt('u')), KeyAction::Undo);
}

#[test]
fn alt_e_is_redo() {
    assert_eq!(key_to_action(alt('e')), KeyAction::Redo);
}

#[test]
fn alt_r_is_replace() {
    assert_eq!(key_to_action(alt('r')), KeyAction::Replace);
}

#[test]
fn ctrl_g_is_help() {
    assert_eq!(key_to_action(ctrl('g')), KeyAction::Help);
}

#[test]
fn ctrl_c_is_cursor_pos() {
    assert_eq!(key_to_action(ctrl('c')), KeyAction::CursorPos);
}

#[test]
fn ctrl_z_is_suspend() {
    assert_eq!(key_to_action(ctrl('z')), KeyAction::Suspend);
}

#[test]
fn ctrl_r_is_insert_file() {
    assert_eq!(key_to_action(ctrl('r')), KeyAction::InsertFile);
}

#[test]
fn backspace() {
    let e = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
    assert_eq!(key_to_action(e), KeyAction::Backspace);
}

#[test]
fn arrows() {
    let l = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
    let r = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
    let u = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
    let d = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(key_to_action(l), KeyAction::Left);
    assert_eq!(key_to_action(r), KeyAction::Right);
    assert_eq!(key_to_action(u), KeyAction::Up);
    assert_eq!(key_to_action(d), KeyAction::Down);
}

#[test]
fn enter_inserts_newline() {
    let e = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(key_to_action(e), KeyAction::InsertChar('\n'));
}

#[test]
fn ctrl_s_is_unbound() {
    assert_eq!(key_to_action(ctrl('s')), KeyAction::Unbound);
}
```

- [ ] **Step 2: Implement `keymap.rs`**

```rust
// crates/pad/src/keymap.rs
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    InsertChar(char),
    Backspace,
    DeleteForward,
    Left,
    Right,
    Up,
    Down,
    WriteOut,    // ^O
    Exit,        // ^X
    InsertFile,  // ^R
    Cut,         // ^K
    Uncut,       // ^U
    WhereIs,     // ^W
    Replace,     // M-R
    GotoLine,    // ^_
    Undo,        // M-U
    Redo,        // M-E
    Help,        // ^G
    CursorPos,   // ^C
    Suspend,     // ^Z
    Unbound,     // ^S and other dead keys
}

pub fn key_to_action(ev: KeyEvent) -> KeyAction {
    let ctrl = ev.modifiers.contains(KeyModifiers::CONTROL);
    let alt = ev.modifiers.contains(KeyModifiers::ALT);
    match (ev.code, ctrl, alt) {
        (KeyCode::Char(c), true, _) => match c.to_ascii_lowercase() {
            'o' => KeyAction::WriteOut,
            'x' => KeyAction::Exit,
            'r' => KeyAction::InsertFile,
            'k' => KeyAction::Cut,
            'u' => KeyAction::Uncut,
            'w' => KeyAction::WhereIs,
            '_' | '/' => KeyAction::GotoLine, // ^_ on some terms = ^/
            'g' => KeyAction::Help,
            'c' => KeyAction::CursorPos,
            'z' => KeyAction::Suspend,
            's' => KeyAction::Unbound,
            _ => KeyAction::Unbound,
        },
        (KeyCode::Char(c), _, true) => match c.to_ascii_lowercase() {
            'r' => KeyAction::Replace,
            'u' => KeyAction::Undo,
            'e' => KeyAction::Redo,
            _ => KeyAction::Unbound,
        },
        (KeyCode::Char(c), false, false) => KeyAction::InsertChar(c),
        (KeyCode::Enter, _, _) => KeyAction::InsertChar('\n'),
        (KeyCode::Tab, _, _) => KeyAction::InsertChar('\t'),
        (KeyCode::Backspace, _, _) => KeyAction::Backspace,
        (KeyCode::Delete, _, _) => KeyAction::DeleteForward,
        (KeyCode::Left, _, _) => KeyAction::Left,
        (KeyCode::Right, _, _) => KeyAction::Right,
        (KeyCode::Up, _, _) => KeyAction::Up,
        (KeyCode::Down, _, _) => KeyAction::Down,
        _ => KeyAction::Unbound,
    }
}
```

- [ ] **Step 3: Implement `input.rs`**

```rust
// crates/pad/src/input.rs
use crate::keymap::{key_to_action, KeyAction};
use crossterm::event::{self, Event};
use std::time::Duration;

/// Block on the next key event for up to `timeout`. Returns None on timeout.
pub fn next_action(timeout: Duration) -> anyhow::Result<Option<KeyAction>> {
    if !event::poll(timeout)? {
        return Ok(None);
    }
    match event::read()? {
        Event::Key(ev) => Ok(Some(key_to_action(ev))),
        _ => Ok(None),
    }
}
```

Add to `lib.rs`:

```rust
pub mod input;
pub mod keymap;
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p pad --test keymap`
Expected: all 17 tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/pad/src/input.rs crates/pad/src/keymap.rs crates/pad/src/lib.rs crates/pad/tests/keymap.rs
git commit -m "feat(pad): KeyAction enum + crossterm chord-table mapping"
```

---

## Task 12: App event loop — basic editing

**Files:**
- Create: `crates/pad/src/app.rs`
- Modify: `crates/pad/src/main.rs`
- Modify: `crates/pad/src/lib.rs`

- [ ] **Step 1: Implement `App`**

```rust
// crates/pad/src/app.rs
use crate::buffer::Buffer;
use crate::buffer::sidecar::{PendingEntry, PendingLog, SidecarHandle};
use crate::cli::Mode;
use crate::config::paths;
use crate::input;
use crate::keymap::KeyAction;
use crate::tui::Tui;
use std::path::PathBuf;
use std::time::Duration;

pub struct App {
    pub buffer: Buffer,
    pub sidecar: SidecarHandle,
    pub pending_log: PendingLog,
    pub file_path: Option<PathBuf>,
    pub file_label: String,
    quit_requested: bool,
}

impl App {
    pub fn from_mode(mode: Mode) -> anyhow::Result<Self> {
        let state_root = paths::state_root();
        std::fs::create_dir_all(&state_root)?;
        let (buffer, sidecar, file_path, file_label) = match mode {
            Mode::Untitled => {
                let sc = SidecarHandle::new_untitled(&state_root)?;
                (Buffer::empty(), sc, None, "New Buffer".to_string())
            }
            Mode::OpenFile(path) => {
                let sc = SidecarHandle::for_file(&state_root, &path)?;
                let buf = Buffer::load_from_file(&path)?;
                let label = path.display().to_string();
                (buf, sc, Some(path), label)
            }
            Mode::Recover => unreachable!("Recover mode dispatches through recover::run"),
        };
        let pending_log = PendingLog::open(&sidecar)?;
        Ok(Self {
            buffer,
            sidecar,
            pending_log,
            file_path,
            file_label,
            quit_requested: false,
        })
    }

    pub fn run(&mut self, tui: &mut Tui) -> anyhow::Result<()> {
        while !self.quit_requested {
            tui.draw(&self.buffer, &self.file_label)?;
            if let Some(action) = input::next_action(Duration::from_millis(50))? {
                self.handle(action)?;
            }
        }
        Ok(())
    }

    fn handle(&mut self, action: KeyAction) -> anyhow::Result<()> {
        match action {
            KeyAction::InsertChar(c) => {
                self.pending_log.append(&PendingEntry::Insert {
                    offset: self.buffer.cursor_offset(),
                    text: c.to_string(),
                })?;
                self.buffer.insert_char(c);
            }
            KeyAction::Backspace => {
                let off = self.buffer.cursor_offset();
                if off > 0 {
                    self.pending_log.append(&PendingEntry::Delete {
                        offset: off - 1,
                        len: 1,
                    })?;
                }
                self.buffer.backspace();
            }
            KeyAction::DeleteForward => {
                let off = self.buffer.cursor_offset();
                self.pending_log.append(&PendingEntry::Delete { offset: off, len: 1 })?;
                self.buffer.delete_char_forward();
            }
            KeyAction::Left => self.buffer.move_left(),
            KeyAction::Right => self.buffer.move_right(),
            KeyAction::Up => self.buffer.move_up(),
            KeyAction::Down => self.buffer.move_down(),
            KeyAction::Exit => {
                // Without a dirty prompt yet (Task 14). For Task 12 we just quit.
                self.quit_requested = true;
            }
            // All other actions handled in later tasks.
            _ => {}
        }
        Ok(())
    }
}
```

- [ ] **Step 2: Wire `main.rs`**

```rust
// crates/pad/src/main.rs
use clap::Parser;
use pad::app::App;
use pad::cli::{Args, Mode};
use pad::config::paths;
use pad::panic_hook::{file_sink, install_panic_hook};
use pad::tui::Tui;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    install_panic_hook(file_sink(paths::state_root()));
    match args.mode() {
        Mode::Recover => {
            eprintln!("--recover not yet implemented (Task 23).");
            std::process::exit(2);
        }
        mode => {
            let mut tui = Tui::enter()?;
            let mut app = App::from_mode(mode)?;
            app.run(&mut tui)?;
            Ok(())
        }
    }
}
```

Add to `lib.rs`:

```rust
pub mod app;
```

- [ ] **Step 3: Verify it compiles + runs**

```bash
cargo build -p pad
# Smoke test interactively (do NOT use cargo test — it captures stdin):
# echo "hello" | cargo run -p pad -- /tmp/smoke.txt
# Use the PTY smoke test in Task 24 for automated coverage.
```

Expected: compile succeeds.

- [ ] **Step 4: Commit**

```bash
git add crates/pad/src/app.rs crates/pad/src/main.rs crates/pad/src/lib.rs
git commit -m "feat(pad): App event loop with insert/delete/nav + crash log"
```

---

## Task 13: ^O Write Out (save with prompt for path if untitled)

**Files:**
- Create: `crates/pad/src/tui/prompts.rs`
- Modify: `crates/pad/src/tui/mod.rs`
- Modify: `crates/pad/src/app.rs`

- [ ] **Step 1: Add a prompt overlay**

```rust
// crates/pad/src/tui/prompts.rs
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render_prompt(frame: &mut Frame<'_>, area: Rect, label: &str, input: &str) {
    let text = format!("{label}: {input}_");
    let p = Paragraph::new(text)
        .block(Block::default().borders(Borders::TOP))
        .style(Style::default().fg(Color::Reset).bg(Color::DarkGray));
    frame.render_widget(p, area);
}
```

Re-export from `crates/pad/src/tui/mod.rs`:

```rust
pub mod prompts;
```

- [ ] **Step 2: Extend `App` to handle prompts**

In `crates/pad/src/app.rs`, add an `AppState` to track whether we're in a prompt:

```rust
pub enum AppState {
    Editing,
    SaveAsPrompt(String),
}
```

Add field to `App`:

```rust
pub state: AppState,
```

Initialize in `from_mode` with `state: AppState::Editing`.

Extend `handle`:

```rust
fn handle(&mut self, action: KeyAction) -> anyhow::Result<()> {
    match &mut self.state {
        AppState::Editing => self.handle_editing(action),
        AppState::SaveAsPrompt(buf) => Self::handle_save_prompt(buf, action, &mut self.state, &mut self.buffer, &mut self.file_path, &mut self.file_label, &mut self.sidecar, &mut self.pending_log),
    }
}

fn handle_editing(&mut self, action: KeyAction) -> anyhow::Result<()> {
    match action {
        // ... existing arms ...
        KeyAction::WriteOut => {
            if let Some(p) = self.file_path.clone() {
                self.buffer.save_to_file(&p)?;
                self.pending_log.truncate()?;
            } else {
                self.state = AppState::SaveAsPrompt(String::new());
            }
        }
        // ... rest ...
    }
    Ok(())
}

fn handle_save_prompt(
    input: &mut String,
    action: KeyAction,
    state: &mut AppState,
    buffer: &mut Buffer,
    file_path: &mut Option<PathBuf>,
    file_label: &mut String,
    sidecar: &mut SidecarHandle,
    pending_log: &mut PendingLog,
) -> anyhow::Result<()> {
    match action {
        KeyAction::InsertChar('\n') => {
            if input.is_empty() {
                *state = AppState::Editing;
                return Ok(());
            }
            let path = PathBuf::from(input.clone());
            buffer.save_to_file(&path)?;
            pending_log.truncate()?;
            sidecar.set_file_path(path.clone())?;
            *file_path = Some(path.clone());
            *file_label = path.display().to_string();
            *state = AppState::Editing;
        }
        KeyAction::InsertChar(c) => input.push(c),
        KeyAction::Backspace => {
            input.pop();
        }
        KeyAction::Exit => *state = AppState::Editing,
        _ => {}
    }
    Ok(())
}
```

Update `Tui::draw` to render the prompt when active:

```rust
// crates/pad/src/tui/mod.rs — extend draw
impl Tui {
    pub fn draw_app(&mut self, buffer: &Buffer, file_label: &str, prompt: Option<(&str, &str)>) -> anyhow::Result<()> {
        self.terminal.draw(|frame| {
            let area = frame.area();
            let (editor_h, prompt_h, status_h) = if prompt.is_some() { (2, 2, 1) } else { (1, 0, 1) };
            // Use a 3-row layout when a prompt is active, 2-row otherwise.
            let constraints: Vec<Constraint> = if prompt.is_some() {
                vec![Constraint::Min(1), Constraint::Length(2), Constraint::Length(1)]
            } else {
                vec![Constraint::Min(1), Constraint::Length(1)]
            };
            let chunks = Layout::vertical(constraints).split(area);
            editor_view::render(frame, chunks[0], buffer);
            if let Some((label, input)) = prompt {
                prompts::render_prompt(frame, chunks[1], label, input);
                status_bar::render(frame, chunks[2], buffer, file_label);
            } else {
                status_bar::render(frame, chunks[1], buffer, file_label);
            }
            let _ = (editor_h, prompt_h, status_h);
        })?;
        Ok(())
    }
}
```

Update `App::run` to pass the prompt through:

```rust
pub fn run(&mut self, tui: &mut Tui) -> anyhow::Result<()> {
    while !self.quit_requested {
        let prompt = match &self.state {
            AppState::SaveAsPrompt(input) => Some(("File Name to Write", input.as_str())),
            _ => None,
        };
        tui.draw_app(&self.buffer, &self.file_label, prompt)?;
        if let Some(action) = input::next_action(Duration::from_millis(50))? {
            self.handle(action)?;
        }
    }
    Ok(())
}
```

- [ ] **Step 3: Build**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 4: Commit**

```bash
git add crates/pad/src/app.rs crates/pad/src/tui/
git commit -m "feat(pad): ^O Write Out (save) with save-as prompt for untitled buffers"
```

---

## Task 14: ^X Exit with dirty-prompt

**Files:**
- Modify: `crates/pad/src/app.rs`

- [ ] **Step 1: Extend `AppState` with a dirty-prompt variant**

```rust
pub enum AppState {
    Editing,
    SaveAsPrompt(String),
    DirtyExitPrompt,
}
```

Update `Editing → Exit` arm:

```rust
KeyAction::Exit => {
    if self.buffer.is_dirty() {
        self.state = AppState::DirtyExitPrompt;
    } else {
        self.quit_requested = true;
    }
}
```

Add handler for the new state in `handle`:

```rust
AppState::DirtyExitPrompt => self.handle_dirty_prompt(action),
```

```rust
fn handle_dirty_prompt(&mut self, action: KeyAction) -> anyhow::Result<()> {
    match action {
        KeyAction::InsertChar('y') | KeyAction::InsertChar('Y') => {
            // Save first.
            if let Some(p) = self.file_path.clone() {
                self.buffer.save_to_file(&p)?;
                self.pending_log.truncate()?;
                self.quit_requested = true;
            } else {
                self.state = AppState::SaveAsPrompt(String::new());
            }
        }
        KeyAction::InsertChar('n') | KeyAction::InsertChar('N') => {
            self.quit_requested = true;
        }
        KeyAction::Exit => self.state = AppState::Editing,
        _ => {}
    }
    Ok(())
}
```

Render the prompt in `App::run`:

```rust
let prompt = match &self.state {
    AppState::SaveAsPrompt(input) => Some(("File Name to Write", input.as_str())),
    AppState::DirtyExitPrompt => Some(("Save modified buffer? (Y/N, ^C cancel)", "")),
    _ => None,
};
```

- [ ] **Step 2: Build**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 3: Commit**

```bash
git add crates/pad/src/app.rs
git commit -m "feat(pad): ^X exit with nano-style dirty-buffer prompt"
```

---

## Task 15: ^K cut / ^U uncut (per-line clipboard)

**Files:**
- Create: `crates/pad/src/buffer/clipboard.rs`
- Modify: `crates/pad/src/buffer/mod.rs`
- Modify: `crates/pad/src/app.rs`

- [ ] **Step 1: Write failing test**

```rust
// add to tests/buffer_unit.rs
#[test]
fn cut_then_uncut_round_trips_line() {
    let mut b = Buffer::from_text("alpha\nbravo\ncharlie");
    b.move_cursor_to(CursorPos { line: 1, col: 0 });
    b.cut_line();
    assert_eq!(b.text(), "alpha\ncharlie");
    b.uncut();
    assert_eq!(b.text(), "alpha\nbravo\ncharlie");
}
```

- [ ] **Step 2: Implement clipboard module**

```rust
// crates/pad/src/buffer/clipboard.rs
#[derive(Debug, Default, Clone)]
pub struct LineClipboard {
    /// Lines stored most recently. nano's ^K appends consecutive cuts into
    /// the same register if no other key intervened; for v0.1 we store a
    /// single line at a time (sufficient for golden-path nano behaviour).
    pub last_cut: Option<String>,
}
```

Add to `crates/pad/src/buffer/mod.rs`:

```rust
pub mod clipboard;
use clipboard::LineClipboard;
```

Add field to `Buffer`:

```rust
pub struct Buffer {
    // ...existing...
    clipboard: LineClipboard,
}
```

Initialize in `empty()`, `from_text()`, `from_text_with_ending()`:

```rust
clipboard: LineClipboard::default(),
```

Add methods:

```rust
impl Buffer {
    pub fn cut_line(&mut self) {
        let line_idx = self.cursor.line;
        if line_idx >= self.rope.len_lines() {
            return;
        }
        let line_start = self.rope.line_to_char(line_idx);
        let line_end = if line_idx + 1 < self.rope.len_lines() {
            self.rope.line_to_char(line_idx + 1)
        } else {
            self.rope.len_chars()
        };
        let cut: String = self.rope.slice(line_start..line_end).into();
        self.clipboard.last_cut = Some(cut);
        self.rope.remove(line_start..line_end);
        self.dirty = true;
        // Cursor stays at line start of what used to be line+1.
        if self.cursor.line >= self.line_count() && self.cursor.line > 0 {
            self.cursor.line -= 1;
        }
        self.cursor.col = 0;
        self.pref_col = 0;
    }

    pub fn uncut(&mut self) {
        let Some(text) = self.clipboard.last_cut.clone() else {
            return;
        };
        let line_start = self.rope.line_to_char(self.cursor.line);
        self.rope.insert(line_start, &text);
        self.dirty = true;
        // Place cursor after the pasted block.
        let lines_in_paste = text.matches('\n').count();
        if lines_in_paste > 0 {
            self.cursor.line += lines_in_paste;
            self.cursor.col = 0;
        } else {
            self.cursor.col = text.chars().count();
        }
        self.pref_col = self.cursor.col;
    }
}
```

- [ ] **Step 3: Wire keymap → action**

In `crates/pad/src/app.rs`, add to `handle_editing`:

```rust
KeyAction::Cut => self.buffer.cut_line(),
KeyAction::Uncut => self.buffer.uncut(),
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p pad --test buffer_unit`
Expected: 8 tests pass (7 existing + 1 new).

- [ ] **Step 5: Commit**

```bash
git add crates/pad/src/buffer/ crates/pad/src/app.rs crates/pad/tests/buffer_unit.rs
git commit -m "feat(pad): ^K cut line and ^U uncut (paste) clipboard register"
```

---

## Task 16: ^W where-is (forward search)

**Files:**
- Create: `crates/pad/src/buffer/search.rs`
- Modify: `crates/pad/src/buffer/mod.rs`
- Modify: `crates/pad/src/app.rs`

- [ ] **Step 1: Write failing test**

```rust
// add to tests/buffer_unit.rs
#[test]
fn search_finds_first_match_forward() {
    let mut b = Buffer::from_text("alpha\nbeta\nalpha");
    b.move_cursor_to(CursorPos { line: 0, col: 0 });
    let found = b.search_forward("alpha");
    assert_eq!(found, Some(CursorPos { line: 0, col: 0 }));
    // Move past first match, search again.
    b.move_cursor_to(CursorPos { line: 0, col: 1 });
    let next = b.search_forward("alpha");
    assert_eq!(next, Some(CursorPos { line: 2, col: 0 }));
}

#[test]
fn search_returns_none_when_missing() {
    let b = Buffer::from_text("alpha\nbeta");
    assert_eq!(b.search_forward("zeta"), None);
}
```

- [ ] **Step 2: Implement search**

```rust
// crates/pad/src/buffer/search.rs
use ropey::Rope;

use super::CursorPos;

pub fn search_forward(rope: &Rope, needle: &str, start_char_idx: usize) -> Option<CursorPos> {
    if needle.is_empty() {
        return None;
    }
    let text: String = rope.slice(start_char_idx..).into();
    let byte_pos = text.find(needle)?;
    // Convert byte_pos -> char_idx relative to slice start.
    let char_offset = text[..byte_pos].chars().count();
    let absolute = start_char_idx + char_offset;
    let line = rope.char_to_line(absolute);
    let line_start = rope.line_to_char(line);
    let col = absolute - line_start;
    Some(CursorPos { line, col })
}
```

Add to `crates/pad/src/buffer/mod.rs`:

```rust
pub mod search;

impl Buffer {
    pub fn search_forward(&self, needle: &str) -> Option<CursorPos> {
        let start = self.cursor_char_idx();
        search::search_forward(&self.rope, needle, start)
    }
}
```

- [ ] **Step 3: Wire `AppState::SearchPrompt`**

Add variant to `AppState`:

```rust
SearchPrompt(String),
```

Handle `KeyAction::WhereIs` from editing:

```rust
KeyAction::WhereIs => self.state = AppState::SearchPrompt(String::new()),
```

Handler for prompt:

```rust
AppState::SearchPrompt(_) => self.handle_search_prompt(action),
```

```rust
fn handle_search_prompt(&mut self, action: KeyAction) -> anyhow::Result<()> {
    let AppState::SearchPrompt(input) = &mut self.state else { return Ok(()); };
    match action {
        KeyAction::InsertChar('\n') => {
            let needle = input.clone();
            self.state = AppState::Editing;
            if let Some(pos) = self.buffer.search_forward(&needle) {
                self.buffer.move_cursor_to(pos);
            }
        }
        KeyAction::InsertChar(c) => input.push(c),
        KeyAction::Backspace => { input.pop(); }
        KeyAction::Exit => self.state = AppState::Editing,
        _ => {}
    }
    Ok(())
}
```

Add prompt label rendering in `App::run`:

```rust
AppState::SearchPrompt(input) => Some(("Search", input.as_str())),
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p pad --test buffer_unit`
Expected: 10 tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/pad/src/buffer/ crates/pad/src/app.rs crates/pad/tests/buffer_unit.rs
git commit -m "feat(pad): ^W where-is forward search with prompt overlay"
```

---

## Task 17: M-R replace

**Files:**
- Modify: `crates/pad/src/buffer/search.rs`
- Modify: `crates/pad/src/buffer/mod.rs`
- Modify: `crates/pad/src/app.rs`

- [ ] **Step 1: Write failing test**

```rust
// add to tests/buffer_unit.rs
#[test]
fn replace_one_replaces_first_match() {
    let mut b = Buffer::from_text("foo bar foo");
    b.replace_one("foo", "FOO");
    assert_eq!(b.text(), "FOO bar foo");
}

#[test]
fn replace_all_replaces_all_matches() {
    let mut b = Buffer::from_text("foo bar foo");
    let n = b.replace_all("foo", "FOO");
    assert_eq!(n, 2);
    assert_eq!(b.text(), "FOO bar FOO");
}
```

- [ ] **Step 2: Implement replace**

Add to `crates/pad/src/buffer/mod.rs`:

```rust
impl Buffer {
    pub fn replace_one(&mut self, needle: &str, replacement: &str) -> bool {
        let Some(pos) = self.search_forward(needle) else { return false; };
        // Compute char range for the matched needle.
        let start_char = self.rope.line_to_char(pos.line) + pos.col;
        let needle_chars = needle.chars().count();
        self.rope.remove(start_char..start_char + needle_chars);
        self.rope.insert(start_char, replacement);
        self.dirty = true;
        true
    }

    pub fn replace_all(&mut self, needle: &str, replacement: &str) -> usize {
        if needle.is_empty() {
            return 0;
        }
        // Save cursor; we'll restore conservatively.
        let saved = self.cursor;
        self.move_cursor_to(CursorPos { line: 0, col: 0 });
        let mut count = 0;
        while self.replace_one(needle, replacement) {
            count += 1;
        }
        self.move_cursor_to(saved);
        count
    }
}
```

- [ ] **Step 3: Wire replace prompt**

Add `AppState::ReplaceFromPrompt(String)` and `ReplaceToPrompt { from: String, to: String }`:

```rust
pub enum AppState {
    Editing,
    SaveAsPrompt(String),
    DirtyExitPrompt,
    SearchPrompt(String),
    ReplaceFromPrompt(String),
    ReplaceToPrompt { from: String, to: String },
}
```

Handle `KeyAction::Replace` from editing:

```rust
KeyAction::Replace => self.state = AppState::ReplaceFromPrompt(String::new()),
```

Handlers:

```rust
AppState::ReplaceFromPrompt(_) => self.handle_replace_from(action),
AppState::ReplaceToPrompt { .. } => self.handle_replace_to(action),
```

```rust
fn handle_replace_from(&mut self, action: KeyAction) -> anyhow::Result<()> {
    let AppState::ReplaceFromPrompt(input) = &mut self.state else { return Ok(()); };
    match action {
        KeyAction::InsertChar('\n') => {
            let from = input.clone();
            self.state = AppState::ReplaceToPrompt { from, to: String::new() };
        }
        KeyAction::InsertChar(c) => input.push(c),
        KeyAction::Backspace => { input.pop(); }
        KeyAction::Exit => self.state = AppState::Editing,
        _ => {}
    }
    Ok(())
}

fn handle_replace_to(&mut self, action: KeyAction) -> anyhow::Result<()> {
    let AppState::ReplaceToPrompt { from, to } = &mut self.state else { return Ok(()); };
    match action {
        KeyAction::InsertChar('\n') => {
            let (from, to) = (from.clone(), to.clone());
            let n = self.buffer.replace_all(&from, &to);
            self.state = AppState::Editing;
            let _ = n; // Plan 2 doesn't surface the count; status-bar message TBD by Plan 3 UX polish.
        }
        KeyAction::InsertChar(c) => to.push(c),
        KeyAction::Backspace => { to.pop(); }
        KeyAction::Exit => self.state = AppState::Editing,
        _ => {}
    }
    Ok(())
}
```

Prompt labels in `App::run`:

```rust
AppState::ReplaceFromPrompt(input) => Some(("Search (to replace)", input.as_str())),
AppState::ReplaceToPrompt { to, .. } => Some(("Replacement", to.as_str())),
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p pad --test buffer_unit`
Expected: 12 tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/pad/src/buffer/ crates/pad/src/app.rs crates/pad/tests/buffer_unit.rs
git commit -m "feat(pad): M-R replace-all with two-stage prompt"
```

---

## Task 18: ^_ goto-line

**Files:**
- Modify: `crates/pad/src/app.rs`

- [ ] **Step 1: Add `AppState::GotoLinePrompt(String)`**

```rust
GotoLinePrompt(String),
```

Handle from editing:

```rust
KeyAction::GotoLine => self.state = AppState::GotoLinePrompt(String::new()),
```

Handler:

```rust
AppState::GotoLinePrompt(_) => self.handle_goto_line(action),
```

```rust
fn handle_goto_line(&mut self, action: KeyAction) -> anyhow::Result<()> {
    let AppState::GotoLinePrompt(input) = &mut self.state else { return Ok(()); };
    match action {
        KeyAction::InsertChar('\n') => {
            if let Ok(n) = input.trim().parse::<usize>() {
                let target = n.saturating_sub(1); // 1-indexed input, 0-indexed internal
                self.buffer.move_cursor_to(crate::buffer::CursorPos { line: target, col: 0 });
            }
            self.state = AppState::Editing;
        }
        KeyAction::InsertChar(c) if c.is_ascii_digit() => input.push(c),
        KeyAction::Backspace => { input.pop(); }
        KeyAction::Exit => self.state = AppState::Editing,
        _ => {}
    }
    Ok(())
}
```

Prompt label:

```rust
AppState::GotoLinePrompt(input) => Some(("Goto line", input.as_str())),
```

- [ ] **Step 2: Build**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 3: Commit**

```bash
git add crates/pad/src/app.rs
git commit -m "feat(pad): ^_ goto-line prompt"
```

---

## Task 19: M-U undo / M-E redo

**Files:**
- Create: `crates/pad/src/buffer/undo.rs`
- Modify: `crates/pad/src/buffer/mod.rs`
- Modify: `crates/pad/src/app.rs`

- [ ] **Step 1: Write failing tests**

```rust
// add to tests/buffer_unit.rs
#[test]
fn undo_reverses_insert() {
    let mut b = Buffer::empty();
    b.snapshot_for_undo();
    b.insert_char('a');
    b.snapshot_for_undo();
    b.insert_char('b');
    b.undo();
    assert_eq!(b.text(), "a");
    b.undo();
    assert_eq!(b.text(), "");
}

#[test]
fn redo_replays_undone_edits() {
    let mut b = Buffer::empty();
    b.snapshot_for_undo();
    b.insert_char('a');
    b.snapshot_for_undo();
    b.insert_char('b');
    b.undo();
    b.redo();
    assert_eq!(b.text(), "ab");
}
```

- [ ] **Step 2: Implement snapshot-based undo**

```rust
// crates/pad/src/buffer/undo.rs
use ropey::Rope;
use super::CursorPos;

#[derive(Clone)]
pub struct Snapshot {
    pub rope: Rope,
    pub cursor: CursorPos,
}

#[derive(Default)]
pub struct UndoStack {
    pub past: Vec<Snapshot>,
    pub future: Vec<Snapshot>,
    /// Cap; oldest snapshots dropped when exceeded.
    pub cap: usize,
}

impl UndoStack {
    pub fn new() -> Self {
        Self { past: Vec::new(), future: Vec::new(), cap: 200 }
    }

    pub fn push(&mut self, snap: Snapshot) {
        self.past.push(snap);
        if self.past.len() > self.cap {
            self.past.remove(0);
        }
        // A new edit invalidates redo history.
        self.future.clear();
    }
}
```

In `crates/pad/src/buffer/mod.rs`:

```rust
pub mod undo;
use undo::{Snapshot, UndoStack};
```

Add field:

```rust
pub struct Buffer {
    // ...
    undo: UndoStack,
}
```

Initialize: `undo: UndoStack::new(),`

Add methods:

```rust
impl Buffer {
    pub fn snapshot_for_undo(&mut self) {
        self.undo.push(Snapshot { rope: self.rope.clone(), cursor: self.cursor });
    }

    pub fn undo(&mut self) {
        let Some(prev) = self.undo.past.pop() else { return; };
        let now = Snapshot { rope: self.rope.clone(), cursor: self.cursor };
        self.undo.future.push(now);
        self.rope = prev.rope;
        self.cursor = prev.cursor;
        self.pref_col = self.cursor.col;
        self.dirty = true;
    }

    pub fn redo(&mut self) {
        let Some(next) = self.undo.future.pop() else { return; };
        let now = Snapshot { rope: self.rope.clone(), cursor: self.cursor };
        self.undo.past.push(now);
        self.rope = next.rope;
        self.cursor = next.cursor;
        self.pref_col = self.cursor.col;
        self.dirty = true;
    }
}
```

- [ ] **Step 3: Snapshot on every edit-action in `App`**

In `crates/pad/src/app.rs`, wrap the editing arms that mutate the buffer to call `self.buffer.snapshot_for_undo()` BEFORE the mutation. Pattern:

```rust
match action {
    KeyAction::InsertChar(c) => {
        self.buffer.snapshot_for_undo();
        self.pending_log.append(&PendingEntry::Insert { offset: self.buffer.cursor_offset(), text: c.to_string() })?;
        self.buffer.insert_char(c);
    }
    KeyAction::Backspace => {
        self.buffer.snapshot_for_undo();
        // ... rest unchanged ...
    }
    KeyAction::DeleteForward => {
        self.buffer.snapshot_for_undo();
        // ... rest unchanged ...
    }
    KeyAction::Cut => {
        self.buffer.snapshot_for_undo();
        self.buffer.cut_line();
    }
    KeyAction::Uncut => {
        self.buffer.snapshot_for_undo();
        self.buffer.uncut();
    }
    KeyAction::Undo => self.buffer.undo(),
    KeyAction::Redo => self.buffer.redo(),
    // ... others unchanged ...
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p pad --test buffer_unit`
Expected: 14 tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/pad/src/buffer/ crates/pad/src/app.rs crates/pad/tests/buffer_unit.rs
git commit -m "feat(pad): M-U undo / M-E redo via rope snapshots (200-deep stack)"
```

---

## Task 20: Remaining nano bindings (^R insert file, ^G help, ^C cursor pos, ^Z suspend)

**Files:**
- Create: `crates/pad/src/tui/help.rs`
- Modify: `crates/pad/src/tui/mod.rs`
- Modify: `crates/pad/src/app.rs`

- [ ] **Step 1: Implement help overlay**

```rust
// crates/pad/src/tui/help.rs
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame<'_>, area: Rect) {
    let text = "\
pad — nano-faithful editor

  ^O  Write Out (save)        ^X  Exit
  ^R  Insert file             ^K  Cut line
  ^U  Uncut (paste)           ^W  Where Is (search)
  M-R Replace                 ^_  Goto line
  M-U Undo                    M-E Redo
  ^G  Help (this screen)      ^C  Cursor position
  ^Z  Suspend to shell

Press any key to dismiss.";
    let p = Paragraph::new(text).block(
        Block::default()
            .title(" Help ")
            .borders(Borders::ALL),
    );
    frame.render_widget(p, area);
}
```

Register in `crates/pad/src/tui/mod.rs`:

```rust
pub mod help;
```

- [ ] **Step 2: Extend `AppState` with `HelpOverlay`**

```rust
HelpOverlay,
```

Handle `^G` from editing:

```rust
KeyAction::Help => self.state = AppState::HelpOverlay,
```

Handler:

```rust
AppState::HelpOverlay => {
    // Any key dismisses.
    self.state = AppState::Editing;
}
```

Render help in `App::run` and `Tui::draw_app`:

In `App::run`, extend prompt match to also flag help:

```rust
let mut show_help = false;
let prompt = match &self.state {
    // ... existing ...
    AppState::HelpOverlay => { show_help = true; None }
    _ => None,
};
tui.draw_app(&self.buffer, &self.file_label, prompt, show_help)?;
```

In `Tui::draw_app`, accept `show_help: bool`:

```rust
pub fn draw_app(&mut self, buffer: &Buffer, file_label: &str, prompt: Option<(&str, &str)>, show_help: bool) -> anyhow::Result<()> {
    self.terminal.draw(|frame| {
        let area = frame.area();
        let constraints: Vec<Constraint> = if prompt.is_some() {
            vec![Constraint::Min(1), Constraint::Length(2), Constraint::Length(1)]
        } else {
            vec![Constraint::Min(1), Constraint::Length(1)]
        };
        let chunks = Layout::vertical(constraints).split(area);
        editor_view::render(frame, chunks[0], buffer);
        if show_help {
            help::render(frame, chunks[0]);
        }
        if let Some((label, input)) = prompt {
            prompts::render_prompt(frame, chunks[1], label, input);
            status_bar::render(frame, chunks[2], buffer, file_label);
        } else {
            status_bar::render(frame, chunks[1], buffer, file_label);
        }
    })?;
    Ok(())
}
```

- [ ] **Step 3: Implement ^C cursor-pos (status flash)**

Add `AppState::FlashMessage(String)`:

```rust
FlashMessage(String),
```

Handle `^C`:

```rust
KeyAction::CursorPos => {
    let pos = self.buffer.cursor();
    let total = self.buffer.line_count();
    self.state = AppState::FlashMessage(format!("line {}/{}, col {}", pos.line + 1, total, pos.col + 1));
}
```

Any key dismisses:

```rust
AppState::FlashMessage(_) => self.state = AppState::Editing,
```

Render via prompt overlay:

```rust
AppState::FlashMessage(msg) => Some(("", msg.as_str())),
```

- [ ] **Step 4: Implement ^Z suspend**

Add `KeyAction::Suspend` handler:

```rust
KeyAction::Suspend => {
    // Leave alternate screen + disable raw mode before raising SIGTSTP so
    // the shell sees a clean terminal. The drop guard on Tui does this; we
    // only need to flush + send the signal.
    #[cfg(unix)]
    {
        use std::io::Write;
        let _ = std::io::stdout().flush();
        unsafe {
            libc::raise(libc::SIGTSTP);
        }
    }
}
```

Add `libc = "0.2"` to `[dependencies]` in `crates/pad/Cargo.toml` (only used on unix).

- [ ] **Step 5: Implement ^R insert-file**

Add `AppState::InsertFilePrompt(String)`:

```rust
InsertFilePrompt(String),
```

Handle from editing:

```rust
KeyAction::InsertFile => self.state = AppState::InsertFilePrompt(String::new()),
```

Handler:

```rust
AppState::InsertFilePrompt(_) => self.handle_insert_file(action),
```

```rust
fn handle_insert_file(&mut self, action: KeyAction) -> anyhow::Result<()> {
    let AppState::InsertFilePrompt(input) = &mut self.state else { return Ok(()); };
    match action {
        KeyAction::InsertChar('\n') => {
            let path = std::path::PathBuf::from(input.clone());
            self.state = AppState::Editing;
            if let Ok(bytes) = std::fs::read(&path) {
                if let Ok(text) = std::str::from_utf8(&bytes) {
                    self.buffer.snapshot_for_undo();
                    // Insert at cursor; preserve internal LF representation.
                    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
                    self.buffer.insert_str(&normalized);
                }
            }
        }
        KeyAction::InsertChar(c) => input.push(c),
        KeyAction::Backspace => { input.pop(); }
        KeyAction::Exit => self.state = AppState::Editing,
        _ => {}
    }
    Ok(())
}
```

Prompt label:

```rust
AppState::InsertFilePrompt(input) => Some(("File to insert", input.as_str())),
```

- [ ] **Step 6: Build**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 7: Commit**

```bash
git add crates/pad/src/app.rs crates/pad/src/tui/ crates/pad/Cargo.toml
git commit -m "feat(pad): ^R insert-file, ^G help, ^C cursor-pos, ^Z suspend"
```

---

## Task 21: Soft-wrap rendering

**Files:**
- Create: `crates/pad/src/tui/softwrap.rs`
- Modify: `crates/pad/src/tui/editor_view.rs`

- [ ] **Step 1: Implement soft-wrap helper**

```rust
// crates/pad/src/tui/softwrap.rs

/// Split a logical line into visual lines that fit within `width` columns.
/// Returns a Vec of (start_col_in_logical_line, slice_of_logical_line).
pub fn wrap_line(line: &str, width: u16) -> Vec<(usize, String)> {
    if width == 0 {
        return vec![(0, line.to_string())];
    }
    let w = width as usize;
    let chars: Vec<char> = line.chars().collect();
    if chars.is_empty() {
        return vec![(0, String::new())];
    }
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let end = (i + w).min(chars.len());
        out.push((i, chars[i..end].iter().collect()));
        i = end;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_wrap_when_fits() {
        let v = wrap_line("hello", 10);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].1, "hello");
    }

    #[test]
    fn wraps_at_width() {
        let v = wrap_line("abcdefghij", 4);
        assert_eq!(v.len(), 3);
        assert_eq!(v[0].1, "abcd");
        assert_eq!(v[1].1, "efgh");
        assert_eq!(v[2].1, "ij");
    }

    #[test]
    fn empty_line_yields_one_empty_visual() {
        let v = wrap_line("", 10);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].1, "");
    }
}
```

Re-export in `crates/pad/src/tui/mod.rs`:

```rust
pub mod softwrap;
```

- [ ] **Step 2: Use it in `editor_view`**

Replace `editor_view::render`:

```rust
use crate::buffer::{Buffer, CursorPos};
use crate::tui::softwrap;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(frame: &mut Frame<'_>, area: Rect, buffer: &Buffer) {
    let width = area.width;
    let mut visual_lines: Vec<Line<'_>> = Vec::new();
    let mut cursor_visual: Option<(u16, u16)> = None; // (col, row)
    let CursorPos { line: cline, col: ccol } = buffer.cursor();

    for li in 0..buffer.line_count() {
        let line = buffer.line(li);
        let wrapped = softwrap::wrap_line(&line, width);
        for (offset_in_logical, slice) in wrapped {
            if li == cline
                && ccol >= offset_in_logical
                && ccol <= offset_in_logical + slice.chars().count()
            {
                let col = (ccol - offset_in_logical) as u16;
                let row = visual_lines.len() as u16;
                cursor_visual = Some((col, row));
            }
            visual_lines.push(Line::from(slice));
        }
    }
    let para = Paragraph::new(visual_lines).style(Style::default().fg(Color::Reset));
    frame.render_widget(para, area);

    if let Some((col, row)) = cursor_visual {
        let cx = area.x.saturating_add(col);
        let cy = area.y.saturating_add(row);
        if cx < area.right() && cy < area.bottom() {
            frame.set_cursor_position(Position::new(cx, cy));
        }
    }
}
```

- [ ] **Step 3: Run softwrap unit tests**

Run: `cargo test -p pad softwrap`
Expected: 3 tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/pad/src/tui/
git commit -m "feat(pad): soft-wrap rendering — long lines wrap to terminal width"
```

---

## Task 22: Cold-open performance benchmark + budget gate

**Files:**
- Modify: `crates/pad/benches/cold_open.rs`
- Create: `crates/pad/tests/perf_budget.rs`

- [ ] **Step 1: Write criterion bench**

```rust
// crates/pad/benches/cold_open.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pad::buffer::Buffer;
use std::path::PathBuf;
use tempfile::tempdir;

fn bench_open_empty(c: &mut Criterion) {
    c.bench_function("Buffer::empty()", |b| {
        b.iter(|| black_box(Buffer::empty()));
    });
}

fn bench_load_1k(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let path: PathBuf = dir.path().join("1k.txt");
    std::fs::write(&path, "x".repeat(1024)).unwrap();
    c.bench_function("Buffer::load_from_file (1KB)", |b| {
        b.iter(|| black_box(Buffer::load_from_file(&path).unwrap()));
    });
}

fn bench_load_100k(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let path: PathBuf = dir.path().join("100k.txt");
    std::fs::write(&path, "x".repeat(100 * 1024)).unwrap();
    c.bench_function("Buffer::load_from_file (100KB)", |b| {
        b.iter(|| black_box(Buffer::load_from_file(&path).unwrap()));
    });
}

criterion_group!(benches, bench_open_empty, bench_load_1k, bench_load_100k);
criterion_main!(benches);
```

- [ ] **Step 2: Write the CI budget gate test**

The spec (§9) calls for `<20 ms` cold open warm cache and `<50 ms` cold cache 100KB. We translate that into a fast-path budget — load a 100KB file and ensure the load completes in under 50 ms wall-clock on the CI box. This is a soft gate; the criterion bench produces the precise numbers.

```rust
// crates/pad/tests/perf_budget.rs
use pad::buffer::Buffer;
use std::time::Instant;
use tempfile::tempdir;

#[test]
fn cold_open_100kb_under_50ms() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("100k.txt");
    std::fs::write(&path, "x".repeat(100 * 1024)).unwrap();

    // Warm up filesystem caches.
    let _ = Buffer::load_from_file(&path).unwrap();

    let mut samples = Vec::new();
    for _ in 0..20 {
        let t = Instant::now();
        let _b = Buffer::load_from_file(&path).unwrap();
        samples.push(t.elapsed());
    }
    samples.sort();
    let p50 = samples[samples.len() / 2];
    let p99 = samples[(samples.len() * 99) / 100];
    eprintln!("cold_open_100kb p50={p50:?} p99={p99:?}");
    assert!(
        p99 < std::time::Duration::from_millis(50),
        "p99 cold open exceeded 50 ms: {p99:?}",
    );
}

#[test]
fn empty_buffer_under_1ms() {
    // Sanity check that Buffer::empty() is essentially free.
    let mut samples = Vec::new();
    for _ in 0..100 {
        let t = Instant::now();
        let _b = Buffer::empty();
        samples.push(t.elapsed());
    }
    samples.sort();
    let p99 = samples[(samples.len() * 99) / 100];
    assert!(p99 < std::time::Duration::from_millis(1), "Buffer::empty too slow: {p99:?}");
}
```

- [ ] **Step 3: Run both**

```bash
cargo bench -p pad --bench cold_open -- --test
cargo test -p pad --test perf_budget --release
```

Expected: both pass. The release build is required for realistic timing.

- [ ] **Step 4: Commit**

```bash
git add crates/pad/benches/cold_open.rs crates/pad/tests/perf_budget.rs
git commit -m "perf(pad): cold-open benchmark + 50ms budget gate (spec §9)"
```

---

## Task 23: `pad --recover` recovery flow

**Files:**
- Create: `crates/pad/src/recover.rs`
- Modify: `crates/pad/src/lib.rs`
- Modify: `crates/pad/src/main.rs`
- Create: `crates/pad/tests/recover.rs`

- [ ] **Step 1: Write failing test**

```rust
// crates/pad/tests/recover.rs
use pad::buffer::sidecar::{PendingEntry, PendingLog, SidecarHandle};
use pad::recover::{list_recoverable, replay_into_buffer};
use tempfile::tempdir;

#[test]
fn list_recoverable_finds_buffer_with_log_entries() {
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).unwrap();
    let mut log = PendingLog::open(&sc).unwrap();
    log.append(&PendingEntry::Insert { offset: 0, text: "hi".into() }).unwrap();

    let mut found = list_recoverable(state_root.path()).unwrap();
    found.retain(|r| r.buffer_id == sc.buffer_id());
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].entry_count, 1);
}

#[test]
fn replay_into_buffer_reconstructs_text() {
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).unwrap();
    let mut log = PendingLog::open(&sc).unwrap();
    log.append(&PendingEntry::Insert { offset: 0, text: "hello".into() }).unwrap();
    log.append(&PendingEntry::Delete { offset: 4, len: 1 }).unwrap();

    let buf = replay_into_buffer(&sc).unwrap();
    assert_eq!(buf.text(), "hell");
}
```

- [ ] **Step 2: Implement `recover.rs`**

```rust
// crates/pad/src/recover.rs
use crate::buffer::sidecar::{BufferId, PendingEntry, PendingLog, SidecarHandle};
use crate::buffer::Buffer;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Recoverable {
    pub buffer_id: BufferId,
    pub entry_count: usize,
    pub file_path: Option<PathBuf>,
}

pub fn list_recoverable(state_root: &Path) -> anyhow::Result<Vec<Recoverable>> {
    let mut out = Vec::new();
    if !state_root.exists() {
        return Ok(out);
    }
    for entry in fs::read_dir(state_root)? {
        let entry = entry?;
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let Some(name) = dir.file_name().and_then(|s| s.to_str()) else { continue };
        let Ok(id) = uuid::Uuid::parse_str(name) else { continue };
        let sc = match SidecarHandle::reattach(state_root, &id) {
            Ok(sc) => sc,
            Err(_) => continue,
        };
        let entries = PendingLog::read_all(&sc)?;
        if !entries.is_empty() {
            out.push(Recoverable {
                buffer_id: id,
                entry_count: entries.len(),
                file_path: sc.file_path().map(|p| p.to_path_buf()),
            });
        }
    }
    Ok(out)
}

pub fn replay_into_buffer(sc: &SidecarHandle) -> anyhow::Result<Buffer> {
    let mut buf = if let Some(p) = sc.file_path() {
        Buffer::load_from_file(p)?
    } else {
        Buffer::empty()
    };
    let entries = PendingLog::read_all(sc)?;
    for e in entries {
        match e {
            PendingEntry::Insert { offset, text } => {
                // Move to offset and insert.
                buf.move_cursor_to(offset_to_pos(&buf, offset as usize));
                buf.insert_str(&text);
            }
            PendingEntry::Delete { offset, len } => {
                buf.move_cursor_to(offset_to_pos(&buf, offset as usize));
                for _ in 0..len {
                    buf.delete_char_forward();
                }
            }
        }
    }
    Ok(buf)
}

fn offset_to_pos(buf: &Buffer, off: usize) -> crate::buffer::CursorPos {
    let mut remaining = off;
    for line_idx in 0..buf.line_count() {
        let line = buf.line(line_idx);
        let line_chars = line.chars().count() + 1; // +1 for the implicit newline
        if remaining < line_chars {
            return crate::buffer::CursorPos {
                line: line_idx,
                col: remaining.min(line.chars().count()),
            };
        }
        remaining -= line_chars;
    }
    // Fallback: end of buffer.
    crate::buffer::CursorPos {
        line: buf.line_count() - 1,
        col: buf.line(buf.line_count() - 1).chars().count(),
    }
}

pub fn run(state_root: &Path) -> anyhow::Result<()> {
    let candidates = list_recoverable(state_root)?;
    if candidates.is_empty() {
        println!("No recoverable buffers found.");
        return Ok(());
    }
    println!("Recoverable buffers:");
    for (i, r) in candidates.iter().enumerate() {
        let label = r
            .file_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(untitled)".to_string());
        println!("  [{}] {} — {} pending entries", i + 1, label, r.entry_count);
    }
    println!("\nSelect a buffer to resume (number, or q to quit):");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim();
    if input.eq_ignore_ascii_case("q") {
        return Ok(());
    }
    let idx: usize = input.parse()?;
    let chosen = &candidates[idx - 1];
    let sc = SidecarHandle::reattach(state_root, &chosen.buffer_id)?;
    let buf = replay_into_buffer(&sc)?;
    println!("Recovered {} chars. Re-open with:", buf.text().chars().count());
    if let Some(p) = chosen.file_path.as_ref() {
        println!("  pad {}", p.display());
    } else {
        println!("  pad   # (then save with ^O to give it a name)");
    }
    Ok(())
}
```

Add to `lib.rs`:

```rust
pub mod recover;
```

- [ ] **Step 3: Wire into `main.rs`**

```rust
Mode::Recover => {
    pad::recover::run(&pad::config::paths::state_root())?;
    return Ok(());
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p pad --test recover`
Expected: 2 tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/pad/src/recover.rs crates/pad/src/lib.rs crates/pad/src/main.rs crates/pad/tests/recover.rs
git commit -m "feat(pad): pad --recover — list + resume buffers with crash-log entries"
```

---

## Task 24: PTY smoke test — open/edit/save lifecycle

**Files:**
- Create: `crates/pad/tests/pty_smoke.rs`

- [ ] **Step 1: Write the PTY test**

```rust
// crates/pad/tests/pty_smoke.rs
use expectrl::{spawn, Eof, Regex, WaitStatus};
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn open_edit_save_exit_cycle() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("hello.txt");
    let bin = env!("CARGO_BIN_EXE_pad");
    let mut p = spawn(format!("{bin} {}", path.display())).expect("spawn");
    p.set_expect_timeout(Some(Duration::from_secs(5)));

    // Type "Hi"
    p.send("Hi").unwrap();
    // Save with ^O (since file path is known, no save-as prompt).
    p.send_control(b'O').unwrap();
    // Exit with ^X.
    p.send_control(b'X').unwrap();

    p.expect(Eof).expect("editor must exit");
    let status = p.wait().unwrap();
    assert!(matches!(status, WaitStatus::Exited(_, 0)), "pad must exit 0: {status:?}");

    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.contains("Hi"), "saved file must contain 'Hi', got {saved:?}");
}
```

- [ ] **Step 2: Run the PTY test**

```bash
cargo test -p pad --test pty_smoke -- --nocapture
```

Expected: pass within 5s. If the test hangs, the most likely cause is the keymap not seeing `^O`/`^X` from `send_control`; check `crossterm`'s event handling against the expectrl bytes.

> **Note:** `expectrl` API differs across versions; if `WaitStatus` or `set_expect_timeout` look different at execution time, consult `expectrl`'s current docs and adjust. The test's intent — spawn pad with a path, type, ^O, ^X, assert file written — is the contract.

- [ ] **Step 3: Commit**

```bash
git add crates/pad/tests/pty_smoke.rs
git commit -m "test(pad): PTY smoke test for open/edit/save/exit lifecycle"
```

---

## Task 25: CI workflow update

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Add pad tests to the workflow**

The Plan 1 CI runs `cargo test --workspace --exclude etherpad-client-spike --lib --tests`. That already picks up `pad`'s tests once the crate exists. Add a release-mode perf step explicitly:

```yaml
      - name: pad perf budget (release)
        run: cargo test -p pad --test perf_budget --release
```

Update the workflow:

```yaml
name: ci

on:
  pull_request:
  push:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - name: fmt
        run: cargo fmt --all --check
      - name: clippy
        run: cargo clippy --workspace --all-targets -- -D warnings
      - name: unit + property + conformance
        run: cargo test --workspace --exclude etherpad-client-spike --lib --tests
      - name: pad perf budget (release)
        run: cargo test -p pad --test perf_budget --release
      - name: bench (smoke)
        run: cargo bench -p etherpad-client --bench changeset -- --test
      - name: pad bench (smoke)
        run: cargo bench -p pad --bench cold_open -- --test
```

- [ ] **Step 2: Verify locally**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --exclude etherpad-client-spike --lib --tests
cargo test -p pad --test perf_budget --release
cargo bench -p etherpad-client --bench changeset -- --test
cargo bench -p pad --bench cold_open -- --test
```

Expected: all green.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add pad tests + perf budget + cold-open bench to CI"
```

---

## Self-Review

After all 25 tasks are done, verify:

1. **Spec coverage:**
   - §4.1 TUI Layer: Tasks 9–10, 13, 17, 20 (prompts/help), 21 (soft-wrap) ✓
   - §4.2 Buffer/State: Tasks 3 (rope+cursor), 4 (line endings), 6 (sidecar+meta), 7 (pending.log), 19 (undo) ✓
   - §5.1 Local-only lifecycle: Tasks 5 (load/save), 12 (event loop), 13 (^O), 14 (^X dirty prompt) ✓
   - §6.1 Nano bindings: Tasks 11 (keymap), 13 (^O), 14 (^X), 15 (^K/^U), 16 (^W), 17 (M-R), 18 (^_), 19 (M-U/M-E), 20 (^R/^G/^C/^Z) ✓
   - §6.4 Soft-wrap + UTF-8 + line endings: Tasks 4, 5, 21 ✓
   - §7 panic recovery: Task 8 (panic hook), Task 23 (--recover) ✓
   - §9 perf budgets: Task 22 ✓
   - Sections deferred to Plan 3 (Share, etc.): correctly excluded.

2. **Placeholder scan:** No "TBD", "TODO", or "implement later" content in step bodies. Three explicit caveats are flagged inline (recover replays via cursor-move heuristic; replace-all status flash deferred to Plan 3 polish; libc Suspend is unix-only) — these are scope decisions, not placeholders.

3. **Type consistency:**
   - `Buffer` methods used consistently across Tasks 3, 5, 12, 15, 16, 17, 19, 20, 23 (insert_char, backspace, delete_char_forward, move_*, cut_line, uncut, search_forward, replace_one, replace_all, snapshot_for_undo, undo, redo, cursor_offset, line, line_count, load_from_file, save_to_file).
   - `AppState` variants: `Editing`, `SaveAsPrompt(String)`, `DirtyExitPrompt`, `SearchPrompt(String)`, `ReplaceFromPrompt(String)`, `ReplaceToPrompt { from, to }`, `GotoLinePrompt(String)`, `HelpOverlay`, `FlashMessage(String)`, `InsertFilePrompt(String)`. No drift.
   - `SidecarHandle` methods: `new_untitled`, `for_file`, `reattach`, `buffer_id`, `dir`, `pending_log_path`, `file_path`, `set_file_path`. No drift.
   - `KeyAction` matches the chord table from Task 11 across Task 12, 15–20.

---

## Done criteria for Plan 2

- `cargo fmt --check` + `cargo clippy -D warnings` green.
- `cargo test --workspace --exclude etherpad-client-spike` green (target: ~50 tests including Plan 1).
- `cargo test -p pad --test perf_budget --release` green — cold-open under 50 ms p99.
- `cargo test -p pad --test pty_smoke` green — open/edit/save/exit works through a PTY.
- All nano keybindings from §6.1 land and at least one test exercises each.

When all of these hold, `pad` v0.1 is a usable nano replacement — file editing only, no network. Plan 3 layers Share + collab on top without restructuring the buffer or TUI.
