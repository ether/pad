use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub type BufferId = Uuid;

#[derive(Debug, Serialize, Deserialize, Default)]
struct Meta {
    file_path: Option<PathBuf>,
    /// Filled in by Plan 3.
    last_share_remote: Option<String>,
    last_pad_id: Option<String>,
}

pub struct SidecarHandle {
    state_root: PathBuf,
    buffer_id: BufferId,
    meta: Meta,
}

impl SidecarHandle {
    pub fn new_untitled(state_root: &Path) -> anyhow::Result<Self> {
        let buffer_id = Uuid::now_v7();
        let sc = Self {
            state_root: state_root.to_path_buf(),
            buffer_id,
            meta: Meta::default(),
        };
        fs::create_dir_all(sc.dir())?;
        sc.flush_meta()?;
        Ok(sc)
    }

    pub fn for_file(state_root: &Path, file_path: &Path) -> anyhow::Result<Self> {
        let pointer = sibling_meta_path(file_path);
        let buffer_id = if pointer.exists() {
            let raw = fs::read_to_string(&pointer)?;
            Uuid::parse_str(raw.trim())
                .map_err(|e| anyhow::anyhow!("invalid sibling pointer: {e}"))?
        } else {
            Uuid::now_v7()
        };
        let sc = Self {
            state_root: state_root.to_path_buf(),
            buffer_id,
            meta: Meta {
                file_path: Some(file_path.to_path_buf()),
                ..Default::default()
            },
        };
        fs::create_dir_all(sc.dir())?;
        sc.flush_meta()?;
        let _ = fs::write(&pointer, buffer_id.to_string());
        Ok(sc)
    }

    pub fn reattach(state_root: &Path, buffer_id: &BufferId) -> anyhow::Result<Self> {
        let dir = state_root.join(buffer_id.to_string());
        if !dir.exists() {
            anyhow::bail!("no sidecar for buffer-id {}", buffer_id);
        }
        let meta_raw = fs::read_to_string(dir.join("meta.json"))?;
        let meta: Meta = serde_json::from_str(&meta_raw)?;
        Ok(Self {
            state_root: state_root.to_path_buf(),
            buffer_id: *buffer_id,
            meta,
        })
    }

    pub fn buffer_id(&self) -> BufferId {
        self.buffer_id
    }

    pub fn dir(&self) -> PathBuf {
        self.state_root.join(self.buffer_id.to_string())
    }

    pub fn pending_log_path(&self) -> PathBuf {
        self.dir().join("pending.log")
    }

    pub fn file_path(&self) -> Option<&Path> {
        self.meta.file_path.as_deref()
    }

    pub fn set_file_path(&mut self, p: PathBuf) -> anyhow::Result<()> {
        self.meta.file_path = Some(p);
        self.flush_meta()
    }

    fn flush_meta(&self) -> anyhow::Result<()> {
        let path = self.dir().join("meta.json");
        let raw = serde_json::to_string_pretty(&self.meta)?;
        fs::write(path, raw)?;
        Ok(())
    }

    /// Snapshot the buffer's current text to `pre-share-<ts>.snapshot`. Called
    /// just before an `M-S` share attaches to a remote so the user can always
    /// recover their pre-share content via `pad --restore`.
    pub fn pre_share_snapshot(&self, buf: &crate::buffer::Buffer) -> anyhow::Result<()> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let path = self.dir().join(format!("pre-share-{ts}.snapshot"));
        fs::write(path, buf.text())?;
        Ok(())
    }

    /// Snapshot the buffer's current text to `pre-merge.snapshot`. Overwrites
    /// any previous one. Called when an inbound remote changeset would land on
    /// top of a long pending-outbound queue, as a safety net against bad OT
    /// rebases.
    pub fn pre_merge_snapshot(&self, buf: &crate::buffer::Buffer) -> anyhow::Result<()> {
        let path = self.dir().join("pre-merge.snapshot");
        fs::write(path, buf.text())?;
        Ok(())
    }
}

fn sibling_meta_path(file_path: &Path) -> PathBuf {
    let parent = file_path.parent().unwrap_or_else(|| Path::new("."));
    let name = file_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "untitled".to_string());
    parent.join(format!(".pad.{name}.meta"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PendingEntry {
    Insert { offset: u32, text: String },
    Delete { offset: u32, len: u32 },
}

pub struct PendingLog {
    file: std::fs::File,
}

impl PendingLog {
    pub fn open(sc: &SidecarHandle) -> anyhow::Result<Self> {
        let path = sc.pending_log_path();
        // We deliberately do NOT use `.append(true)` here even though
        // this is an append-only log. On Windows, Rust's OpenOptions
        // strips FILE_WRITE_DATA from the access mask whenever append
        // is set, which makes `SetEndOfFile` (the syscall under
        // `set_len(0)` in `truncate()` below) fail with "Access is
        // denied (os error 5)". Instead we open plain `write` and seek
        // to EOF before each append — there's only one writer per
        // sidecar (one pad process owns one buffer), so the lack of
        // atomic O_APPEND semantics doesn't matter.
        // `.truncate(false)` makes clippy's `suspicious_open_options`
        // happy and codifies intent: existing log content survives the
        // open (we only zero it explicitly via `truncate()` below).
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(path)?;
        file.seek(SeekFrom::End(0))?;
        Ok(Self { file })
    }

    pub fn append(&mut self, entry: &PendingEntry) -> anyhow::Result<()> {
        // Seek to end every time so a prior `truncate()` (which leaves
        // the cursor where it was, beyond the new len) doesn't write
        // into the middle of a now-shorter file.
        self.file.seek(SeekFrom::End(0))?;
        let mut line = serde_json::to_string(entry)?;
        line.push('\n');
        self.file.write_all(line.as_bytes())?;
        self.file.sync_data()?;
        Ok(())
    }

    pub fn truncate(&mut self) -> anyhow::Result<()> {
        self.file.set_len(0)?;
        self.file.sync_data()?;
        Ok(())
    }

    pub fn read_all(sc: &SidecarHandle) -> anyhow::Result<Vec<PendingEntry>> {
        let path = sc.pending_log_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let f = std::fs::File::open(path)?;
        let reader = BufReader::new(f);
        let mut entries = Vec::new();
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<PendingEntry>(&line) {
                Ok(e) => entries.push(e),
                Err(_) => break,
            }
        }
        Ok(entries)
    }
}
