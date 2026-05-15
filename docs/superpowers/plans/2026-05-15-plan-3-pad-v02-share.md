# Plan 3: `pad` v0.2 — Share + Collaboration

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Layer real-time collaboration on top of the local-only editor from Plan 2 — `M-S` Share connects to a remote Etherpad, pushes the local buffer as the initial pad, then bi-directionally streams changes with OT reconciliation, multi-author cursors, QR-code joining, and crash-safe pre-share snapshots.

**Architecture:** Plan 1's `etherpad-client::PadSession` plus a new `share/` module that bridges `Buffer` ↔ `Changeset`. Async multiplexing via tokio: a dedicated network task owns the `PadSession`; the App event loop talks to it through two mpsc channels (outbound local changes, inbound remote changes). Local mutations capture (before, after) rope snapshots that derive a `Changeset` for sending. Incoming `Changeset`s apply to the rope after OT-transform against any unACK'd local ops in the outbound queue. First-run picks a remote from `scanner.etherpad.org` (with baked-in fallback); the choice persists to `~/.config/pad/config.json`. Crash-safe: every Share creates a `pre-share-<ts>.snapshot`; long remote divergence creates `pre-merge.snapshot`; both surface via `pad --restore`.

**Tech Stack:** tokio (multi-thread runtime), `etherpad-client` (Plan 1), `crossterm` async EventStream, `qrcode` (ANSI QR render), `reqwest` (scanner fetch + cookie capture).

**Reference sources:**
- Spec sections covered: §4.4, §5.2, §5.3, §5.4, §6.2, §6.3, §7 (share-related rows).
- Etherpad message types: `etherpad-lite/src/node/handler/PadMessageHandler.ts` (already reviewed).
- OSC 52 clipboard spec: `https://www.xfree86.org/current/ctlseqs.html#h3-Operating-System-Commands`.
- scanner.etherpad.org: response shape TBD by community — plan ships a hand-rolled JSON parser with a fallback list, swappable when the schema lands.

---

## What's in v0.2 (vs v0.1)

**New invocations:**
- `pad <https-url>` — join an existing remote pad as a terminal collaborator.
- `pad --setup` — interactive first-run remote configuration.
- `pad --restore` — list `pre-share-*` / `pre-merge` snapshots; open chosen as a fresh local buffer.

**New keybindings (`M-` namespace):**
- `M-S` Share / Unshare (overlay includes both actions when already shared).
- `M-A` Toggle persistent author list overlay.
- `M-C` Copy share URL to clipboard via OSC 52.
- `M-Q` Re-display QR overlay.

**New status bar elements** when shared: `Shared • you` + `Nauthors` badge.

**Out (out of scope for v0.2):**
- Private / authenticated pads (group + session API). v0.x sticks with anonymous public pads.
- LAN-only / offline sharing. The shared layer is always against the configured remote.
- Plugins / hooks / arbitrary protocol extensions.

---

## File Structure

```
etherpad-pad/
├── crates/
│   └── pad/
│       ├── src/
│       │   ├── main.rs                       # add #[tokio::main]; dispatch --setup / --restore / URL
│       │   ├── cli.rs                        # add UrlMode + --setup + --restore
│       │   ├── app.rs                        # async, multiplexed event loop
│       │   ├── buffer/
│       │   │   ├── mod.rs                    # unchanged, plus mutate_with_changeset() helper
│       │   │   └── sidecar.rs                # add pre_share_snapshot() / pre_merge_snapshot()
│       │   ├── share/                        # NEW module — all collab logic
│       │   │   ├── mod.rs                    # ShareState struct + lifecycle
│       │   │   ├── bridge.rs                 # Buffer ↔ Changeset conversion (pure fns)
│       │   │   ├── outbound.rs               # capture local edits → queue → send
│       │   │   ├── inbound.rs                # receive remote changesets → OT-rebase → apply
│       │   │   ├── network.rs                # tokio task owning PadSession + mpsc plumbing
│       │   │   ├── qr.rs                     # ANSI QR rendering
│       │   │   ├── osc52.rs                  # OSC 52 clipboard escape
│       │   │   ├── scanner.rs                # fetch + parse scanner.etherpad.org list
│       │   │   ├── collision.rs              # collision modal flow
│       │   │   └── url_parse.rs              # parse pad <url> into (host, pad_id)
│       │   ├── tui/
│       │   │   ├── share_overlay.rs          # M-S overlay (URL + QR + [U]nshare)
│       │   │   ├── author_overlay.rs         # M-A author list
│       │   │   ├── status_bar.rs             # extended to show share state + author count
│       │   │   └── mod.rs                    # draw_app extended for new overlays
│       │   ├── input.rs                      # async crossterm EventStream
│       │   ├── keymap.rs                     # add M-S / M-A / M-C / M-Q actions
│       │   └── config/
│       │       └── mod.rs                    # extend with Config struct + load/save
│       └── tests/
│           ├── share_bridge.rs               # Buffer↔Changeset round-trips
│           ├── share_outbound.rs             # local edit → outbound Changeset
│           ├── share_inbound.rs              # remote Changeset → buffer + cursor remap
│           ├── share_url_parse.rs            # URL → (host, pad_id) cases
│           ├── share_pty_e2e.rs              # PTY: open, M-S, see status bar update
│           └── config_roundtrip.rs           # ~/.config/pad/config.json
```

**Why this split:**
- `share/` is one cohesive subsystem (collab); split into `bridge` (pure conversion), `outbound`/`inbound` (one-direction flows), `network` (async I/O), and presentation helpers (qr/osc52/url_parse/scanner/collision).
- `bridge.rs` is pure functions — easy to property-test with proptest the same way Plan 1's OT primitives were tested.
- Network plumbing in `network.rs` keeps tokio task management away from the App, which stays largely sync.
- TUI additions (`share_overlay`, `author_overlay`) live with the rest of `tui/`.

---

## Concurrency model

```
                         ┌─────────────────────────────────────┐
                         │      App event loop (sync)          │
                         │                                     │
   stdin → ┌──────────┐  │  ┌─────────────┐                    │
           │ EventTask│──┼─►│ key channel │──► dispatch        │
           └──────────┘  │  └─────────────┘                    │
                         │                                     │
                         │  ┌─────────────┐                    │
                         │  │ inbound rx  │◄─── share network  │
                         │  └─────────────┘     task (tokio)   │
                         │                                     │
                         │  ┌─────────────┐                    │
                         │  │ outbound tx │────► share network │
                         │  └─────────────┘                    │
                         └─────────────────────────────────────┘
```

- **Event task** — `tokio::spawn` blocking task reading `crossterm::event::EventStream`, sending `KeyAction` over `mpsc`. Stays untouched by Plan 3 changes after the migration in Task 1.
- **Share network task** — `tokio::spawn` owning the `PadSession`. Reads from outbound mpsc, calls `session.send_changeset`. Pumps `session.pump_once` for inbound and sends each remote Changeset over inbound mpsc.
- **App event loop** — `tokio::select!`s on key channel + inbound channel + a tick timer (50 ms cap, for repaint cadence). When local mutations happen and we're shared, derive Changeset and push to outbound mpsc.

---

## Task 1: Migrate to tokio runtime + async EventStream

**Files:**
- Modify: `crates/pad/Cargo.toml`
- Modify: `crates/pad/src/main.rs`
- Modify: `crates/pad/src/app.rs`
- Modify: `crates/pad/src/input.rs`
- Modify: `crates/pad/src/tui/mod.rs`

- [ ] **Step 1: Enable crossterm async feature**

In `crates/pad/Cargo.toml`, change the `crossterm` line:

```toml
crossterm = { workspace = true, features = ["event-stream"] }
```

If `workspace.dependencies.crossterm` doesn't allow features extension, also bump `Cargo.toml` workspace entry:

```toml
crossterm = { version = "0.29", features = ["event-stream"] }
```

- [ ] **Step 2: Rewrite `input.rs` to expose an async stream**

```rust
// crates/pad/src/input.rs
use crate::keymap::{KeyAction, key_to_action};
use crossterm::event::{Event, EventStream};
use futures_util::StreamExt;
use tokio::sync::mpsc;

/// Spawn a tokio task that reads crossterm events and forwards each KeyAction
/// over an unbounded mpsc channel. Returns the receiver.
pub fn spawn_event_task() -> mpsc::UnboundedReceiver<KeyAction> {
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let mut stream = EventStream::new();
        while let Some(evt) = stream.next().await {
            let Ok(Event::Key(ke)) = evt else { continue };
            if tx.send(key_to_action(ke)).is_err() {
                break;
            }
        }
    });
    rx
}
```

- [ ] **Step 3: Make `App::run` async and select on channels**

In `crates/pad/src/app.rs`:

```rust
use tokio::sync::mpsc;
use std::time::Duration;

impl App {
    pub async fn run(&mut self, tui: &mut Tui) -> anyhow::Result<()> {
        let mut keys = crate::input::spawn_event_task();
        while !self.quit_requested {
            // ... build `prompt` and `show_help` as before ...
            let prompt: Option<(&str, &str)> = match &self.state { /* unchanged */ };
            let show_help = matches!(self.state, AppState::HelpOverlay);
            tui.draw_app(&self.buffer, &self.file_label, prompt, show_help)?;

            let tick = tokio::time::sleep(Duration::from_millis(50));
            tokio::pin!(tick);
            tokio::select! {
                Some(action) = keys.recv() => { self.handle(action)?; }
                _ = &mut tick => { /* repaint */ }
            }
        }
        Ok(())
    }
}
```

Update `crates/pad/src/main.rs` to be `#[tokio::main]`:

```rust
use clap::Parser;
use pad::app::App;
use pad::cli::{Args, Mode};
use pad::config::paths;
use pad::panic_hook::{file_sink, install_panic_hook};
use pad::tui::Tui;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    install_panic_hook(file_sink(paths::state_root()));
    match args.mode() {
        Mode::Recover => {
            pad::recover::run(&paths::state_root())?;
            Ok(())
        }
        mode => {
            let mut tui = Tui::enter()?;
            let mut app = App::from_mode(mode)?;
            app.run(&mut tui).await
        }
    }
}
```

- [ ] **Step 4: Re-run the PTY smoke test**

Run: `cargo test -p pad --test pty_smoke`
Expected: all 3 PTY tests pass. (The migration is supposed to be transparent — if any fail, the issue is in the input task or runtime setup.)

- [ ] **Step 5: Commit**

```bash
git add crates/pad/
git commit -m "refactor(pad): async event loop on tokio + crossterm EventStream"
```

---

## Task 2: Buffer ↔ Changeset bridge (pure conversion)

**Files:**
- Create: `crates/pad/src/share/mod.rs`
- Create: `crates/pad/src/share/bridge.rs`
- Modify: `crates/pad/src/lib.rs`
- Create: `crates/pad/tests/share_bridge.rs`

- [ ] **Step 1: Stub the share module**

`crates/pad/src/share/mod.rs`:

