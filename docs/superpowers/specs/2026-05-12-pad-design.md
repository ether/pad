# `pad` — Design Spec

**Date:** 2026-05-12
**Status:** Draft, awaiting user review
**Repository:** `etherpad-pad`

---

## 1. Overview

`pad` is a hyper-lightweight terminal text editor designed as a drop-in nano replacement for Linux, with optional real-time collaboration backed by Etherpad. It runs entirely in the terminal, opens instantly from disk, and adds collaboration only when the user explicitly presses Share. Installable via `apt` and `snap`; the binary is called `pad`.

**Tagline:** "nano, plus a Share button."

### Invocation forms

| Command | Behavior |
|---|---|
| `pad` | Open an untitled buffer with a fresh in-memory ID. `^O` prompts for a save path nano-style; `M-S` Share generates a random pad name. |
| `pad myfile.txt` | Open/create local file `./myfile.txt`. Default pad-name-on-Share = `myfile`. |
| `pad test` | Open/create local file `./test`. Default pad-name-on-Share = `test`. |
| `pad https://etherpad.example.com/p/xyz` | Join a remote pad as a terminal collaborator. Buffer is untitled; `^O` prompts for a local save path. |
| `pad --setup` | Interactive first-run remote configuration. |
| `pad --recover` | List buffers with unsaved crash state. |
| `pad --restore` | List `pre-share-*` and `pre-merge` snapshots; open chosen one as a fresh local buffer. |

## 2. Goals & Non-Goals

### Priorities (in order)

1. **Loading speed must be faster or equal to nano.** Cold open under 50 ms to first paint.
2. **Must run in terminal.** TUI only; no GUI dependency.
3. **Must be reliable for network comms.** Editor never blocks input on network state; reconnect is automatic.
4. **Must be accurate on sync.** Changeset OT bit-exact with Etherpad's JS reference; corruption fails loudly with a recoverable snapshot rather than drifting silently.

### Non-goals (v1)

- Full nano feature parity (no syntax highlighting, multi-buffer, line numbers, soft-wrap toggle, justify, spell check, `.nanorc` parsing).
- Bundled local Etherpad server. Share is remote-only.
- Private / authenticated Etherpad pads (group + session API). v1 uses anonymous public pads with unguessable IDs.
- LAN-only / offline-LAN sharing.
- Joiners must use `pad` — web browser joining is fully supported.
- Telemetry beyond opt-in anonymous crash reports.

## 3. Architecture

### Process model

Single Rust binary (`pad`), single process, tokio async runtime. Three logical concurrent tasks:

1. **Input** — blocking stdin reader on a dedicated thread, events forwarded via channel.
2. **Sync** — `etherpad-client` socket + reconnect state machine.
3. **Render** — frame timer (60 Hz cap), redraws only on dirty.

No daemon, no subprocesses, no fork. The Etherpad backend is always external — either a remote the user configured or one picked from `scanner.etherpad.org` at first-Share.

### System shape

```
┌─────────────────────────────────────────────────────────────┐
│  pad  (single Rust binary, ~5 MB stripped)                  │
│                                                             │
│  ┌──────────────┐  ┌──────────────────┐  ┌───────────────┐  │
│  │   TUI Layer  │  │  Buffer / State  │  │ Etherpad      │  │
│  │   (ratatui   │←→│  (rope, undo,    │←→│ Client        │  │
│  │   + cross-   │  │   sidecar meta,  │  │ (socket.io +  │  │
│  │   term)      │  │   crash log)     │  │  changesets)  │  │
│  └──────────────┘  └──────────────────┘  └───────┬───────┘  │
│                                                  │          │
│  ┌──────────────────────────────────────────┐    │          │
│  │  Config / First-run / Scanner.etherpad   │    │          │
│  └──────────────────────────────────────────┘    │          │
└──────────────────────────────────────────────────┼──────────┘
                                                   │ tokio TLS websocket
                                          ┌────────▼────────┐
                                          │ Etherpad server │
                                          └─────────────────┘
```

### Workspace layout

```
etherpad-pad/
├── Cargo.toml              # workspace
├── crates/
│   └── etherpad-client/    # publishable, reusable
│       ├── src/
│       │   ├── changeset.rs
│       │   ├── socket.rs
│       │   ├── session.rs
│       │   └── lib.rs
│       └── tests/
│           └── conformance/   # fixtures + JS-reference parity
└── src/                    # the pad binary
    ├── main.rs             # arg parse, top-level event loop
    ├── tui/
    ├── buffer/
    ├── config/
    ├── share/              # M-S handler, QR, collision flow
    └── recover.rs          # pad --recover / pad --restore
```

