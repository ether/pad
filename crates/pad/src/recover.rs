use crate::buffer::Buffer;
use crate::buffer::sidecar::{BufferId, PendingEntry, PendingLog, SidecarHandle};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Recoverable {
    pub buffer_id: BufferId,
    pub entry_count: usize,
    pub file_path: Option<PathBuf>,
}

pub fn list_recoverable(state_root: &Path) -> anyhow::Result<Vec<Recoverable>> {
    let mut out = Vec::new();
    if !state_root.exists() {
        return Ok(out);
    }
    for entry in fs::read_dir(state_root)? {
        let entry = entry?;
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let Some(name) = dir.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let Ok(id) = uuid::Uuid::parse_str(name) else {
            continue;
        };
        let sc = match SidecarHandle::reattach(state_root, &id) {
            Ok(sc) => sc,
            Err(_) => continue,
        };
        let entries = PendingLog::read_all(&sc)?;
        if !entries.is_empty() {
            out.push(Recoverable {
                buffer_id: id,
                entry_count: entries.len(),
                file_path: sc.file_path().map(|p| p.to_path_buf()),
            });
        }
    }
    Ok(out)
}

pub fn replay_into_buffer(sc: &SidecarHandle) -> anyhow::Result<Buffer> {
    let mut buf = if let Some(p) = sc.file_path() {
        Buffer::load_from_file(p)?
    } else {
        Buffer::empty()
    };
    let entries = PendingLog::read_all(sc)?;
    for e in entries {
        match e {
            PendingEntry::Insert { offset, text } => {
                buf.move_cursor_to(offset_to_pos(&buf, offset as usize));
                buf.insert_str(&text);
            }
            PendingEntry::Delete { offset, len } => {
                buf.move_cursor_to(offset_to_pos(&buf, offset as usize));
                for _ in 0..len {
                    buf.delete_char_forward();
                }
            }
        }
    }
    Ok(buf)
}

fn offset_to_pos(buf: &Buffer, off: usize) -> crate::buffer::CursorPos {
    let mut remaining = off;
    for line_idx in 0..buf.line_count() {
        let line = buf.line(line_idx);
        let line_chars = line.chars().count() + 1;
        if remaining < line_chars {
            return crate::buffer::CursorPos {
                line: line_idx,
                col: remaining.min(line.chars().count()),
            };
        }
        remaining -= line_chars;
    }
    crate::buffer::CursorPos {
        line: buf.line_count() - 1,
        col: buf.line(buf.line_count() - 1).chars().count(),
    }
}

pub fn run(state_root: &Path) -> anyhow::Result<()> {
    let candidates = list_recoverable(state_root)?;
    if candidates.is_empty() {
        println!("No recoverable buffers found.");
        return Ok(());
    }
    println!("Recoverable buffers:");
    for (i, r) in candidates.iter().enumerate() {
        let label = r
            .file_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(untitled)".to_string());
        println!(
            "  [{}] {} — {} pending entries",
            i + 1,
            label,
            r.entry_count
        );
    }
    println!("\nSelect a buffer to resume (number, or q to quit):");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim();
    if input.eq_ignore_ascii_case("q") {
        return Ok(());
    }
    let idx: usize = input.parse()?;
    let chosen = &candidates[idx - 1];
    let sc = SidecarHandle::reattach(state_root, &chosen.buffer_id)?;
    let buf = replay_into_buffer(&sc)?;
    println!(
        "Recovered {} chars. Re-open with:",
        buf.text().chars().count()
    );
    if let Some(p) = chosen.file_path.as_ref() {
        println!("  pad {}", p.display());
    } else {
        println!("  pad   # (then save with ^O to give it a name)");
    }
    Ok(())
}
