//! Regression: every Insert op in an outbound USER_CHANGES MUST carry the
//! author attribute reference (`*0` mapped to `["author", "<author-id>"]`
//! in the apool). Without it the server still ACKs the changeset, but the
//! resulting AText has unattributed text — and Etherpad's BROWSER client
//! then throws "mismatch error setting raw text in setDocAText" trying to
//! reconcile its local copy.
//!
//! We verify by inspecting what TungsteniteSocket actually emits via a
//! MockSocket (the wire frame), then checking:
//!   - The changeset string contains "*0"
//!   - The apool maps 0 -> ["author", <session author>]

#![cfg(feature = "mock-socket")]

use etherpad_client::changeset::{Changeset, Op, OpCode};
use etherpad_client::session::{PadSession, SessionConfig};
use etherpad_client::socket::mock::MockSocket;
use serde_json::json;

#[tokio::test]
async fn outbound_insert_includes_author_attrib_and_apool() {
    let socket = MockSocket::new();
    let sent_view = socket.sent.clone();
    let injector = socket.injector.clone();

    let mut session = PadSession::new(
        Box::new(socket),
        SessionConfig {
            pad_id: "p".into(),
            token: "t.x".into(),
            protocol_version: 2,
        },
    );

    // Drive the handshake with a synthetic CLIENT_VARS so session.author_id
    // gets populated with a known value.
    injector
        .send(json!({
            "type": "CLIENT_VARS",
            "data": {
                "padId": "p",
                "userId": "a.author-xyz",
                "collab_client_vars": {
                    "rev": 0,
                    "initialAttributedText": { "text": "hello\n", "attribs": "*0|1+6" },
                    "apool": { "numToAttrib": { "0": ["author", "a.author-xyz"] }, "nextNum": 1 }
                }
            }
        }))
        .unwrap();
    session.handshake().await.expect("handshake");
    assert_eq!(session.author_id().as_str(), "a.author-xyz");

    // Build a one-char Insert changeset (the simplest case that the user
    // might trigger by typing a single key).
    let cs = Changeset {
        old_len: 5,
        net_delta: 1,
        ops: vec![
            Op {
                opcode: OpCode::Keep,
                chars: 5,
                lines: 0,
                attribs: vec![],
            },
            Op {
                opcode: OpCode::Insert,
                chars: 1,
                lines: 0,
                attribs: vec![],
            },
        ],
        char_bank: "X".into(),
    };
    session.send_changeset(&cs).await.expect("send");

    let sent = sent_view.lock().await;
    let (_event, payload) = sent.last().expect("must have sent at least one frame");

    // Wire structure: { type: COLLABROOM, data: { type: USER_CHANGES, ... } }
    let data = &payload["data"];
    assert_eq!(data["type"], "USER_CHANGES");

    // The changeset wire MUST contain the author attribute marker.
    let cs_wire = data["changeset"].as_str().expect("changeset is a string");
    assert!(
        cs_wire.contains("*0"),
        "outbound changeset wire missing *0 author attribute marker: {cs_wire}"
    );

    // The apool MUST include the author binding.
    let apool = &data["apool"];
    let author_binding = &apool["numToAttrib"]["0"];
    assert_eq!(
        author_binding[0], "author",
        "apool entry 0 must be the 'author' attribute, got: {apool:?}"
    );
    assert_eq!(
        author_binding[1], "a.author-xyz",
        "apool author value must match session author id, got: {apool:?}"
    );
    assert_eq!(
        apool["nextNum"], 1,
        "apool nextNum must be 1 when only the author attribute is registered"
    );
}
