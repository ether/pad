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
    buf.move_cursor_to(CursorPos { line: 0, col: 3 });
    let remote = parse("Z:5>1+1=5$Z").unwrap();
    apply_remote(&mut buf, &remote, &empty_queue()).unwrap();
    assert_eq!(buf.text(), "Zhello");
    assert_eq!(buf.cursor(), CursorPos { line: 0, col: 4 });
}

#[test]
fn remote_insert_after_cursor_leaves_cursor() {
    let mut buf = Buffer::from_text("hello");
    buf.move_cursor_to(CursorPos { line: 0, col: 2 });
    let remote = parse("Z:5>1=5+1$!").unwrap();
    apply_remote(&mut buf, &remote, &empty_queue()).unwrap();
    assert_eq!(buf.text(), "hello!");
    assert_eq!(buf.cursor(), CursorPos { line: 0, col: 2 });
}

#[test]
fn remote_delete_before_cursor_shifts_cursor_back() {
    let mut buf = Buffer::from_text("hello");
    buf.move_cursor_to(CursorPos { line: 0, col: 4 });
    let remote = parse("Z:5<2=2-2=1$ll").unwrap();
    apply_remote(&mut buf, &remote, &empty_queue()).unwrap();
    assert_eq!(buf.text(), "heo");
    assert_eq!(buf.cursor(), CursorPos { line: 0, col: 2 });
}