The `etherpad-client` crate is published independently so future `etherpad-desktop` / mobile Rust rewrites can reuse it.

## 4. Components

### 4.1 TUI Layer (`src/tui/`)

- **Backend:** `crossterm` for raw mode, key events, resize, color.
- **Rendering:** `ratatui` for editor surface, status bar, prompt overlays, and the optional `M-A` author overlay.
- **Event-driven render loop:** input + sync + timer events feed a single channel; at most one render per frame.
- **First-paint target:** under 30 ms after process start (no network on open).

### 4.2 Buffer / State (`src/buffer/`)

- **Document storage:** `ropey` rope.
- **Per-buffer identity:** UUIDv7 buffer-id, minted at first open.
- **Sidecar artifacts** under `~/.local/state/pad/<buffer-id>/`:
  - `pending.log` — append-only crash-safe log, fsync per changeset.
  - `pre-share-<ISO-timestamp>.snapshot` — saved before "Edit existing" collision-flow replace.
  - `pre-merge.snapshot` — saved before applying long-divergence remote merge.
  - `meta.json` — file path if any, last share remote, last pad ID.
- **File-backed buffers** also get `.pad.<filename>.meta` next to the file (gitignored by convention; excluded from Save). Keeps buffer-id stable across renames.
- **Undo/redo:** rope-snapshot based; `M-U` undo, `M-E` redo. Buffer never blocks input on network; undo is always available locally.

### 4.3 Etherpad Client (`crates/etherpad-client/`)

Reusable Rust crate. Implements:

- **Socket.io v4 client** — likely `rust-socketio` (validated by a prerequisite spike before full implementation lands; see §11).
- **Changeset codec** — parse/serialize `Z:N>M|...` format.
- **OT primitives** — `apply`, `compose`, `inverse`, `follow`/`transform`.
- **Pad session lifecycle** — `CLIENT_READY`, author colors, cursor positions, `USER_NEWINFO` presence events.
- **Reconnect state machine** — exponential backoff (1s, 2s, 4s, 8s, capped at 30s).

Every public API call is exercised by the conformance test suite against the JS reference (see §7).

### 4.4 Config / First-run (`src/config/`)

