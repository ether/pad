//! Downstream wire-compatibility vectors (Phase 2 of ether/etherpad#7923).
//!
//! Etherpad core ships a canonical wire-format fixture that every client must
//! decode identically. This test loads that fixture and, for each vector,
//! applies `changeset` to `initialText` using the crate's OWN changeset
//! parser + OT apply path (the same code exercised by `changeset_roundtrip.rs`
//! and `ot_apply.rs`), then asserts the result equals `resultText`.
//!
//! Applying a changeset only produces TEXT — attribute pools annotate spans but
//! do not change the resulting characters — so the pool field is loaded for
//! fidelity with core's fixture schema but is not needed to verify the text
//! outcome.
//!
//! The fixture path is overridable via `ETHERPAD_WIRE_VECTORS` so core CI can
//! inject a fresh copy; it defaults to the vendored
//! `tests/fixtures/wire-vectors.json`.

use etherpad_client::changeset::parser::parse;
use etherpad_client::ot::apply;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Pool {
    #[serde(rename = "numToAttrib", default)]
    #[allow(dead_code)]
    num_to_attrib: BTreeMap<String, Vec<String>>,
    #[serde(rename = "nextNum", default)]
    #[allow(dead_code)]
    next_num: u32,
}

#[derive(Deserialize)]
struct Vector {
    name: String,
    #[serde(rename = "initialText")]
    initial_text: String,
    changeset: String,
    #[allow(dead_code)]
    pool: Pool,
    #[serde(rename = "resultText")]
    result_text: String,
}

fn vectors_path() -> PathBuf {
    match std::env::var("ETHERPAD_WIRE_VECTORS") {
        Ok(p) if !p.is_empty() => PathBuf::from(p),
        _ => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/wire-vectors.json"),
    }
}

#[test]
fn wire_vectors() {
    let path = vectors_path();
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read wire vectors {}: {e}", path.display()));
    let vectors: Vec<Vector> = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("parse wire vectors {}: {e}", path.display()));

    assert!(
        !vectors.is_empty(),
        "wire vectors file {} is empty",
        path.display()
    );

    let mut failures = Vec::new();
    for v in &vectors {
        let cs = match parse(&v.changeset) {
            Ok(cs) => cs,
            Err(e) => {
                failures.push(format!("{}: parse failed: {e}", v.name));
                continue;
            }
        };
        match apply(&cs, &v.initial_text) {
            Ok(actual) if actual == v.result_text => {
                eprintln!("ok: {}", v.name);
            }
            Ok(actual) => failures.push(format!(
                "{}: apply mismatch\n  expected {:?}\n  got      {:?}",
                v.name, v.result_text, actual
            )),
            Err(e) => failures.push(format!("{}: apply failed: {e}", v.name)),
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} wire vectors failed:\n{}",
        failures.len(),
        vectors.len(),
        failures.join("\n")
    );
}
