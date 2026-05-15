//! Unit-level test: PadSession::handshake against a MockSocket.
#![cfg(feature = "mock-socket")]

use etherpad_client::session::{PadSession, SessionConfig};
use etherpad_client::socket::mock::MockSocket;
use serde_json::json;

#[tokio::test]
async fn client_ready_handshake_consumes_client_vars() {
    let socket = MockSocket::new();
    let sent_view = socket.sent.clone();
    let injector = socket.injector.clone();

    let cfg = SessionConfig {
        pad_id: "myPad".into(),
        token: "t.abc".into(),
        protocol_version: 2,
    };
    let mut session = PadSession::new(Box::new(socket), cfg);

    // Pre-queue the CLIENT_VARS reply.
    let reply = json!({
        "type": "CLIENT_VARS",
        "data": {
            "padId": "myPad",
            "userId": "a.author1",
            "userColor": 3,
            "collab_client_vars": {
                "rev": 0,
                "initialAttributedText": { "text": "hello world\n", "attribs": "|1+c" },
                "apool": { "numToAttrib": {}, "nextNum": 0 }
            }
        }
    });
    injector.send(reply).unwrap();

    session.handshake().await.expect("handshake");
    assert_eq!(session.initial_text(), "hello world\n");
    assert_eq!(session.author_id().as_str(), "a.author1");
    assert_eq!(session.rev(), 0);

    let sent = sent_view.lock().await;
    assert_eq!(sent.len(), 1);
    let (evt, payload) = &sent[0];
    assert_eq!(evt, "message");
    assert_eq!(payload["component"], "pad");
    assert_eq!(payload["type"], "CLIENT_READY");
    assert_eq!(payload["padId"], "myPad");
    assert_eq!(payload["token"], "t.abc");
}
