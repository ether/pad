pub type PanicSink = Box<dyn Fn(&str) + Send + Sync>;

/// Install a panic hook that restores the terminal and writes the panic stack
/// to the provided sink.
pub fn install_panic_hook(sink: PanicSink) {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();

        let payload = info.payload();
        let msg = if let Some(s) = payload.downcast_ref::<&'static str>() {
            (*s).to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "panic payload not a string".to_string()
        };
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "<unknown>".to_string());
        let dump = format!("PANIC at {location}: {msg}");
        sink(&dump);

        eprintln!("\npad crashed: {msg}\nRun 'pad --recover' to resume.");

        prev(info);
    }));
}

fn restore_terminal() -> std::io::Result<()> {
    use crossterm::execute;
    use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
    let _ = disable_raw_mode();
    let mut stdout = std::io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen);
    Ok(())
}

pub fn file_sink(crash_dir: std::path::PathBuf) -> PanicSink {
    use std::fs::OpenOptions;
    use std::io::Write;
    Box::new(move |s: &str| {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _ = std::fs::create_dir_all(&crash_dir);
        let path = crash_dir.join(format!("crash-{ts}.log"));
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = writeln!(f, "{s}");
            let _ = f.sync_data();
        }
    })
}