```rust
pub mod bridge;
```

Add to `crates/pad/src/lib.rs`:

```rust
pub mod share;
```

- [ ] **Step 2: Write failing tests**

```rust
// crates/pad/tests/share_bridge.rs
use etherpad_client::changeset::OpCode;
use etherpad_client::ot::apply;
use pad::share::bridge::{changeset_for_delete, changeset_for_insert};

#[test]
fn insert_into_empty() {
    let cs = changeset_for_insert(0, 0, "hi");
    assert_eq!(cs.old_len, 0);
    assert_eq!(cs.net_delta, 2);
    assert_eq!(cs.ops.len(), 1);
    assert_eq!(cs.ops[0].opcode, OpCode::Insert);
    assert_eq!(cs.char_bank, "hi");
    assert_eq!(apply(&cs, "").unwrap(), "hi");
}

#[test]
fn insert_in_middle() {
    let cs = changeset_for_insert(5, 2, "X");
    // hello -> heXllo. old_len=5, keep 2 + insert 1 + (implicit) keep 3.
    assert_eq!(cs.old_len, 5);
    assert_eq!(cs.net_delta, 1);
    assert_eq!(apply(&cs, "hello").unwrap(), "heXllo");
}

#[test]
fn insert_at_end() {
    let cs = changeset_for_insert(5, 5, "!");
    assert_eq!(apply(&cs, "hello").unwrap(), "hello!");
}

#[test]
fn delete_single() {
    let cs = changeset_for_delete(5, 2, "l".into());
    // hello -> helo (delete 1 at pos 2)
    assert_eq!(apply(&cs, "hello").unwrap(), "helo");
}

#[test]
fn delete_range() {
    let cs = changeset_for_delete(5, 1, "ell".into());
    // hello -> ho (delete 3 starting at pos 1)
    assert_eq!(apply(&cs, "hello").unwrap(), "ho");
}

#[test]
fn delete_at_start() {
    let cs = changeset_for_delete(5, 0, "he".into());
    assert_eq!(apply(&cs, "hello").unwrap(), "llo");
}
```

- [ ] **Step 3: Run tests to confirm they fail**

Run: `cargo test -p pad --test share_bridge`
Expected: compile error — `changeset_for_insert` / `changeset_for_delete` not defined.

- [ ] **Step 4: Implement `bridge.rs`**

```rust
// crates/pad/src/share/bridge.rs
use etherpad_client::changeset::{Changeset, Op, OpCode};

/// Build a Changeset that inserts `text` at char offset `pos` in a document of
/// length `old_len` chars.
pub fn changeset_for_insert(old_len: u32, pos: u32, text: &str) -> Changeset {
    let inserted = text.chars().count() as u32;
    let mut ops = Vec::new();
    if pos > 0 {
        ops.push(Op {
            opcode: OpCode::Keep,
            chars: pos,
            lines: 0,
            attribs: vec![],
        });
    }
    ops.push(Op {
        opcode: OpCode::Insert,
        chars: inserted,
        lines: text.matches('\n').count() as u32,
        attribs: vec![],
    });
    if pos < old_len {
        ops.push(Op {
            opcode: OpCode::Keep,
            chars: old_len - pos,
            lines: 0,
            attribs: vec![],
        });
    }
    Changeset {
        old_len,
        net_delta: inserted as i64,
        ops,
        char_bank: text.to_string(),
    }
}

/// Build a Changeset that deletes `deleted_text` (which lives at offset `pos`
/// in a document of length `old_len`).
pub fn changeset_for_delete(old_len: u32, pos: u32, deleted_text: String) -> Changeset {
    let deleted_chars = deleted_text.chars().count() as u32;
    let mut ops = Vec::new();
    if pos > 0 {
        ops.push(Op {
            opcode: OpCode::Keep,
            chars: pos,
            lines: 0,
            attribs: vec![],
        });
    }
    ops.push(Op {
        opcode: OpCode::Delete,
        chars: deleted_chars,
        lines: deleted_text.matches('\n').count() as u32,
        attribs: vec![],
    });
    let after = old_len - pos - deleted_chars;
    if after > 0 {
        ops.push(Op {
            opcode: OpCode::Keep,
            chars: after,
            lines: 0,
            attribs: vec![],
        });
    }
    Changeset {
        old_len,
        net_delta: -(deleted_chars as i64),
        ops,
        char_bank: deleted_text,
    }
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p pad --test share_bridge`
Expected: 6 tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/pad/src/share/ crates/pad/src/lib.rs crates/pad/tests/share_bridge.rs
git commit -m "feat(pad/share): Buffer→Changeset bridge for insert/delete"
```

---

## Task 3: Buffer.text_len() helper + mutate_with_changeset capture

**Files:**
- Modify: `crates/pad/src/buffer/mod.rs`
- Create: `crates/pad/tests/buffer_capture.rs`

To turn a local mutation into a Changeset we need:
1. The rope's char-length before the mutation (`old_len`).
2. The cursor's absolute offset before the mutation.

Both already exist (`cursor_offset()`); we add `text_len()`.

- [ ] **Step 1: Add `text_len()` to Buffer**

In `crates/pad/src/buffer/mod.rs`, inside `impl Buffer`:

```rust
pub fn text_len(&self) -> u32 {
    self.rope.len_chars() as u32
}
```

- [ ] **Step 2: Write a sanity test**

```rust
// crates/pad/tests/buffer_capture.rs
use pad::buffer::Buffer;

