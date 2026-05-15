# Spike: rust_socketio against Etherpad

Throwaway crate validating that the Rust socket.io ecosystem can speak to a
real Etherpad before we sink time into the changeset codec.

```bash
docker run --rm -d --name etherpad-spike -p 9001:9001 etherpad/etherpad:latest
until curl -fs http://localhost:9001/ > /dev/null; do sleep 1; done
cargo run -p etherpad-client-spike
docker stop etherpad-spike
```

Pass criterion: prints `SPIKE OK — received: ...` and exits 0.

Note: modern Etherpad auto-creates pads on first CLIENT_READY for anonymous
access — no HTTP-API pre-seed needed.
