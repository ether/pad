use pad::panic_hook::{PanicSink, install_panic_hook};
use std::sync::{Arc, Mutex};

#[test]
fn panic_hook_writes_to_sink() {
    let captured: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink: PanicSink = {
        let c = captured.clone();
        Box::new(move |s: &str| c.lock().unwrap().push(s.to_string()))
    };
    install_panic_hook(sink);

    let _ = std::panic::catch_unwind(|| panic!("boom"));

    let dumped = captured.lock().unwrap();
    assert!(
        dumped.iter().any(|s| s.contains("boom")),
        "panic message captured: {:?}",
        *dumped
    );
}
