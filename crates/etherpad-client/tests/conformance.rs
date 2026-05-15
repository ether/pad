use etherpad_client::changeset::parser::parse;
use etherpad_client::changeset::serializer::serialize;
use etherpad_client::ot::{apply, compose};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Fixture {
    name: String,
    kind: String,
    input: Option<ApplyInput>,
    compose: Option<ComposeInput>,
    expected: Expected,
}

#[derive(Deserialize)]
struct ApplyInput {
    text: String,
    changeset: String,
}

#[derive(Deserialize)]
struct ComposeInput {
    a: String,
    b: String,
}

#[derive(Deserialize)]
struct Expected {
    text: Option<String>,
    changeset: Option<String>,
}

#[test]
fn conformance() {
    let dir: PathBuf =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/conformance/fixtures");
    let mut count = 0usize;
    let mut failures = Vec::new();
    for entry in fs::read_dir(&dir).expect("fixtures dir") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let raw = fs::read_to_string(&path).unwrap();
        let f: Fixture = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
        count += 1;
        let ok = match f.kind.as_str() {
            "apply" => {
                let inp = f.input.as_ref().unwrap();
                let cs = parse(&inp.changeset).unwrap();
                let actual = apply(&cs, &inp.text).unwrap();
                let expected = f.expected.text.as_ref().unwrap();
                if &actual != expected {
                    failures.push(format!(
                        "{}: apply mismatch\n  expected {:?}\n  got      {:?}",
                        f.name, expected, actual
                    ));
                    false
                } else {
                    true
                }
            }
            "compose" => {
                let inp = f.compose.as_ref().unwrap();
                let a = parse(&inp.a).unwrap();
                let b = parse(&inp.b).unwrap();
                let c = compose(&a, &b).unwrap();
                let actual = serialize(&c);
                let expected = f.expected.changeset.as_ref().unwrap();
                if &actual != expected {
                    failures.push(format!(
                        "{}: compose mismatch\n  expected {:?}\n  got      {:?}",
                        f.name, expected, actual
                    ));
                    false
                } else {
                    true
                }
            }
            other => panic!("unknown fixture kind: {other}"),
        };
        if ok {
            eprintln!("ok: {}", f.name);
        }
    }
    assert!(count >= 3, "expected at least 3 fixtures, found {count}");
    assert!(
        failures.is_empty(),
        "{} fixture failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
