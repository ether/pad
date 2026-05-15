# Conformance fixtures

Each `*.json` here is a captured trace from a real Etherpad JS client. Format:

```json
{
  "name": "human-readable description",
  "kind": "apply" | "compose",
  "input":   { "text": "...", "changeset": "Z:...$..." },
  "compose": { "a": "Z:...$...", "b": "Z:...$..." },
  "expected": {
    "text": "...",          // for apply
    "changeset": "Z:...$..." // for compose
  }
}
```

Add new fixtures whenever a JS/Rust parity break is discovered. Capture from
the JS client by adding a `console.log(packed)` call in `Changeset.js` and
reproducing the scenario in a real Etherpad session.