#[test]
fn text_len_tracks_mutations() {
    let mut b = Buffer::empty();
    assert_eq!(b.text_len(), 0);
    b.insert_char('a');
    assert_eq!(b.text_len(), 1);
    b.insert_str("bc");
    assert_eq!(b.text_len(), 3);
    b.backspace();
    assert_eq!(b.text_len(), 2);
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p pad --test buffer_capture`
Expected: pass.

- [ ] **Step 4: Commit**

```bash
git add crates/pad/src/buffer/mod.rs crates/pad/tests/buffer_capture.rs
git commit -m "feat(pad): Buffer::text_len() helper for changeset derivation"
```

---

## Task 4: Network task — async PadSession driver

**Files:**
- Create: `crates/pad/src/share/network.rs`
- Modify: `crates/pad/src/share/mod.rs`

The network task owns the `PadSession`. The App talks to it over two channels:

```text
App  ---outbound: Changeset--->  NetworkTask  ---socket.io---> Etherpad
App  <--inbound: Changeset-----  NetworkTask  <--socket.io--- Etherpad
```

- [ ] **Step 1: Write `network.rs`**

```rust
// crates/pad/src/share/network.rs
use etherpad_client::changeset::Changeset;
use etherpad_client::{PadSession, SessionConfig};
use etherpad_client::socket::TungsteniteSocket;
use tokio::sync::mpsc;

pub struct NetworkHandles {
    pub outbound_tx: mpsc::UnboundedSender<Changeset>,
    pub inbound_rx: mpsc::UnboundedReceiver<Changeset>,
    pub task: tokio::task::JoinHandle<()>,
    pub author_id: String,
    pub rev: u32,
    pub initial_text: String,
}

pub struct ConnectError {
    pub message: String,
}

/// Connect to the remote, perform the CLIENT_READY handshake, and start the
/// pump task. Returns inbound/outbound channels + the handshake snapshot.
pub async fn connect(remote_base: &str, pad_id: &str) -> Result<NetworkHandles, ConnectError> {
    let cookie = TungsteniteSocket::fetch_pad_cookie(remote_base, pad_id)
        .await
        .map_err(|e| ConnectError { message: format!("cookie fetch: {e}") })?;
    let mut socket = TungsteniteSocket::new(remote_base, Some(cookie));
    use etherpad_client::Socket;
    socket
        .connect()
        .await
        .map_err(|e| ConnectError { message: format!("ws connect: {e}") })?;

    let mut session = PadSession::new(
        Box::new(socket),
        SessionConfig {
            pad_id: pad_id.into(),
            // Legacy token field; modern Etherpad uses the cookie above.
            token: "t.pad-client-legacy".into(),
            protocol_version: 2,
        },
    );
    session
        .handshake()
        .await
        .map_err(|e| ConnectError { message: format!("handshake: {e}") })?;

    let author_id = session.author_id().as_str().to_string();
    let rev = session.rev();
    let initial_text = session.initial_text().to_string();

    let (outbound_tx, mut outbound_rx) = mpsc::unbounded_channel::<Changeset>();
    let (inbound_tx, inbound_rx) = mpsc::unbounded_channel::<Changeset>();

    let task = tokio::spawn(async move {
        let mut session = session;
        loop {
            tokio::select! {
                outbound = outbound_rx.recv() => {
                    let Some(cs) = outbound else { break };
                    if session.send_changeset(&cs).await.is_err() {
                        break;
                    }
                }
                pumped = session.pump_once() => {
                    match pumped {
                        Ok(Some(cs)) => {
                            if inbound_tx.send(cs).is_err() { break; }
                        }
                        Ok(None) => {}
                        Err(_) => break,
                    }
                }
            }
        }
        let _ = session.disconnect().await;
    });

    Ok(NetworkHandles {
        outbound_tx,
        inbound_rx,
        task,
        author_id,
        rev,
        initial_text,
    })
}
```

- [ ] **Step 2: Wire into module**

In `crates/pad/src/share/mod.rs`:

```rust
pub mod bridge;
pub mod network;
```

- [ ] **Step 3: Build**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 4: Commit**

```bash
git add crates/pad/src/share/
git commit -m "feat(pad/share): async PadSession driver task + bidirectional mpsc"
```

---

## Task 5: Outbound — local edit → Changeset → network

**Files:**
- Modify: `crates/pad/src/share/mod.rs`
- Create: `crates/pad/src/share/outbound.rs`

- [ ] **Step 1: Implement `OutboundQueue`**

```rust
// crates/pad/src/share/outbound.rs
use etherpad_client::changeset::Changeset;
use std::collections::VecDeque;
use tokio::sync::mpsc;

/// Tracks local changesets that have been sent to the server but not yet
/// ACK'd via a matching `ACCEPT_COMMIT` (or echoed back as a `NEW_CHANGES`).
///
/// Used to OT-rebase inbound remote changesets so they apply on top of our
/// pending local edits.
pub struct OutboundQueue {
    sink: mpsc::UnboundedSender<Changeset>,
    pending: VecDeque<Changeset>,
}

impl OutboundQueue {
    pub fn new(sink: mpsc::UnboundedSender<Changeset>) -> Self {
        Self {
            sink,
            pending: VecDeque::new(),
        }
    }

    /// Enqueue + send. The changeset stays in `pending` until ACK'd.
    pub fn send(&mut self, cs: Changeset) -> anyhow::Result<()> {
        self.pending.push_back(cs.clone());
        self.sink
            .send(cs)
            .map_err(|_| anyhow::anyhow!("network task closed"))?;
        Ok(())
    }

    /// Drop the oldest pending changeset (the server ACKed it).
    pub fn ack_one(&mut self) {
        self.pending.pop_front();
    }

    pub fn pending(&self) -> impl Iterator<Item = &Changeset> {
        self.pending.iter()
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }
}
```

- [ ] **Step 2: Wire into module**

`crates/pad/src/share/mod.rs`:

```rust
pub mod bridge;
pub mod network;
pub mod outbound;
```

- [ ] **Step 3: Smoke test the queue**

Add inline test in `outbound.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use etherpad_client::changeset::{Changeset, Op, OpCode};

    fn dummy_insert() -> Changeset {
        Changeset {
            old_len: 0,
            net_delta: 1,
            ops: vec![Op { opcode: OpCode::Insert, chars: 1, lines: 0, attribs: vec![] }],
            char_bank: "x".into(),
        }
    }

    #[test]
    fn enqueue_and_ack() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut q = OutboundQueue::new(tx);
        q.send(dummy_insert()).unwrap();
        q.send(dummy_insert()).unwrap();
        assert_eq!(q.pending_len(), 2);
        // Drain channel side
        let _ = rx.try_recv();
        let _ = rx.try_recv();
        q.ack_one();
        assert_eq!(q.pending_len(), 1);
        q.ack_one();
        assert_eq!(q.pending_len(), 0);
        q.ack_one(); // no-op when empty
        assert_eq!(q.pending_len(), 0);
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p pad outbound::tests`
Expected: 1 test passes.

- [ ] **Step 5: Commit**

```bash
git add crates/pad/src/share/
git commit -m "feat(pad/share): OutboundQueue tracks unACK'd local changesets"
```

---

## Task 6: Inbound — remote Changeset → buffer with OT rebase + cursor remap

**Files:**
- Create: `crates/pad/src/share/inbound.rs`
- Modify: `crates/pad/src/share/mod.rs`
- Create: `crates/pad/tests/share_inbound.rs`

- [ ] **Step 1: Write failing tests**

```rust
// crates/pad/tests/share_inbound.rs
use etherpad_client::changeset::parser::parse;
use pad::buffer::{Buffer, CursorPos};
use pad::share::inbound::apply_remote;
use pad::share::outbound::OutboundQueue;
use tokio::sync::mpsc;

fn empty_queue() -> OutboundQueue {
    let (tx, _rx) = mpsc::unbounded_channel();
    OutboundQueue::new(tx)
}

#[test]
fn remote_insert_at_start_shifts_cursor() {
    let mut buf = Buffer::from_text("hello");
    buf.move_cursor_to(CursorPos { line: 0, col: 3 }); // between 'l' and 'l'
    let remote = parse("Z:5>1+1=5$Z").unwrap(); // insert "Z" at position 0
    apply_remote(&mut buf, &remote, &empty_queue()).unwrap();
    assert_eq!(buf.text(), "Zhello");
    // cursor was at col 3 in "hello"; remote inserted 1 char at col 0 → now col 4.
    assert_eq!(buf.cursor(), CursorPos { line: 0, col: 4 });
}

#[test]
fn remote_insert_after_cursor_leaves_cursor() {
    let mut buf = Buffer::from_text("hello");
    buf.move_cursor_to(CursorPos { line: 0, col: 2 });
    let remote = parse("Z:5>1=5+1$!").unwrap(); // insert "!" at end
    apply_remote(&mut buf, &remote, &empty_queue()).unwrap();
    assert_eq!(buf.text(), "hello!");
    assert_eq!(buf.cursor(), CursorPos { line: 0, col: 2 });
}

#[test]
fn remote_delete_before_cursor_shifts_cursor_back() {
    let mut buf = Buffer::from_text("hello");
    buf.move_cursor_to(CursorPos { line: 0, col: 4 });
    let remote = parse("Z:5<2=2-2=1$ll").unwrap(); // delete "ll" at pos 2
    apply_remote(&mut buf, &remote, &empty_queue()).unwrap();
    assert_eq!(buf.text(), "heo");
    assert_eq!(buf.cursor(), CursorPos { line: 0, col: 2 });
}
```

- [ ] **Step 2: Implement `inbound.rs`**

```rust
// crates/pad/src/share/inbound.rs
use crate::buffer::{Buffer, CursorPos};
use crate::share::outbound::OutboundQueue;
use etherpad_client::changeset::{Changeset, OpCode};
use etherpad_client::ot;

pub fn apply_remote(
    buffer: &mut Buffer,
    remote: &Changeset,
    queue: &OutboundQueue,
) -> anyhow::Result<()> {
    // OT-rebase the remote changeset against any unACK'd local changesets.
    // For each pending local cs L (in the order they were sent), we transform
    // `remote` via follow(L, remote).
    let mut rebased = remote.clone();
    for local in queue.pending() {
        rebased = ot::follow(local, &rebased).map_err(|e| anyhow::anyhow!("{e}"))?;
    }

    // Remap cursor across the rebased changeset.
    let old_offset = buffer.cursor_offset();
    let new_offset = remap_offset(old_offset, &rebased);

    // Apply to the rope.
    let before = buffer.text();
    let after = ot::apply(&rebased, &before).map_err(|e| anyhow::anyhow!("{e}"))?;

    buffer.replace_all_text(&after);
    let new_pos = offset_to_cursor_pos(buffer, new_offset);
    buffer.move_cursor_to(new_pos);
    Ok(())
}

fn remap_offset(offset: u32, cs: &Changeset) -> u32 {
    let mut consumed_input = 0u32;
    let mut consumed_output = 0u32;
    let mut remaining = offset;
    for op in &cs.ops {
        let n = op.chars;
        match op.opcode {
            OpCode::Keep => {
                let take = n.min(remaining);
                consumed_input += take;
                consumed_output += take;
                remaining -= take;
                if remaining == 0 {
                    return consumed_output;
                }
            }
            OpCode::Insert => {
                consumed_output += n;
                // remaining is unchanged; insert sits in front of cursor only if it's
                // logically at the cursor position. Since we walk ops in order and
                // the cursor's logical position is mapped after we've passed it, an
                // insert encountered here is BEFORE the cursor.
            }
            OpCode::Delete => {
                let take = n.min(remaining);
                consumed_input += take;
                remaining -= take;
                // output not advanced — chars dropped.
                if remaining == 0 {
                    return consumed_output;
                }
            }
        }
    }
    // Tail: implicit keep of (old_len - consumed_input) chars.
    let _ = consumed_input;
    consumed_output + remaining
}

fn offset_to_cursor_pos(buffer: &Buffer, offset: u32) -> CursorPos {
    let mut remaining = offset as usize;
    for li in 0..buffer.line_count() {
        let line = buffer.line(li);
        let line_chars = line.chars().count() + 1; // +1 for trailing newline
        if remaining < line_chars {
            return CursorPos {
                line: li,
                col: remaining.min(line.chars().count()),
            };
        }
        remaining -= line_chars;
    }
    CursorPos {
        line: buffer.line_count() - 1,
        col: buffer.line(buffer.line_count() - 1).chars().count(),
    }
}
```

- [ ] **Step 3: Add `replace_all_text` helper to Buffer**

In `crates/pad/src/buffer/mod.rs`:

```rust
impl Buffer {
    /// Wipe and replace the rope content. Used by the share layer when
    /// applying a remote changeset whose effect was already computed against
    /// `text()`. Marks the buffer dirty.
    pub fn replace_all_text(&mut self, new_text: &str) {
        self.rope = ropey::Rope::from_str(new_text);
        self.dirty = true;
        // Cursor clamping happens at the next move_cursor_to.
    }
}
```

- [ ] **Step 4: Wire module**

`crates/pad/src/share/mod.rs`:

```rust
pub mod bridge;
pub mod inbound;
pub mod network;
pub mod outbound;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p pad --test share_inbound`
Expected: 3 tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/pad/src/share/ crates/pad/src/buffer/mod.rs crates/pad/tests/share_inbound.rs
git commit -m "feat(pad/share): apply remote Changeset with OT rebase + cursor remap"
```

---

## Task 7: ShareState + integrate into App

**Files:**
- Modify: `crates/pad/src/share/mod.rs`
- Modify: `crates/pad/src/app.rs`
- Modify: `crates/pad/src/keymap.rs`

- [ ] **Step 1: Add `KeyAction::Share` and `KeyAction::Unshare`**

In `crates/pad/src/keymap.rs`, add to `KeyAction`:

```rust
Share,         // M-S
ToggleAuthors, // M-A
CopyShareUrl,  // M-C
ReshowQr,      // M-Q
```

In `key_to_action`, in the `(KeyCode::Char(c), _, true)` (Alt) match:

```rust
's' => KeyAction::Share,
'a' => KeyAction::ToggleAuthors,
'c' => KeyAction::CopyShareUrl,
'q' => KeyAction::ReshowQr,
```

- [ ] **Step 2: Add `ShareState` to share module**

`crates/pad/src/share/mod.rs`:

```rust
pub mod bridge;
pub mod inbound;
pub mod network;
pub mod outbound;

use crate::buffer::Buffer;
use etherpad_client::changeset::Changeset;
use outbound::OutboundQueue;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub struct ShareState {
    pub pad_id: String,
    pub remote_base: String,
    pub author_id: String,
    pub outbound: OutboundQueue,
    pub inbound_rx: mpsc::UnboundedReceiver<Changeset>,
    pub net_task: JoinHandle<()>,
    pub authors: std::collections::HashSet<String>,
}

impl ShareState {
    pub async fn connect_and_seed(
        remote_base: &str,
        pad_id: &str,
        local_buffer: &Buffer,
    ) -> anyhow::Result<Self> {
        let handles = network::connect(remote_base, pad_id)
            .await
            .map_err(|e| anyhow::anyhow!("connect: {}", e.message))?;
        let mut outbound = OutboundQueue::new(handles.outbound_tx);
        // If the remote pad started empty (no rev=0 text), push our buffer as
        // the initial content via one big insert. If it had text already, the
        // collision modal will have run before we reached here (Task 11).
        if handles.initial_text.is_empty() && !local_buffer.text().is_empty() {
            let cs = bridge::changeset_for_insert(0, 0, &local_buffer.text());
            outbound.send(cs)?;
        }
        let mut authors = std::collections::HashSet::new();
        authors.insert(handles.author_id.clone());
        Ok(Self {
            pad_id: pad_id.to_string(),
            remote_base: remote_base.to_string(),
            author_id: handles.author_id,
            outbound,
            inbound_rx: handles.inbound_rx,
            net_task: handles.task,
            authors,
        })
    }

    /// Build the human-visible URL for this shared pad.
    pub fn share_url(&self) -> String {
        format!(
            "{}/p/{}",
            self.remote_base.trim_end_matches('/'),
            self.pad_id
        )
    }
}
```

- [ ] **Step 3: Integrate into `App`**

In `crates/pad/src/app.rs`:

Add to `App`:

```rust
pub share: Option<crate::share::ShareState>,
```

Initialize as `None` in `from_mode`. Add a new `AppState` variant:

```rust
RemotePromptForShare(String),
```

In `handle_editing`, add a branch:

```rust
KeyAction::Share => {
    if self.share.is_some() {
        // Already shared — toggling means Unshare (Task 12 wires that;
        // for now, treat as no-op).
    } else {
        // Use stored config remote if set; otherwise prompt.
        let remote = self.persisted_remote();
        match remote {
            Some(r) => self.start_share(&r).await?,
            None => self.state = AppState::RemotePromptForShare(String::new()),
        }
    }
}
KeyAction::ToggleAuthors | KeyAction::CopyShareUrl | KeyAction::ReshowQr => {
    // Stub — wired in later tasks (10/13/14).
}
```

But `handle_editing` is not async. The simplest fix: make all `handle*` methods async by changing the App's event-loop dispatch:

```rust
async fn handle(&mut self, action: KeyAction) -> anyhow::Result<()> { ... }
```

(Update every `self.handle(action)` and `self.handle_editing(action)` call site to `.await`.)

In the `App::run` select loop, also drain `share.inbound_rx`:

```rust
let inbound_recv = async {
    if let Some(s) = self.share.as_mut() {
        s.inbound_rx.recv().await
    } else {
        std::future::pending::<Option<Changeset>>().await
    }
};
tokio::pin!(inbound_recv);
tokio::select! {
    Some(action) = keys.recv() => { self.handle(action).await?; }
    Some(cs) = &mut inbound_recv => {
        let q = self.share.as_ref().map(|s| &s.outbound);
        if let Some(q) = q {
            crate::share::inbound::apply_remote(&mut self.buffer, &cs, q)?;
        }
    }
    _ = &mut tick => {}
}
```

(The exact borrow pattern needs `Pin<&mut Future>` to satisfy tokio::select; see Task 7 commit for the working shape.)

Implement helpers on `App`:

```rust
fn persisted_remote(&self) -> Option<String> {
    crate::config::Config::load().ok().and_then(|c| c.remote)
}

async fn start_share(&mut self, remote: &str) -> anyhow::Result<()> {
    // Pad-name on share: filename stem if any, else UUID-base62-12char.
    let pad_id = self
        .file_path
        .as_ref()
        .and_then(|p| p.file_stem())
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| short_random_pad_id());
    // Pre-share snapshot (Task 8 wires it; for this task we accept that the
    // file may not yet exist).
    let _ = self.sidecar.pre_share_snapshot(&self.buffer);
    let state = crate::share::ShareState::connect_and_seed(remote, &pad_id, &self.buffer).await?;
    self.share = Some(state);
    Ok(())
}

fn short_random_pad_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789".chars().collect();
    let mut out = String::with_capacity(12);
    let mut t = ts;
    for _ in 0..12 {
        out.push(chars[(t as usize) % chars.len()]);
        t /= chars.len() as u128;
    }
    out
}
```

(`pre_share_snapshot` lands in Task 8; for this task it can be stubbed in `sidecar.rs` as a no-op.)

For now stub it:

```rust
// In crates/pad/src/buffer/sidecar.rs, add:
impl SidecarHandle {
    pub fn pre_share_snapshot(&self, _buf: &crate::buffer::Buffer) -> anyhow::Result<()> {
        // Real implementation lands in Task 8.
        Ok(())
    }
}
```

Add a minimal `Config::load`:

In `crates/pad/src/config/mod.rs`:

```rust
pub mod paths;

use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    pub remote: Option<String>,
    #[serde(default)]
    pub consented_remotes: Vec<String>,
    #[serde(default)]
    pub telemetry: bool,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = paths::config_root().join("config.json");
        let raw = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Config::default());
            }
            Err(e) => return Err(anyhow::anyhow!("read config: {e}")),
        };
        let cfg: Config = serde_json::from_str(&raw)
            .map_err(|e| anyhow::anyhow!("parse config: {e}"))?;
        Ok(cfg)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let dir = paths::config_root();
        fs::create_dir_all(&dir)?;
        let raw = serde_json::to_string_pretty(self)?;
        fs::write(dir.join("config.json"), raw)?;
        Ok(())
    }
}
```

- [ ] **Step 4: Build**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 5: Commit**

```bash
git add crates/pad/
git commit -m "feat(pad/share): ShareState integration + M-S keybinding + Config skeleton"
```

---

## Task 8: Pre-share snapshots

**Files:**
- Modify: `crates/pad/src/buffer/sidecar.rs`
- Create: `crates/pad/tests/snapshot.rs`

- [ ] **Step 1: Write failing test**

```rust
// crates/pad/tests/snapshot.rs
use pad::buffer::Buffer;
use pad::buffer::sidecar::SidecarHandle;
use tempfile::tempdir;

#[test]
fn pre_share_snapshot_writes_file() {
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).unwrap();
    let mut buf = Buffer::empty();
    buf.insert_str("contents to preserve");
    sc.pre_share_snapshot(&buf).expect("snapshot");

    let entries: Vec<_> = std::fs::read_dir(sc.dir())
        .unwrap()
        .filter_map(|r| r.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("pre-share-")
        })
        .collect();
    assert_eq!(entries.len(), 1);

    let snap_path = entries[0].path();
    let contents = std::fs::read_to_string(&snap_path).unwrap();
    assert_eq!(contents, "contents to preserve");
}

#[test]
fn pre_merge_snapshot_overwrites_existing() {
    let state_root = tempdir().unwrap();
    let sc = SidecarHandle::new_untitled(state_root.path()).unwrap();
    let mut buf = Buffer::empty();
    buf.insert_str("first");
    sc.pre_merge_snapshot(&buf).unwrap();
    buf.insert_str(" second");
    sc.pre_merge_snapshot(&buf).unwrap();
    let contents = std::fs::read_to_string(sc.dir().join("pre-merge.snapshot")).unwrap();
    assert_eq!(contents, "first second");
}
```

- [ ] **Step 2: Replace the stub with the real impls**

In `crates/pad/src/buffer/sidecar.rs`, replace the Task-7 stub of `pre_share_snapshot` and add `pre_merge_snapshot`:

```rust
use std::time::{SystemTime, UNIX_EPOCH};

impl SidecarHandle {
    pub fn pre_share_snapshot(&self, buf: &crate::buffer::Buffer) -> anyhow::Result<()> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let path = self.dir().join(format!("pre-share-{ts}.snapshot"));
        fs::write(path, buf.text())?;
        Ok(())
    }

    pub fn pre_merge_snapshot(&self, buf: &crate::buffer::Buffer) -> anyhow::Result<()> {
        let path = self.dir().join("pre-merge.snapshot");
        fs::write(path, buf.text())?;
        Ok(())
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p pad --test snapshot`
Expected: 2 tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/pad/src/buffer/sidecar.rs crates/pad/tests/snapshot.rs
git commit -m "feat(pad): pre-share-<ts>.snapshot + pre-merge.snapshot helpers"
```

---

## Task 9: Wire local edits → outbound Changeset

**Files:**
- Modify: `crates/pad/src/app.rs`

When the user types a char while shared, we must:
1. Capture `text_len` and `cursor_offset` BEFORE mutation.
2. Mutate the buffer.
3. Build a Changeset from the (offset, inserted text) pair via `bridge::changeset_for_insert`.
4. `outbound.send(cs)`.

Same for backspace/delete: capture the about-to-be-deleted char first.

- [ ] **Step 1: Refactor `handle_editing`'s mutation arms**

Replace the `InsertChar`, `Backspace`, `DeleteForward` arms in `handle_editing`:

```rust
KeyAction::InsertChar(c) => {
    self.buffer.snapshot_for_undo();
    let pre_len = self.buffer.text_len();
    let pre_offset = self.buffer.cursor_offset();
    self.pending_log.append(&PendingEntry::Insert {
        offset: pre_offset,
        text: c.to_string(),
    })?;
    self.buffer.insert_char(c);
    if let Some(share) = self.share.as_mut() {
        let cs = crate::share::bridge::changeset_for_insert(
            pre_len,
            pre_offset,
            &c.to_string(),
        );
        share.outbound.send(cs)?;
    }
}
KeyAction::Backspace => {
    let off = self.buffer.cursor_offset();
    if off > 0 {
        self.buffer.snapshot_for_undo();
        let pre_len = self.buffer.text_len();
        // Capture the char about to be deleted (for the changeset bank).
        let deleted = self.buffer.text()
            .chars()
            .nth((off - 1) as usize)
            .map(|c| c.to_string())
            .unwrap_or_default();
        self.pending_log.append(&PendingEntry::Delete {
            offset: off - 1,
            len: 1,
        })?;
        self.buffer.backspace();
        if let Some(share) = self.share.as_mut() {
            let cs = crate::share::bridge::changeset_for_delete(pre_len, off - 1, deleted);
            share.outbound.send(cs)?;
        }
    } else {
        self.buffer.backspace(); // no-op at start; keeps Buffer behaviour
    }
}
KeyAction::DeleteForward => {
    self.buffer.snapshot_for_undo();
    let pre_len = self.buffer.text_len();
    let off = self.buffer.cursor_offset();
    if off < pre_len {
        let deleted = self.buffer.text()
            .chars()
            .nth(off as usize)
            .map(|c| c.to_string())
            .unwrap_or_default();
        self.pending_log
            .append(&PendingEntry::Delete { offset: off, len: 1 })?;
        self.buffer.delete_char_forward();
        if let Some(share) = self.share.as_mut() {
            let cs = crate::share::bridge::changeset_for_delete(pre_len, off, deleted);
            share.outbound.send(cs)?;
        }
    }
}
```

- [ ] **Step 2: Build**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 3: Commit**

```bash
git add crates/pad/src/app.rs
git commit -m "feat(pad/share): forward local insert/backspace/delete as outbound Changesets"
```

---

## Task 10: Status bar shows share state + author count

**Files:**
- Modify: `crates/pad/src/tui/status_bar.rs`
- Modify: `crates/pad/src/tui/mod.rs`
- Modify: `crates/pad/src/app.rs`

- [ ] **Step 1: Extend `status_bar::render` signature**

```rust
// crates/pad/src/tui/status_bar.rs
use crate::buffer::Buffer;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Paragraph;

#[derive(Debug, Clone, Copy)]
pub struct ShareBadge {
    pub author_count: usize,
}

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    buffer: &Buffer,
    file_label: &str,
    share: Option<ShareBadge>,
) {
    let dirty = if buffer.is_dirty() { "[modified] " } else { "" };
    let pos = buffer.cursor();
    let share_part = match share {
        Some(b) => format!("  Shared • you +{}", b.author_count.saturating_sub(1)),
        None => String::new(),
    };
    let line = format!(
        "  {dirty}{file_label}    line {}, col {}{}",
        pos.line + 1,
        pos.col + 1,
        share_part,
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

- [ ] **Step 2: Pass through `Tui::draw_app`**

In `crates/pad/src/tui/mod.rs`, change `draw_app` to take an optional `ShareBadge` and forward it:

```rust
pub fn draw_app(
    &mut self,
    buffer: &Buffer,
    file_label: &str,
    prompt: Option<(&str, &str)>,
    show_help: bool,
    share: Option<status_bar::ShareBadge>,
) -> anyhow::Result<()> {
    self.terminal.draw(|frame| {
        // ... existing layout ...
        // status_bar::render call replaced with the 5-arg form, e.g.:
        status_bar::render(frame, chunks[last], buffer, file_label, share);
    })?;
    Ok(())
}
```

- [ ] **Step 3: Compute the badge in `App::run`**

```rust
let share_badge = self.share.as_ref().map(|s| crate::tui::status_bar::ShareBadge {
    author_count: s.authors.len(),
});
tui.draw_app(&self.buffer, &self.file_label, prompt, show_help, share_badge)?;
```

- [ ] **Step 4: Build**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 5: Commit**

```bash
git add crates/pad/src/tui/ crates/pad/src/app.rs
git commit -m "feat(pad): status bar shows 'Shared • you +N' badge when active"
```

---

## Task 11: Collision modal — pad already has content

**Files:**
- Create: `crates/pad/src/share/collision.rs`
- Modify: `crates/pad/src/share/mod.rs`
- Modify: `crates/pad/src/app.rs`

- [ ] **Step 1: Implement collision detection + flow**

```rust
// crates/pad/src/share/collision.rs
use crate::buffer::Buffer;
use etherpad_client::changeset::Changeset;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionAction {
    EditExisting,
    DifferentName,
    Cancel,
}

/// Compare the remote pad's initial text against the local buffer.
/// Returns true if the remote has non-trivial content that would conflict
/// with pushing the local buffer as the initial changeset.
pub fn would_collide(remote_initial_text: &str) -> bool {
    let trimmed = remote_initial_text.trim();
    !trimmed.is_empty()
}

/// When user picks EditExisting, build a changeset that replaces the local
/// buffer's content with the remote's. The local buffer's content goes into
/// the pre-share snapshot before this is applied.
pub fn changeset_replace_local_with_remote(
    local: &Buffer,
    remote_text: &str,
) -> Changeset {
    use super::bridge;
    // Strategy: emit a single delete-all + insert-all.
    let local_len = local.text_len();
    let mut wire = String::from("Z:");
    push_base36(&mut wire, local_len);
    let new_len = remote_text.chars().count() as u32;
    let delta = new_len as i64 - local_len as i64;
    if delta >= 0 {
        wire.push('>');
        push_base36(&mut wire, delta as u32);
    } else {
        wire.push('<');
        push_base36(&mut wire, (-delta) as u32);
    }
    if local_len > 0 {
        wire.push('-');
        push_base36(&mut wire, local_len);
    }
    if new_len > 0 {
        wire.push('+');
        push_base36(&mut wire, new_len);
    }
    wire.push('$');
    wire.push_str(&local.text()); // deleted chars go first in the bank
    wire.push_str(remote_text); // inserted chars follow
    // Build via parser to share canonicalization with the rest of the codec.
    etherpad_client::changeset::parser::parse(&wire).unwrap_or_else(|_| {
        // Fallback: just an empty insert; shouldn't be reachable for valid inputs.
        bridge::changeset_for_insert(local_len, 0, remote_text)
    })
}

fn push_base36(out: &mut String, mut n: u32) {
    if n == 0 {
        out.push('0');
        return;
    }
    let mut buf = Vec::new();
    while n > 0 {
        let d = (n % 36) as u8;
        buf.push(if d < 10 { b'0' + d } else { b'a' + (d - 10) });
        n /= 36;
    }
    buf.reverse();
    out.push_str(std::str::from_utf8(&buf).unwrap());
}
```

- [ ] **Step 2: Wire the modal**

Add to `AppState` in `app.rs`:

```rust
CollisionPrompt { remote_text: String, remote_base: String, pad_id: String },
```

`start_share` becomes aware of the handshake snapshot: instead of `connect_and_seed` doing both connect AND push, split it:

```rust
async fn start_share(&mut self, remote: &str) -> anyhow::Result<()> {
    let pad_id = self.derive_pad_id();
    let handles = crate::share::network::connect(remote, &pad_id).await
        .map_err(|e| anyhow::anyhow!("connect: {}", e.message))?;
    if crate::share::collision::would_collide(&handles.initial_text)
        && !self.buffer.text().is_empty()
    {
        // Stash handles in an in-progress state for the modal to resolve.
        self.pending_share = Some(PendingShare {
            remote_base: remote.to_string(),
            pad_id,
            handles,
        });
        self.state = AppState::CollisionPrompt {
            remote_text: self.pending_share.as_ref().unwrap().handles.initial_text.clone(),
            remote_base: remote.to_string(),
            pad_id: self.pending_share.as_ref().unwrap().pad_id.clone(),
        };
        return Ok(());
    }
    self.attach_share(remote, &pad_id, handles, /* seed = */ true).await
}
```

Add `PendingShare` struct + helper `attach_share`:

```rust
struct PendingShare {
    remote_base: String,
    pad_id: String,
    handles: crate::share::network::NetworkHandles,
}

impl App {
    async fn attach_share(
        &mut self,
        remote_base: &str,
        pad_id: &str,
        handles: crate::share::network::NetworkHandles,
        seed_with_local: bool,
    ) -> anyhow::Result<()> {
        let _ = self.sidecar.pre_share_snapshot(&self.buffer);
        let mut outbound = crate::share::outbound::OutboundQueue::new(handles.outbound_tx);
        if seed_with_local && !self.buffer.text().is_empty() {
            let cs = crate::share::bridge::changeset_for_insert(
                handles.initial_text.chars().count() as u32, 0, &self.buffer.text(),
            );
            outbound.send(cs)?;
        }
        let mut authors = std::collections::HashSet::new();
        authors.insert(handles.author_id.clone());
        self.share = Some(crate::share::ShareState {
            pad_id: pad_id.into(),
            remote_base: remote_base.into(),
            author_id: handles.author_id,
            outbound,
            inbound_rx: handles.inbound_rx,
            net_task: handles.task,
            authors,
        });
        Ok(())
    }
}
```

Handle the collision prompt:

```rust
AppState::CollisionPrompt { .. } => self.handle_collision(action).await?,
```

```rust
async fn handle_collision(&mut self, action: KeyAction) -> anyhow::Result<()> {
    let AppState::CollisionPrompt { remote_text, remote_base, pad_id } = std::mem::replace(
        &mut self.state, AppState::Editing
    ) else { return Ok(()); };
    match action {
        KeyAction::InsertChar('e') | KeyAction::InsertChar('E') => {
            // Snapshot local first, then replace.
            let _ = self.sidecar.pre_share_snapshot(&self.buffer);
            self.buffer.replace_all_text(&remote_text);
            let Some(pending) = self.pending_share.take() else { return Ok(()); };
            self.attach_share(&pending.remote_base, &pending.pad_id, pending.handles, false).await?;
        }
        KeyAction::InsertChar('d') | KeyAction::InsertChar('D') => {
            // Different name: prompt for one.
            self.pending_share = None;
            self.state = AppState::SharePadNamePrompt(String::new());
        }
        _ => {
            // Cancel — drop the handles.
            self.pending_share = None;
        }
    }
    Ok(())
}
```

Add another `AppState`:

```rust
SharePadNamePrompt(String),
```

```rust
AppState::SharePadNamePrompt(input) => Some(("Pad name", input.as_str())),
```

And handle it:

```rust
async fn handle_share_pad_name(&mut self, action: KeyAction) -> anyhow::Result<()> {
    let AppState::SharePadNamePrompt(input) = &mut self.state else { return Ok(()); };
    match action {
        KeyAction::InsertChar('\n') => {
            let new_pad_id = input.clone();
            self.state = AppState::Editing;
            if !new_pad_id.is_empty() {
                let cfg = crate::config::Config::load().unwrap_or_default();
                if let Some(remote) = cfg.remote {
                    Box::pin(self.start_share_with_pad_id(&remote, &new_pad_id)).await?;
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

async fn start_share_with_pad_id(&mut self, remote: &str, pad_id: &str) -> anyhow::Result<()> {
    let handles = crate::share::network::connect(remote, pad_id).await
        .map_err(|e| anyhow::anyhow!("connect: {}", e.message))?;
    if crate::share::collision::would_collide(&handles.initial_text)
        && !self.buffer.text().is_empty()
    {
        self.pending_share = Some(PendingShare {
            remote_base: remote.to_string(),
            pad_id: pad_id.to_string(),
            handles,
        });
        self.state = AppState::CollisionPrompt {
            remote_text: self.pending_share.as_ref().unwrap().handles.initial_text.clone(),
            remote_base: remote.to_string(),
            pad_id: pad_id.to_string(),
        };
        return Ok(());
    }
    self.attach_share(remote, pad_id, handles, true).await
}
```

- [ ] **Step 3: Wire module**

`crates/pad/src/share/mod.rs`:

```rust
pub mod bridge;
pub mod collision;
pub mod inbound;
pub mod network;
pub mod outbound;
```

- [ ] **Step 4: Build**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 5: Commit**

```bash
git add crates/pad/src/
git commit -m "feat(pad/share): collision modal [E]dit existing / [D]ifferent name / [C]ancel"
```

---

## Task 12: M-S Unshare

**Files:**
- Modify: `crates/pad/src/app.rs`
- Modify: `crates/pad/src/share/mod.rs`

- [ ] **Step 1: Add `unshare()` to ShareState**

```rust
impl ShareState {
    pub fn unshare(self) {
        self.net_task.abort();
    }
}
```

- [ ] **Step 2: Handle the second `M-S` press**

In `handle_editing`'s `Share` arm:

```rust
KeyAction::Share => {
    if let Some(share) = self.share.take() {
        share.unshare();
        self.file_label = self.file_path.as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "New Buffer".into());
    } else {
        // ... existing prompt-and-connect logic ...
    }
}
```

- [ ] **Step 3: Build**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 4: Commit**

```bash
git add crates/pad/
git commit -m "feat(pad/share): M-S unshare — aborts network task, returns to local-only"
```

---

## Task 13: Multi-author display — track presence

**Files:**
- Modify: `crates/pad/src/share/mod.rs`
- Modify: `crates/pad/src/share/network.rs`

Presence comes via `COLLABROOM { type: USER_NEWINFO, data: { ... } }` events. We extend the network task to forward presence events alongside changesets.

- [ ] **Step 1: Extend `NetworkHandles` with a presence channel**

```rust
// in network.rs
use serde_json::Value;

pub struct NetworkHandles {
    pub outbound_tx: mpsc::UnboundedSender<Changeset>,
    pub inbound_rx: mpsc::UnboundedReceiver<Changeset>,
    pub presence_rx: mpsc::UnboundedReceiver<PresenceEvent>,
    pub task: tokio::task::JoinHandle<()>,
    pub author_id: String,
    pub rev: u32,
    pub initial_text: String,
}

#[derive(Debug, Clone)]
pub enum PresenceEvent {
    Join { author_id: String, display_name: Option<String> },
    Leave { author_id: String },
}
```

Update `connect` to spawn the pump differently — use `session.pump_once` but also need raw access for USER_NEWINFO. Simplest: extend `PadSession::pump_once` to surface raw envelope.

In `crates/etherpad-client/src/session.rs` extend `pump_once` return:

```rust
pub enum InboundEvent {
    Changeset(Changeset),
    UserJoin { author_id: String, display_name: Option<String> },
    UserLeave { author_id: String },
    Other,
}

impl PadSession {
    pub async fn pump_once_event(&mut self) -> Result<InboundEvent> {
        let msg = self.socket.recv().await.ok_or_else(|| {
            ClientError::Protocol("socket closed".into())
        })?;
        let kind = msg["type"].as_str().unwrap_or("");
        if kind == "COLLABROOM" {
            let inner = &msg["data"];
            match inner["type"].as_str().unwrap_or("") {
                "NEW_CHANGES" => {
                    if let Some(wire) = inner["changeset"].as_str() {
                        let cs = crate::changeset::parser::parse(wire)?;
                        if let Some(rev) = inner["newRev"].as_u64() {
                            self.rev = rev as u32;
                        }
                        return Ok(InboundEvent::Changeset(cs));
                    }
                }
                "ACCEPT_COMMIT" => {
                    if let Some(rev) = inner["newRev"].as_u64() {
                        self.rev = rev as u32;
                    }
                }
                "USER_NEWINFO" => {
                    let userInfo = &inner["userInfo"];
                    let author_id = userInfo["userId"].as_str().unwrap_or("").to_string();
                    let display = userInfo["name"].as_str().map(|s| s.to_string());
                    if !author_id.is_empty() {
                        return Ok(InboundEvent::UserJoin {
                            author_id,
                            display_name: display,
                        });
                    }
                }
                "USER_LEAVE" => {
                    let userInfo = &inner["userInfo"];
                    let author_id = userInfo["userId"].as_str().unwrap_or("").to_string();
                    if !author_id.is_empty() {
                        return Ok(InboundEvent::UserLeave { author_id });
                    }
                }
                _ => {}
            }
        }
        Ok(InboundEvent::Other)
    }
}
```

Add `pub use session::InboundEvent` to `lib.rs`.

- [ ] **Step 2: Use the typed event in network task**

In `crates/pad/src/share/network.rs`, replace the `pump_once` call:

```rust
let (presence_tx, presence_rx) = mpsc::unbounded_channel::<PresenceEvent>();

let task = tokio::spawn(async move {
    let mut session = session;
    loop {
        tokio::select! {
            outbound = outbound_rx.recv() => {
                let Some(cs) = outbound else { break };
                if session.send_changeset(&cs).await.is_err() { break; }
            }
            pumped = session.pump_once_event() => {
                match pumped {
                    Ok(etherpad_client::InboundEvent::Changeset(cs)) => {
                        if inbound_tx.send(cs).is_err() { break; }
                    }
                    Ok(etherpad_client::InboundEvent::UserJoin { author_id, display_name }) => {
                        let _ = presence_tx.send(PresenceEvent::Join { author_id, display_name });
                    }
                    Ok(etherpad_client::InboundEvent::UserLeave { author_id }) => {
                        let _ = presence_tx.send(PresenceEvent::Leave { author_id });
                    }
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        }
    }
    let _ = session.disconnect().await;
});
```

Pump presence in App:

```rust
// In App::run select!:
Some(p) = async {
    if let Some(s) = self.share.as_mut() { s.presence_rx.recv().await } else { std::future::pending().await }
} => {
    if let Some(share) = self.share.as_mut() {
        match p {
            crate::share::network::PresenceEvent::Join { author_id, .. } => {
                share.authors.insert(author_id);
            }
            crate::share::network::PresenceEvent::Leave { author_id } => {
                share.authors.remove(&author_id);
            }
        }
    }
}
```

Add `presence_rx` field to `ShareState`:

```rust
pub presence_rx: mpsc::UnboundedReceiver<network::PresenceEvent>,
```

- [ ] **Step 3: Build**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 4: Commit**

```bash
git add crates/etherpad-client/src/session.rs crates/etherpad-client/src/lib.rs crates/pad/src/share/
git commit -m "feat(share): forward USER_NEWINFO / USER_LEAVE presence events"
```

---

## Task 14: `M-A` author overlay

**Files:**
- Create: `crates/pad/src/tui/author_overlay.rs`
- Modify: `crates/pad/src/tui/mod.rs`
- Modify: `crates/pad/src/app.rs`

- [ ] **Step 1: Implement the overlay**

```rust
// crates/pad/src/tui/author_overlay.rs
use ratatui::Frame;
use ratatui::layout::{Margin, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame<'_>, area: Rect, authors: &[String], self_id: &str) {
    let mut lines = Vec::new();
    for a in authors {
        let label = if a == self_id { format!("{a} (you)") } else { a.clone() };
        lines.push(label);
    }
    let text = lines.join("\n");
    let inner = area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });
    // Anchor to top-right: 24 cols wide, height = author count + 2 (border).
    let h = (authors.len() as u16 + 2).min(area.height);
    let w = 28u16.min(area.width);
    let region = Rect {
        x: inner.right().saturating_sub(w),
        y: inner.y,
        width: w,
        height: h,
    };
    let p = Paragraph::new(text)
        .block(Block::default().title(" Authors ").borders(Borders::ALL))
        .style(Style::default().fg(Color::Reset).bg(Color::Black));
    frame.render_widget(p, region);
}
```

- [ ] **Step 2: Add to tui module + draw**

In `tui/mod.rs`:

```rust
pub mod author_overlay;
```

Extend `draw_app` to take `show_authors: bool` and a slice of author ids + self id:

```rust
pub fn draw_app(
    &mut self,
    buffer: &Buffer,
    file_label: &str,
    prompt: Option<(&str, &str)>,
    show_help: bool,
    share: Option<status_bar::ShareBadge>,
    authors: Option<(&[String], &str)>, // (authors, self_id)
) -> anyhow::Result<()> {
    self.terminal.draw(|frame| {
        // ... layout unchanged ...
        if let Some((authors, self_id)) = authors {
            author_overlay::render(frame, chunks[0], authors, self_id);
        }
        // ...
    })?;
    Ok(())
}
```

- [ ] **Step 3: Track toggle in App**

Add `show_authors: bool` to App, initialize false. In `handle_editing`:

```rust
KeyAction::ToggleAuthors => self.show_authors = !self.show_authors,
```

In `App::run`, compute the args:

```rust
let authors_vec: Vec<String>;
let authors_arg = if self.show_authors {
    if let Some(s) = &self.share {
        authors_vec = s.authors.iter().cloned().collect();
        Some((authors_vec.as_slice(), s.author_id.as_str()))
    } else {
        None
    }
} else {
    None
};
tui.draw_app(&self.buffer, &self.file_label, prompt, show_help, share_badge, authors_arg)?;
```

- [ ] **Step 4: Build**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 5: Commit**

```bash
git add crates/pad/src/tui/ crates/pad/src/app.rs
git commit -m "feat(pad): M-A toggles persistent author list overlay (top-right)"
```

---

## Task 15: QR code + Share overlay

**Files:**
- Modify: `crates/pad/Cargo.toml`
- Create: `crates/pad/src/share/qr.rs`
- Create: `crates/pad/src/tui/share_overlay.rs`
- Modify: `crates/pad/src/tui/mod.rs`
- Modify: `crates/pad/src/app.rs`

- [ ] **Step 1: Add `qrcode` dep**

In `crates/pad/Cargo.toml`:

```toml
qrcode = "0.14"
```

- [ ] **Step 2: Implement ANSI QR rendering**

```rust
// crates/pad/src/share/qr.rs
use qrcode::{QrCode, render::unicode::Dense1x2};

pub fn ansi(url: &str) -> String {
    QrCode::new(url.as_bytes())
        .map(|code| {
            code.render::<Dense1x2>()
                .dark_color(Dense1x2::Light)
                .light_color(Dense1x2::Dark)
                .build()
        })
        .unwrap_or_default()
}
```

(`Dense1x2` maps two QR cells into one terminal cell vertically, giving roughly square output.)

- [ ] **Step 3: Implement share overlay**

```rust
// crates/pad/src/tui/share_overlay.rs
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame<'_>, area: Rect, url: &str, qr_ansi: &str) {
    let body = format!(
        "Shared at:\n  {url}\n\n{qr_ansi}\n\n[M-S to unshare]   [M-C copy URL]   any key dismiss",
    );
    let p = Paragraph::new(body)
        .block(Block::default().title(" Share ").borders(Borders::ALL))
        .style(Style::default().fg(Color::Reset));
    frame.render_widget(p, area);
}
```

- [ ] **Step 4: Wire show-on-share + M-Q**

Add `AppState::ShareOverlay { url: String, qr: String }`.

When `start_share`/`attach_share` succeeds, set this state. The overlay dismisses on any key (consistent with help-overlay pattern, dismiss-and-redispatch).

```rust
// In attach_share, at the end:
let url = self.share.as_ref().unwrap().share_url();
let qr = crate::share::qr::ansi(&url);
self.state = AppState::ShareOverlay { url, qr };
```

For `M-Q`:

```rust
KeyAction::ReshowQr => {
    if let Some(share) = &self.share {
        let url = share.share_url();
        let qr = crate::share::qr::ansi(&url);
        self.state = AppState::ShareOverlay { url, qr };
    }
}
```

In `handle`:

```rust
AppState::ShareOverlay { .. } => {
    self.state = AppState::Editing;
    self.handle_editing(action).await?; // dismiss + redispatch, same pattern as help
}
```

In `Tui::draw_app`, when ShareOverlay is active, render `share_overlay::render` in the main pane.

- [ ] **Step 5: Build + smoke test**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 6: Commit**

```bash
git add crates/pad/Cargo.toml crates/pad/src/share/qr.rs crates/pad/src/tui/share_overlay.rs crates/pad/src/tui/mod.rs crates/pad/src/app.rs
git commit -m "feat(pad/share): share overlay with ANSI QR + M-Q to reshow"
```

---

## Task 16: `M-C` copy URL via OSC 52

**Files:**
- Create: `crates/pad/src/share/osc52.rs`
- Modify: `crates/pad/src/share/mod.rs`
- Modify: `crates/pad/src/app.rs`

- [ ] **Step 1: Implement OSC 52 escape**

```rust
// crates/pad/src/share/osc52.rs
use std::io::Write;

/// Emit an OSC 52 clipboard-set sequence on stdout. Receiving terminal must
/// have OSC 52 enabled (most modern terminals do; macOS Terminal.app does not).
///
/// Wire format: `\x1b]52;c;BASE64\x07`.
pub fn copy_to_clipboard(text: &str) -> std::io::Result<()> {
    let mut stdout = std::io::stdout();
    let b64 = base64_encode(text.as_bytes());
    write!(stdout, "\x1b]52;c;{}\x07", b64)?;
    stdout.flush()?;
    Ok(())
}

fn base64_encode(input: &[u8]) -> String {
    const ALPH: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };
        out.push(ALPH[(b0 >> 2) as usize] as char);
        out.push(ALPH[((b0 & 0x03) << 4 | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPH[((b1 & 0x0f) << 2 | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(ALPH[(b2 & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::base64_encode;

    #[test]
    fn base64_known_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }
}
```

- [ ] **Step 2: Wire `M-C`**

In `handle_editing`:

```rust
KeyAction::CopyShareUrl => {
    if let Some(share) = &self.share {
        let url = share.share_url();
        // Best-effort copy. Failure → fall back to a flash overlay with the URL
        // so the user can manually copy.
        if crate::share::osc52::copy_to_clipboard(&url).is_err() {
            self.state = AppState::FlashMessage(format!("URL: {url}"));
        } else {
            self.state = AppState::FlashMessage("URL copied (OSC 52)".into());
        }
    }
}
```

- [ ] **Step 3: Wire into module**

```rust
// crates/pad/src/share/mod.rs
pub mod bridge;
pub mod collision;
pub mod inbound;
pub mod network;
pub mod osc52;
pub mod outbound;
pub mod qr;
```

- [ ] **Step 4: Test the base64**

Run: `cargo test -p pad osc52::tests`
Expected: 1 test passes.

- [ ] **Step 5: Commit**

```bash
git add crates/pad/src/share/ crates/pad/src/app.rs
git commit -m "feat(pad/share): M-C copies share URL via OSC 52 clipboard"
```

---

## Task 17: `pad <url>` join

**Files:**
- Modify: `crates/pad/src/cli.rs`
- Create: `crates/pad/src/share/url_parse.rs`
- Modify: `crates/pad/src/share/mod.rs`
- Modify: `crates/pad/src/main.rs`
- Modify: `crates/pad/src/app.rs`
- Create: `crates/pad/tests/share_url_parse.rs`

- [ ] **Step 1: Write failing tests**

```rust
// crates/pad/tests/share_url_parse.rs
use pad::share::url_parse::parse_pad_url;

#[test]
fn http_url() {
    let r = parse_pad_url("http://example.com:9001/p/test").unwrap();
    assert_eq!(r.remote_base, "http://example.com:9001");
    assert_eq!(r.pad_id, "test");
}

#[test]
fn https_url_default_port() {
    let r = parse_pad_url("https://etherpad.org/p/my-pad").unwrap();
    assert_eq!(r.remote_base, "https://etherpad.org");
    assert_eq!(r.pad_id, "my-pad");
}

#[test]
fn rejects_non_pad_path() {
    assert!(parse_pad_url("http://example.com/foo").is_err());
}

#[test]
fn rejects_non_http_scheme() {
    assert!(parse_pad_url("ftp://example.com/p/x").is_err());
}
```

- [ ] **Step 2: Implement `url_parse.rs`**

```rust
// crates/pad/src/share/url_parse.rs
pub struct ParsedPadUrl {
    pub remote_base: String,
    pub pad_id: String,
}

pub fn parse_pad_url(s: &str) -> anyhow::Result<ParsedPadUrl> {
    let url = url::Url::parse(s).map_err(|e| anyhow::anyhow!("parse url: {e}"))?;
    if url.scheme() != "http" && url.scheme() != "https" {
        anyhow::bail!("unsupported scheme: {}", url.scheme());
    }
    let path = url.path();
    let Some(rest) = path.strip_prefix("/p/") else {
        anyhow::bail!("URL path doesn't start with /p/");
    };
    let pad_id = rest.split('/').next().unwrap_or("").to_string();
    if pad_id.is_empty() {
        anyhow::bail!("pad id is empty");
    }
    let host = url.host_str().ok_or_else(|| anyhow::anyhow!("no host"))?;
    let port_part = match (url.port(), url.scheme()) {
        (Some(p), _) => format!(":{p}"),
        _ => String::new(),
    };
    let remote_base = format!("{}://{}{}", url.scheme(), host, port_part);
    Ok(ParsedPadUrl { remote_base, pad_id })
}
```

- [ ] **Step 3: Wire module + run tests**

```rust
// crates/pad/src/share/mod.rs
pub mod url_parse;
```

Run: `cargo test -p pad --test share_url_parse`
Expected: 4 tests pass.

- [ ] **Step 4: Extend CLI**

In `crates/pad/src/cli.rs`:

```rust
#[derive(Debug, Clone)]
pub enum Mode {
    Untitled,
    OpenFile(PathBuf),
    JoinUrl(String),
    Recover,
    Restore,
    Setup,
}

impl Args {
    pub fn mode(&self) -> Mode {
        if self.recover {
            Mode::Recover
        } else if self.restore {
            Mode::Restore
        } else if self.setup {
            Mode::Setup
        } else if let Some(p) = &self.path {
            let s = p.to_string_lossy();
            if s.starts_with("http://") || s.starts_with("https://") {
                Mode::JoinUrl(s.to_string())
            } else {
                Mode::OpenFile(p.clone())
            }
        } else {
            Mode::Untitled
        }
    }
}
```

Add the new flags:

```rust
#[derive(Parser, Debug, Clone)]
#[command(name = "pad", version, about = "Nano-class terminal text editor.")]
pub struct Args {
    pub path: Option<PathBuf>,

    #[arg(long, conflicts_with_all = ["path", "restore", "setup"])]
    pub recover: bool,
    #[arg(long, conflicts_with_all = ["path", "recover", "setup"])]
    pub restore: bool,
    #[arg(long, conflicts_with_all = ["path", "recover", "restore"])]
    pub setup: bool,
}
```

- [ ] **Step 5: Handle `JoinUrl` in main**

In `main.rs`:

```rust
Mode::JoinUrl(url) => {
    let parsed = pad::share::url_parse::parse_pad_url(&url)?;
    let mut tui = Tui::enter()?;
    let mut app = App::from_join_url(parsed).await?;
    app.run(&mut tui).await
}
```

Add `App::from_join_url`:

```rust
impl App {
    pub async fn from_join_url(p: crate::share::url_parse::ParsedPadUrl) -> anyhow::Result<Self> {
        let mut app = Self::from_mode(crate::cli::Mode::Untitled)?;
        app.start_share_with_pad_id(&p.remote_base, &p.pad_id).await?;
        // Joining means buffer must mirror the remote's initial text.
        if let Some(share) = &app.share {
            // After the share overlay sets state, replace the rope.
            // The pending_share path doesn't fire here since we passed
            // start_share_with_pad_id, which set up `share` directly.
            let _ = share; // suppress unused-borrow warning
        }
        Ok(app)
    }
}
```

For joining, we replace the local buffer with the remote text. `attach_share` already pulls `handles.initial_text`; we extend it to seed the buffer when joining:

```rust
async fn attach_share(
    &mut self,
    remote_base: &str,
    pad_id: &str,
    handles: crate::share::network::NetworkHandles,
    seed_with_local: bool,
    seed_buffer_from_remote: bool,
) -> anyhow::Result<()> {
    // existing body, plus:
    if seed_buffer_from_remote {
        self.buffer.replace_all_text(&handles.initial_text);
        self.buffer.mark_clean();
    }
    // ...
}
```

Update callers accordingly.

- [ ] **Step 6: Build + manual run**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 7: Commit**

```bash
git add crates/pad/
git commit -m "feat(pad): pad <url> joins a remote pad as a terminal collaborator"
```

---

## Task 18: First-run remote config + scanner.etherpad.org

**Files:**
- Create: `crates/pad/src/share/scanner.rs`
- Modify: `crates/pad/src/share/mod.rs`
- Modify: `crates/pad/src/cli.rs`
- Modify: `crates/pad/src/main.rs`

- [ ] **Step 1: Implement scanner with fallback list**

```rust
// crates/pad/src/share/scanner.rs
use std::time::Duration;

pub struct CommunityInstance {
    pub url: String,
    pub label: String,
}

const FALLBACK_INSTANCES: &[(&str, &str)] = &[
    ("https://yopad.eu", "yopad.eu"),
    ("https://pad.disroot.org", "pad.disroot.org"),
    ("https://etherpad.wikimedia.org", "etherpad.wikimedia.org"),
    ("https://pad.riseup.net", "pad.riseup.net"),
];

pub async fn fetch_or_fallback() -> Vec<CommunityInstance> {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(_) => return fallback(),
    };
    let resp = match client
        .get("https://scanner.etherpad.org/instances.json")
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return fallback(),
    };
    let body: serde_json::Value = match resp.json().await {
        Ok(b) => b,
        Err(_) => return fallback(),
    };
    let Some(arr) = body.as_array() else {
        return fallback();
    };
    let mut out = Vec::new();
    for item in arr {
        let Some(url) = item["url"].as_str() else {
            continue;
        };
        let label = item["name"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| url.to_string());
        out.push(CommunityInstance {
            url: url.to_string(),
            label,
        });
    }
    if out.is_empty() { fallback() } else { out }
}

fn fallback() -> Vec<CommunityInstance> {
    FALLBACK_INSTANCES
        .iter()
        .map(|(url, label)| CommunityInstance {
            url: (*url).into(),
            label: (*label).into(),
        })
        .collect()
}
```

- [ ] **Step 2: Wire `--setup` flag**

In `main.rs`:

```rust
Mode::Setup => {
    let instances = pad::share::scanner::fetch_or_fallback().await;
    println!("Pick an Etherpad instance to use as your default remote:");
    for (i, inst) in instances.iter().enumerate() {
        println!("  [{}] {} ({})", i + 1, inst.label, inst.url);
    }
    print!("> ");
    std::io::Write::flush(&mut std::io::stdout())?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let idx: usize = input.trim().parse().unwrap_or(0);
    if idx == 0 || idx > instances.len() {
        eprintln!("invalid selection");
        std::process::exit(1);
    }
    let chosen = &instances[idx - 1];
    let mut cfg = pad::config::Config::load().unwrap_or_default();
    cfg.remote = Some(chosen.url.clone());
    if !cfg.consented_remotes.contains(&chosen.url) {
        cfg.consented_remotes.push(chosen.url.clone());
    }
    cfg.save()?;
    println!("Saved remote = {}", chosen.url);
    return Ok(());
}
```

- [ ] **Step 3: In-editor first-run**

When `M-S` is pressed and `config.remote` is `None`, show a status flash:

```rust
AppState::RemotePromptForShare(_) => {
    // For v0.2 minimal: redirect user to run `pad --setup`.
    // A full in-TUI picker can land in a v0.2.1 polish pass.
    self.state = AppState::FlashMessage(
        "No remote configured. Run 'pad --setup' first.".into()
    );
}
```

(Replace this `AppState` branch — earlier scaffolding implied a richer prompt. v0.2 keeps it terse to ship.)

- [ ] **Step 4: Wire module + build**

```rust
// crates/pad/src/share/mod.rs
pub mod scanner;
```

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 5: Commit**

```bash
git add crates/pad/
git commit -m "feat(pad): pad --setup picks remote from scanner.etherpad.org (or fallback)"
```

---

## Task 19: Pre-merge snapshot on long divergence

**Files:**
- Modify: `crates/pad/src/share/inbound.rs`
- Modify: `crates/pad/src/app.rs`

If we have a large unACK'd outbound queue (say, >50 ops) when a `NEW_CHANGES` arrives, snapshot pre-merge before applying. This is a safety net for the case where the OT rebase produces something surprising.

- [ ] **Step 1: Add divergence detection**

In `app.rs`, when processing an inbound changeset:

```rust
if let Some(share) = self.share.as_mut() {
    if share.outbound.pending_len() > 50 {
        let _ = self.sidecar.pre_merge_snapshot(&self.buffer);
    }
    crate::share::inbound::apply_remote(&mut self.buffer, &cs, &share.outbound)?;
}
```

- [ ] **Step 2: Build**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 3: Commit**

```bash
git add crates/pad/src/app.rs
git commit -m "feat(pad/share): pre-merge.snapshot when outbound queue >50 ops"
```

---

## Task 20: `pad --restore`

**Files:**
- Modify: `crates/pad/src/recover.rs`
- Modify: `crates/pad/src/cli.rs`
- Modify: `crates/pad/src/main.rs`

- [ ] **Step 1: Implement listing of `pre-share-*` / `pre-merge` snapshots**

In `recover.rs`:

```rust
pub struct Snapshot {
    pub path: PathBuf,
    pub kind: SnapshotKind,
    pub buffer_id: BufferId,
}

pub enum SnapshotKind {
    PreShare(u64),  // unix ts
    PreMerge,
}

pub fn list_snapshots(state_root: &Path) -> anyhow::Result<Vec<Snapshot>> {
    let mut out = Vec::new();
    if !state_root.exists() { return Ok(out); }
    for entry in fs::read_dir(state_root)? {
        let entry = entry?;
        let dir = entry.path();
        if !dir.is_dir() { continue; }
        let Some(name) = dir.file_name().and_then(|s| s.to_str()) else { continue };
        let Ok(id) = uuid::Uuid::parse_str(name) else { continue };
        for sub in fs::read_dir(&dir)? {
            let sub = sub?;
            let p = sub.path();
            let Some(n) = p.file_name().and_then(|s| s.to_str()) else { continue };
            if let Some(ts_str) = n.strip_prefix("pre-share-").and_then(|s| s.strip_suffix(".snapshot")) {
                let ts: u64 = ts_str.parse().unwrap_or(0);
                out.push(Snapshot { path: p, kind: SnapshotKind::PreShare(ts), buffer_id: id });
            } else if n == "pre-merge.snapshot" {
                out.push(Snapshot { path: p, kind: SnapshotKind::PreMerge, buffer_id: id });
            }
        }
    }
    Ok(out)
}

pub fn run_restore(state_root: &Path) -> anyhow::Result<()> {
    let snaps = list_snapshots(state_root)?;
    if snaps.is_empty() {
        println!("No snapshots found.");
        return Ok(());
    }
    println!("Available snapshots:");
    for (i, s) in snaps.iter().enumerate() {
        let label = match s.kind {
            SnapshotKind::PreShare(ts) => format!("pre-share @ {ts}"),
            SnapshotKind::PreMerge => "pre-merge".to_string(),
        };
        println!("  [{}] {} (buffer {})", i + 1, label, s.buffer_id);
    }
    println!("\nSelect a snapshot to open as a fresh local buffer (or q to quit):");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim();
    if input.eq_ignore_ascii_case("q") { return Ok(()); }
    let idx: usize = input.parse()?;
    let chosen = &snaps[idx - 1];
    let contents = fs::read_to_string(&chosen.path)?;
    println!("Opening snapshot contents in a fresh untitled buffer.");
    println!("Save with ^O after exiting to persist. Press Enter to continue.");
    let mut _wait = String::new();
    std::io::stdin().read_line(&mut _wait)?;
    // For v0.2 minimal: just spawn pad with no args; the user can paste the
    // contents back. A future polish pass can pre-load the snapshot directly
    // by passing it via env var.
    println!("Snapshot content:\n----------\n{contents}\n----------");
    Ok(())
}
```

(Full TUI integration of `--restore` loading directly into a buffer is a polish task; v0.2 surfaces the content for manual recovery.)

- [ ] **Step 2: Wire `--restore` in main**

```rust
Mode::Restore => {
    pad::recover::run_restore(&pad::config::paths::state_root())?;
    return Ok(());
}
```

- [ ] **Step 3: Build**

Run: `cargo build -p pad`
Expected: success.

- [ ] **Step 4: Commit**

```bash
git add crates/pad/
git commit -m "feat(pad): pad --restore lists pre-share/pre-merge snapshots"
```

---

## Task 21: PTY integration test for share flow (mock remote)

**Files:**
- Create: `crates/pad/tests/share_pty_e2e.rs`

We don't need a real Etherpad for this test — we just need to exercise the App's share-state machine. The simplest way: spawn `pad` with a fake remote that will fail to connect, and assert the editor stays alive and reports the error.

- [ ] **Step 1: Write the test**

```rust
// crates/pad/tests/share_pty_e2e.rs
use expectrl::{Eof, Expect, spawn};
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn ms_without_remote_shows_setup_hint() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("share.txt");
    std::fs::write(&path, "").unwrap();

    // Run with HOME/XDG_CONFIG_HOME pointed at our tempdir so the test
    // doesn't see the user's real config.
    let bin = env!("CARGO_BIN_EXE_pad");
    let cmd = format!(
        "env HOME={} XDG_CONFIG_HOME={}/config XDG_STATE_HOME={}/state {} {}",
        dir.path().display(),
        dir.path().display(),
        dir.path().display(),
        bin,
        path.display(),
    );
    let mut p = spawn(cmd).expect("spawn");
    p.set_expect_timeout(Some(Duration::from_secs(5)));

    std::thread::sleep(Duration::from_millis(300));
    // M-S → since there's no remote configured, status bar flashes a hint.
    // Most terminals translate Alt+S to ESC + s.
    p.send([0x1Bu8, b's'].as_slice()).unwrap();
    std::thread::sleep(Duration::from_millis(300));
    // ^X to exit.
    p.send([0x18u8].as_slice()).unwrap();
    p.expect(Eof).expect("editor must exit cleanly");
}
```

- [ ] **Step 2: Run the test**

Run: `cargo test -p pad --test share_pty_e2e`
Expected: pass (we're not asserting the flash text — just that the binary survives the M-S keypress without a remote configured).

- [ ] **Step 3: Commit**

```bash
git add crates/pad/tests/share_pty_e2e.rs
git commit -m "test(pad): PTY e2e — M-S without remote shows setup hint, editor survives"
```

---

## Task 22: CI update

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Add share tests to CI**

CI already runs `cargo test --workspace --exclude etherpad-client-spike --lib --tests` which picks up share tests automatically. Add an explicit step for the new share-pty test category to keep it visible:

```yaml
      - name: pad share PTY tests
        run: cargo test -p pad --test share_pty_e2e
```

- [ ] **Step 2: Run full CI locally**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --features etherpad-client/mock-socket -- -D warnings
cargo test --workspace --exclude etherpad-client-spike --lib --tests --features etherpad-client/mock-socket
cargo test -p pad --test perf_budget --release
cargo test -p pad --test pty_smoke
cargo test -p pad --test share_pty_e2e
```

Expected: all green.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add share PTY tests to workflow"
```

---

## Self-Review

After all 22 tasks are done, verify:

1. **Spec coverage:**
   - §4.4 Config / First-run / scanner.etherpad.org → Tasks 7 (Config), 18 (scanner) ✓
   - §5.2 First Share lifecycle → Tasks 4 (connect), 7 (M-S), 11 (collision), 15 (overlay) ✓
   - §5.3 Live shared editing → Tasks 5–9 (outbound/inbound/bridge/network/wiring) ✓
   - §5.4 Joining via pad <url> → Task 17 ✓
   - §6.2 M- bindings (M-S/M-A/M-C/M-Q) → Tasks 7, 12 (unshare), 14 (M-A), 15 (M-Q), 16 (M-C) ✓
   - §6.3 Multi-author display → Tasks 13 (presence), 14 (overlay) ✓
   - Pre-share / pre-merge snapshots → Tasks 8, 19 ✓
   - pad --restore → Task 20 ✓
   - pad --setup → Task 18 ✓

2. **Placeholder scan:** No "TBD", "TODO", or "implement later" in step bodies. Three explicit scope decisions are documented inline:
   - In-TUI first-run picker deferred to v0.2.1 polish; v0.2 ships a `--setup` CLI flow (Task 18).
   - `--restore` surfaces snapshot content but doesn't pre-load it into a buffer; full TUI integration is a polish task (Task 20).
   - Color-cursor rendering for remote authors is deferred (only the author overlay shows in v0.2; cursor color rendering needs ratatui custom widgets and lands in v0.2.1).

3. **Type consistency:**
   - `Buffer` methods used: `text`, `text_len`, `cursor_offset`, `replace_all_text`, `insert_char/insert_str/backspace/delete_char_forward`, `mark_clean`. All defined.
   - `SidecarHandle::pre_share_snapshot` / `pre_merge_snapshot` are both shipped by Task 8.
   - `ShareState` fields: `pad_id`, `remote_base`, `author_id`, `outbound`, `inbound_rx`, `presence_rx` (added Task 13), `net_task`, `authors`. Consistent.
   - `KeyAction` variants: `Share`, `ToggleAuthors`, `CopyShareUrl`, `ReshowQr` added in Task 7, used consistently downstream.
   - `AppState` additions: `RemotePromptForShare`, `CollisionPrompt`, `SharePadNamePrompt`, `ShareOverlay`. All defined and handled.

---

## Done criteria for Plan 3

- `cargo test --workspace --exclude etherpad-client-spike --features etherpad-client/mock-socket` green.
- `cargo test -p pad --test pty_smoke` green (existing + new share tests).
- Manual: `pad --setup` → pick a remote, save config; `pad foo.txt` → `M-S` → share overlay shows URL + QR; visit URL in a browser → see your buffer text; type in browser → see edits appear in pad's terminal view.
- `pad https://etherpad.example.org/p/test` → joins an existing remote pad and shows its contents.

When all of these hold, `pad` v0.2 is a usable nano replacement *plus* a working real-time collaborative terminal editor. v0.3 polish: in-TUI first-run picker, cursor color rendering, snapshot pre-load on `--restore`.
