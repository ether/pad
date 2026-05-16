//! Print the actual wire output of bridge::changeset_for_insert /
//! changeset_for_delete + the session-decorated form for visual inspection.

use etherpad_client::changeset::serializer::serialize as serialize_raw;
use pad::share::bridge::{changeset_for_delete, changeset_for_insert};

#[test]
fn print_typical_wire_outputs() {
    // Pad-dev welcome.
    let pad = "Proudly sponsored by vpsdime\n";
    let len = pad.chars().count();
    eprintln!("pad len: {len}");

    let cs1 = changeset_for_insert(pad, 0, "x");
    eprintln!("RAW insert 'x' @0: {}", serialize_raw(&cs1));

    let cs2 = changeset_for_insert(pad, 10, "x");
    eprintln!("RAW insert 'x' @10: {}", serialize_raw(&cs2));

    let cs3 = changeset_for_insert(pad, 28, "x");
    eprintln!("RAW insert 'x' @28 (before final \\n): {}", serialize_raw(&cs3));

    let cs4 = changeset_for_delete(pad, 0, "P".into());
    eprintln!("RAW delete 'P' @0: {}", serialize_raw(&cs4));

    // Now multi-line — three lines + trailing newline.
    let multi = "abc\ndef\nghi\n";
    eprintln!("multi pad len: {}", multi.chars().count());

    let cs5 = changeset_for_insert(multi, 12, "test");
    eprintln!("RAW insert 'test' @12 (end of multi): {}", serialize_raw(&cs5));

    let cs6 = changeset_for_insert(multi, 8, "test");
    eprintln!("RAW insert 'test' @8 (start of 'ghi'): {}", serialize_raw(&cs6));

    let cs7 = changeset_for_insert(multi, 4, "test");
    eprintln!("RAW insert 'test' @4 (start of 'def'): {}", serialize_raw(&cs7));
}
