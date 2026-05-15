# etherpad-client

Reusable Rust client for the [Etherpad](https://etherpad.org) socket.io +
changeset protocol. Used by `pad` (a nano-class terminal editor) and intended
for future Rust ports of Etherpad's desktop and mobile clients.

## Status

Pre-1.0. The API is shaped by `pad`'s needs first; breaking changes likely
until v0.1.0 of `pad` ships. **Networking layer not yet shipped** — see Plan 1
in the parent repo for the deferred socket.io work.

## What's in the box

- Changeset parser + serializer (`Z:N>M|...$bank` wire format).
- OT primitives: `apply`, `compose`, `inverse`, `follow`.
  - `follow` uses a serialized-form tiebreaker for symmetric insert resolution
    (`follow(a,b)` and `follow(b,a)` agree without needing author IDs).
- Conformance test suite (`tests/conformance/fixtures/`) — add JSON fixtures
  captured from a real Etherpad JS client; CI gates on byte-equal output.
- Property tests (`proptest`): parse/serialize roundtrip, compose ≡ sequential
  apply, inverse round-trip, follow convergence — ~1500 cases per CI run.
- Exponential-backoff `Reconnect` state machine (1s/2s/4s/8s/16s/30s).
- Presence types: `AuthorId`, `ColorId` (deterministic `hash mod 7` palette),
  `AuthorInfo`, `CursorPos`, `PresenceEvent`.

## Known gaps (caught by conformance fixtures over time)

- Header line-bank parsing/emission is not yet implemented; per-op `|N`
  newline prefix is.
- `compose` may not produce byte-identical `char_bank` for delete-from-original
  ops (applied outcome is correct).
- `Socket` trait + `PadSession` + the rust_socketio integration are deferred
  pending a separate spike — `rust_socketio v0.6` did not deliver messages
  through to a real Etherpad server in initial testing, and the network layer
  is paused until that's resolved (or a hand-rolled socket.io v4 client is
  built per Plan 1's fallback path).

## License

Apache-2.0.
