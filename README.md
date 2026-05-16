# pad

A nano-class terminal text editor with optional real-time collaboration backed by [Etherpad](https://etherpad.org).

Edit files locally like nano, or join any Etherpad pad over the network and see other people's edits land in your terminal as they type.

```
pad notes.md                              # local file
pad https://pad.example.com/p/standup     # join a remote pad
```

## Status

`v0.1.0` — first tagged release. Plans 1 (offline client crate) and 2 (local nano-class editor) are complete; Plan 3 (full real-time collab against `pad-dev.etherpad.org`) is shippable with the caveats below.

## Features

- nano-style keybindings, including canonical alternatives:
  `Ctrl-A` / `Home`, `Ctrl-E` / `End`, `Ctrl-B`/`Ctrl-F`/`Ctrl-P`/`Ctrl-N` for movement; `Ctrl-D` / `Ctrl-H` for delete/backspace; `Ctrl-\` / `Ctrl-|` / `M-R` for replace; `Ctrl-W` for find; `M-G` / `Ctrl-_` for goto-line; `M-<` / `M->` / `Ctrl-Home` / `Ctrl-End` for doc start/end; `Ctrl-Y` / `Ctrl-V` / `PgUp` / `PgDn` for paging.
- `Ctrl-K` cut / `Ctrl-U` uncut / `Ctrl-R` insert-file / `Ctrl-O` save / `Ctrl-X` exit / `Ctrl-G` help / `Ctrl-C` cursor-position.
- Multi-line bracketed paste, `\r\n` / `\r` line-ending normalization, auto-scroll keeps the caret visible.
- Optional real-time collab when given a `https://<etherpad-host>/p/<padId>` URL: window title shows `<padId> @ <host>`, status bar shows the full URL, M-S re-opens the share overlay (URL + QR), M-A toggles author colors.
- Soft-wrap rendering, line numbers via `Ctrl-C`, on-disk crash recovery via a per-session sidecar (`pad --recover`).

## Real-time collab caveats

- The Etherpad web client's DOM-render path has an upstream bug under rapid remote `NEW_CHANGES`: bursts of small inserts on a single content line can scramble in the live view (the underlying `baseAText` is correct; refresh restores order). We work around this by batching outbound changesets with a fixed-cadence 1800ms commit window so receivers never see more than ~0.6 NEW_CHANGES/sec from us. Tradeoff: collaborators see your edits up to ~1.8s later than they would from a peer browser. Tracked at [ether/etherpad#7773](https://github.com/ether/etherpad/pull/7773) and a follow-up PR.
- A second related bug — server-side `_handleUserChanges` accepting changesets that strand the trailing `\n` — is patched in the same upstream PR. Until that lands and you're running a patched Etherpad, the pad's `setDocAText` reconcile path may disconnect a browser if a non-JS client misbehaves. `pad` itself is defensive enough not to trigger this.

## Install

One-liner (requires `cargo` — installs Rust first if you don't have it; see [rustup.rs](https://rustup.rs)):

```
curl -fsSL https://raw.githubusercontent.com/ether/pad/main/install.sh | sh
```

That runs `cargo install --locked --git https://github.com/ether/pad pad` under the hood and tells you where the binary landed. The same command works if you'd rather skip the script:

```
cargo install --locked --git https://github.com/ether/pad pad
```

## Building from source

```
cargo build --release -p pad
install -m 755 target/release/pad ~/.local/bin/
```

## Development

```
cargo test --workspace               # unit + integration
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

Bidi (network-touching) tests against a live Etherpad:

```
PAD_ETHERPAD_BASE=https://pad-dev.etherpad.org \
  cargo test -p pad --test bidi_scenarios -- --test-threads=1 --nocapture
```

Live-DOM verification via Playwright + a headless Chromium attached to the pad URL lives in `/tmp/ep-watcher/` (separate from this repo — see the `reference_playwright_browser_watch` memory note for the recipe).

## Diagnostics

```
PAD_DIAG_LOG=/tmp/pad-diag.log pad https://pad-dev.etherpad.org/p/<id>
```

Logs every outbound wire (changeset, baseRev, old_len, net_delta) and every inbound message to the file. Useful for "my edit didn't propagate" / "the browser sees scrambled text" investigations.

## License

Apache-2.0. See LICENSE.