- **Config file:** `~/.config/pad/config.json` (example shape; `remote` value is illustrative — actual default selected at first-run from `scanner.etherpad.org`, see §12 #3)
  ```json
  {
    "remote": "https://example-etherpad.org",
    "apikey": null,
    "consented_remotes": ["https://example-etherpad.org"],
    "telemetry": false
  }
  ```
- **Precedence:** CLI flag > env var (`PAD_REMOTE`, `PAD_APIKEY`, `PAD_TELEMETRY`) > config file > built-in defaults.
- **First-Share onboarding:** fetches the live community-instances list from `scanner.etherpad.org`; falls back to a baked-in list of 3–5 well-known instances if scanner is unreachable. User picks; choice persists in `consented_remotes`. Re-shows consent prompt the first time a *new* remote is used.

## 5. Data Flow — Four Lifecycles

### 5.1 Local-only editing (the 99% case)

```
pad foo.md
   │
   ▼
Parse arg → file path "./foo.md"
   │
   ▼
Mint buffer-id (UUIDv7); read .pad.foo.md.meta if it exists, else create
   │
   ▼
Open file (or empty buffer if new); detect encoding + line endings
   │
   ▼
ratatui paints first frame  (target: <30 ms from process start)
   │
   ▼
Event loop: keystroke → buffer mutation → autosave-debounce (5s/50keys) → crash log append
   │
   ▼
^O write → flush rope to file, preserving line endings
   │
   ▼
^X exit → if dirty, "Save modified buffer? [Y/N/^C]" nano-style
```

No network. `etherpad-client` linked but uninvoked.

### 5.2 First Share from a local buffer

```
M-S pressed
   │
   ▼
Have config.remote? ─── No ──► First-run prompt:
   │                          fetch scanner.etherpad.org → list
   │                          User picks → save to config.remote
   │                          + add to consented_remotes
   ▼
Connect (socket.io + TLS); attempt to bind pad-name
   ├─ buffer has filename "foo.md" → pad-name = "foo" (stripped)
   └─ untitled buffer        → pad-name = UUIDv7-base62-12char
   │
   ▼
Pad exists with content? ─── Yes ──► Collision modal:
   │                                "[E]dit existing / [D]ifferent name / [C]ancel"
   │                                  │
   │                                  └─ E: snapshot local buffer →
   │                                       replace contents w/ remote →
   │                                       attach as collaborator
   ▼
Push local buffer as initial changeset; pin author color
   │
   ▼
Render share overlay: URL + ANSI QR (qrcode-rust crate) + "Press any key to dismiss"
   │
   ▼
Status bar: "Shared • you" + author count badge
```

### 5.3 Live shared editing

```
Local keystroke
   │
   ▼
Apply to rope locally (instant feedback)
   │
   ▼
Convert edit to changeset; append to pending.log (fsync); enqueue for socket
   │
   ▼
Socket writable? ─── No ──► hold in queue, status bar: "Reconnecting (Nth try)…"
   │
   ▼
Send to Etherpad; await ACK
   │
   ▼
Remote ACK with rev → trim pending.log past that rev


Remote inbound changeset
   │
   ▼
OT-rebase against any unACKed local changesets in queue
   │
   ▼
Apply to rope; update author-cursor map; redraw
```

### 5.4 Joining via `pad <url>`

```
Parse arg → URL matches Etherpad pad pattern (http(s)://host/p/<id>)
   │
   ▼
Connect socket.io to host; subscribe to pad-id
   │
   ▼
Server sends CLIENT_READY snapshot → seed rope with full pad text
   │
   ▼
Buffer is untitled (no file path); buffer-id minted fresh
   │
   ▼
Identical to §5.3 from here on
   ^O prompts for save path nano-style (local-only; doesn't affect pad)
```

## 6. UX & Keybindings

### 6.1 Nano-faithful bindings (v1 scope)

| Binding | Action |
|---|---|
| `^O` | Write Out (save) |
| `^X` | Exit (with dirty-prompt) |
| `^R` | Insert another file |
| `^K` | Cut current line |
| `^U` | Paste / uncut |
| `^W` | Where Is (search) |
| `M-R` | Replace |
| `^_` | Goto line |
| `M-U` | Undo |
| `M-E` | Redo |
| `^G` | Help |
| `^C` | Show cursor position |
| `^Z` | Suspend to shell (nano-faithful) |
| `^S` | Unbound (avoid flow-control conflict) |

### 6.2 New collaboration bindings (`M-` namespace)

| Binding | Action |
|---|---|
| `M-S` | Share (open/refresh share overlay). When already shared, the overlay includes an `[U]nshare` action — avoids needing a separate `M-U` binding that would collide with Undo. |
| `M-A` | Toggle persistent author list overlay |
| `M-C` | Copy share URL to clipboard (via OSC 52) |
| `M-Q` | Re-display QR overlay |

### 6.3 Multi-author display

- Each remote author rendered as a colored cell at their cursor position.
- 8-color palette using xterm colors 1–7 (red, green, yellow, blue, magenta, cyan, white) deterministically picked from `hash(author-id) mod 7`. Legible without 256-color support.
- Floating name label appears for ~1 s on connect/move, then fades.
- `M-A` toggles a persistent author list overlay in the top-right corner.
- The local user's cursor stays the terminal default cursor (no special color).

### 6.4 Soft-wrap and other nano defaults

- Soft-wrap stays on (matching nano 4.0+ default). Not toggleable in v1.
- UTF-8 only; non-UTF-8 input converted on open with a status-bar note.
- Line endings preserved on save (LF/CRLF/CR detected at open).

## 7. Error Handling

The editor **never blocks input** and **never silently loses data**.

| Failure mode | Behavior |
|---|---|
| File read fails (permissions, missing dir) | Open empty buffer, status bar: `Cannot read foo.md: <reason>. Save will create it.` Editing continues. |
| File write fails on `^O` | Modal: `Save failed: <reason>. [R]etry / [P]ath... / [C]ancel`. Buffer stays dirty. |
| Disk full during crash-log append | Banner: `Disk full — crash recovery disabled until space frees.` In-memory rope authoritative. |
| Network down at first Share | Status bar: `Cannot reach <remote>: <reason>. Press M-S to retry.` Buffer unchanged. |
| Network drops mid-share | Queue locally → exponential backoff reconnect → status bar `Reconnecting (Nth try)…` → soft warning after 5 min or 1k ops queued. |
| Server returns auth error / pad gone | Demote to local-only; banner: `Lost connection to <pad-id>. Reconnect with M-S or save locally with ^O.` |
| Changeset corruption detected | Hard fail: snapshot rope to `pre-corruption.snapshot`, exit with diagnostic. Loud halt over silent drift. |
| Conformance suite parity break | Caught in CI; ship blocked. |
| `~/.config/pad/config.json` malformed | Fall back to built-in defaults; banner notes the parse error. |
| `scanner.etherpad.org` unreachable at first-Share | Baked-in fallback list of 3–5 well-known instances. |
| Panic in render or buffer code | `panic::set_hook` flushes crash-safe log, restores terminal, writes stack trace to `~/.local/state/pad/crash-<ts>.log`. |

### Specific safety net: panic + crash log

A panic anywhere in pad triggers a hook that:

1. Flushes the in-memory pending log to disk.
2. Calls `crossterm`'s terminal reset to leave the user with a usable shell.
3. Writes the panic stack + buffer-id + last-known rope hash to `~/.local/state/pad/crash-<ts>.log`.
4. Prints a single recovery hint: `Crash recovered. Run 'pad --recover' to resume.`

Result: even an OS-level kill or panic loses at most the last keystroke.

## 8. Testing Strategy

Five layers, in order of cost:

### 8.1 Unit tests (~1 s)
Rope ops, undo/redo invariants, encoding detection, config parsing, arg-disambiguation rules, QR fixture comparison.

### 8.2 Property tests (~5 s, `proptest`)
- `compose(a, b)` then `apply` ≡ `apply(a)` then `apply(b)`.
- `apply(inverse(c), apply(c, x))` ≡ `x`.
- OT convergence: `follow(a, b)` then apply `a'` after `b` ≡ `follow(b, a)` then apply `b'` after `a`.
- 10k cases per property per CI run.

### 8.3 Conformance suite (~30 s)
`tests/conformance/` directory of `(input changeset stream, expected output)` pairs captured from a real Etherpad JS client. Test harness pipes each stream through the Rust client; asserts byte-equal output. New fixtures added whenever a JS/Rust parity break is discovered.

### 8.4 Integration tests against real Etherpad (~60 s)
CI step: `docker run etherpad/etherpad` on a random port. A Rust `pad` client and a headless Chromium Etherpad client run concurrently against the same pad with randomly generated ops from both sides; final pad state asserted identical via HTTP API. Covers reconnect, collision-flow, multi-author cursor sync.

### 8.5 End-to-end terminal tests (~120 s)
PTY-driven (`expectrl` crate). Drives keystrokes, asserts rendered output. Smoke-tests: open-edit-save, M-S Share with first-run prompt, `pad <url>` join, network-drop survival.

### CI gating

All five layers run on PRs. Layers 1+2 also run in pre-commit hooks (fast).

## 9. Performance Budget

| Budget | Target | How verified |
|---|---|---|
| Cold open `pad foo.md` (warm cache) | < 20 ms to first paint | `criterion` benchmark in CI; regressions fail PR. |
| Cold open `pad foo.md` (cold cache, 100 KB file) | < 50 ms to first paint | Same benchmark with cache-drop shim. |
| Steady-state keystroke latency (shared, 100 ms RTT) | < 16 ms key→repaint p99 | Instrumented in e2e tests; p50/p99 reported. |

These are testable assertions, not aspirations. CI fails PRs that miss them.

## 10. Distribution

- **snap** — classic confinement initially (file system access); strict + plugs once feature set is stable. Standard `snapcraft.yaml` flow.
- **apt** — PPA at `ppa:etherpad/pad` (or wherever Etherpad PPAs live). Single `.deb` with stripped binary, man page (`man pad`), bash/zsh/fish completions. No runtime deps — statically link TLS via `rustls`.
- **cargo** — `cargo install pad-editor` for Rust users.
- **GitHub releases** — prebuilt binaries for macOS and Windows (tier-2 platform support).

## 11. Prerequisite Spike

Before the full implementation lands, validate `rust-socketio` (or equivalent) against a real Etherpad: ~50 lines of Rust, connect to a Docker `etherpad/etherpad`, send a single changeset, observe it apply. If the Rust socket.io ecosystem doesn't work cleanly, we discover that here rather than 2k lines into the changeset codec.

## 12. Open Questions / Known Unknowns

1. **socket.io client choice** — `rust-socketio` vs hand-rolled minimal client. Resolved by the spike in §11.
2. **Scanner.etherpad.org response shape** — needs concrete API contract before the first-run picker can be implemented. May require a small Pad-specific endpoint or a defined JSON schema agreement.
3. **`pad.etherpad.org` operator** — A3 in Q7b deferred this. If no community instance lists itself as a "pad-default" candidate, may need a small project-operated instance after all.
4. **OSC 52 clipboard reliability** — `M-C` copy-URL depends on terminal OSC 52 support. Fallback: show the URL in a clearly selectable overlay for manual copy.
5. **Stripped-down apt/snap dependency surface** — exact OpenSSL/glibc baseline for older Ubuntu LTS support needs measurement.

---

*This spec is the output of a brainstorming session conducted on 2026-05-12. The implementation plan will be drafted next via the `writing-plans` skill.*
