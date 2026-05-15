pub mod clipboard;
pub mod line_endings;
pub mod search;
pub mod sidecar;
pub mod undo;
pub use line_endings::LineEnding;

use clipboard::LineClipboard;
use ropey::Rope;
use std::path::Path;
use undo::{Snapshot, UndoStack};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPos {
    pub line: usize,
    pub col: usize,
}

pub struct Buffer {
    rope: Rope,
    cursor: CursorPos,
    dirty: bool,
    pref_col: usize,
    line_ending: LineEnding,
    clipboard: LineClipboard,
    undo: UndoStack,
}

impl Buffer {
    pub fn empty() -> Self {
        Self {
            rope: Rope::new(),
            cursor: CursorPos { line: 0, col: 0 },
            dirty: false,
            pref_col: 0,
            line_ending: LineEnding::Lf,
            clipboard: LineClipboard::default(),
            undo: UndoStack::new(),
        }
    }

    pub fn from_text(s: &str) -> Self {
        let normalized = s.replace("\r\n", "\n").replace('\r', "\n");
        Self {
            rope: Rope::from_str(&normalized),
            cursor: CursorPos { line: 0, col: 0 },
            dirty: false,
            pref_col: 0,
            line_ending: LineEnding::Lf,
            clipboard: LineClipboard::default(),
            undo: UndoStack::new(),
        }
    }

    pub fn from_text_with_ending(s: &str) -> Self {
        let ending = LineEnding::detect(s);
        let normalized = s.replace("\r\n", "\n").replace('\r', "\n");
        Self {
            rope: Rope::from_str(&normalized),
            cursor: CursorPos { line: 0, col: 0 },
            dirty: false,
            pref_col: 0,
            line_ending: ending,
            clipboard: LineClipboard::default(),
            undo: UndoStack::new(),
        }
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn cursor(&self) -> CursorPos {
        self.cursor
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines().max(1)
    }

    pub fn line(&self, idx: usize) -> String {
        if idx >= self.rope.len_lines() {
            return String::new();
        }
        let l = self.rope.line(idx);
        let s = l.to_string();
        s.strip_suffix('\n').map(|x| x.to_string()).unwrap_or(s)
    }

    pub fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    pub fn serialize_for_save(&self) -> String {
        let text = self.rope.to_string();
        match self.line_ending {
            LineEnding::Lf => text,
            LineEnding::Crlf => text.replace('\n', "\r\n"),
            LineEnding::Cr => text.replace('\n', "\r"),
        }
    }

    pub fn move_cursor_to(&mut self, pos: CursorPos) {
        self.cursor = self.clamp(pos);
        self.pref_col = self.cursor.col;
    }

    pub fn move_left(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        } else if self.cursor.line > 0 {
            self.cursor.line -= 1;
            self.cursor.col = self.line(self.cursor.line).chars().count();
        }
        self.pref_col = self.cursor.col;
    }

    pub fn move_right(&mut self) {
        let line_len = self.line(self.cursor.line).chars().count();
        if self.cursor.col < line_len {
            self.cursor.col += 1;
        } else if self.cursor.line + 1 < self.line_count() {
            self.cursor.line += 1;
            self.cursor.col = 0;
        }
        self.pref_col = self.cursor.col;
    }

    pub fn move_up(&mut self) {
        if self.cursor.line == 0 {
            return;
        }
        self.cursor.line -= 1;
        let line_len = self.line(self.cursor.line).chars().count();
        self.cursor.col = self.pref_col.min(line_len);
    }

    pub fn move_down(&mut self) {
        if self.cursor.line + 1 >= self.line_count() {
            return;
        }
        self.cursor.line += 1;
        let line_len = self.line(self.cursor.line).chars().count();
        self.cursor.col = self.pref_col.min(line_len);
    }

    pub fn insert_char(&mut self, c: char) {
        let char_idx = self.cursor_char_idx();
        self.rope.insert_char(char_idx, c);
        self.dirty = true;
        if c == '\n' {
            self.cursor.line += 1;
            self.cursor.col = 0;
        } else {
            self.cursor.col += 1;
        }
        self.pref_col = self.cursor.col;
    }

    pub fn insert_str(&mut self, s: &str) {
        for c in s.chars() {
            self.insert_char(c);
        }
    }

    pub fn backspace(&mut self) {
        let char_idx = self.cursor_char_idx();
        if char_idx == 0 {
            return;
        }
        let prev = self.rope.char(char_idx - 1);
        // Capture previous line's length BEFORE mutation when we'll need it.
        let prev_line_len = if prev == '\n' && self.cursor.line > 0 {
            self.line(self.cursor.line - 1).chars().count()
        } else {
            0
        };
        self.rope.remove(char_idx - 1..char_idx);
        self.dirty = true;
        if prev == '\n' {
            self.cursor.line -= 1;
            self.cursor.col = prev_line_len;
        } else {
            self.cursor.col -= 1;
        }
        self.pref_col = self.cursor.col;
    }

    pub fn delete_char_forward(&mut self) {
        let char_idx = self.cursor_char_idx();
        if char_idx >= self.rope.len_chars() {
            return;
        }
        self.rope.remove(char_idx..char_idx + 1);
        self.dirty = true;
    }

    pub(crate) fn cursor_char_idx(&self) -> usize {
        let line_start = self.rope.line_to_char(self.cursor.line);
        let line = self.line(self.cursor.line);
        let col_chars = line.chars().take(self.cursor.col).count();
        line_start + col_chars
    }

    fn clamp(&self, pos: CursorPos) -> CursorPos {
        let line = pos.line.min(self.line_count().saturating_sub(1));
        let max_col = self.line(line).chars().count();
        CursorPos {
            line,
            col: pos.col.min(max_col),
        }
    }

    pub fn cursor_offset(&self) -> u32 {
        self.cursor_char_idx() as u32
    }

    pub fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        match std::fs::read(path) {
            Ok(bytes) => {
                let text = std::str::from_utf8(&bytes)
                    .map_err(|e| anyhow::anyhow!("file is not valid UTF-8: {e}"))?;
                Ok(Self::from_text_with_ending(text))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::empty()),
            Err(e) => Err(anyhow::anyhow!("read failed: {e}")),
        }
    }

    pub fn save_to_file(&mut self, path: &Path) -> anyhow::Result<()> {
        let text = self.serialize_for_save();
        std::fs::write(path, text).map_err(|e| anyhow::anyhow!("write failed: {e}"))?;
        self.mark_clean();
        Ok(())
    }

    pub fn cut_line(&mut self) {
        let line_idx = self.cursor.line;
        if line_idx >= self.rope.len_lines() {
            return;
        }
        let line_start = self.rope.line_to_char(line_idx);
        let line_end = if line_idx + 1 < self.rope.len_lines() {
            self.rope.line_to_char(line_idx + 1)
        } else {
            self.rope.len_chars()
        };
        if line_start == line_end {
            return;
        }
        let cut: String = self.rope.slice(line_start..line_end).into();
        self.clipboard.last_cut = Some(cut);
        self.rope.remove(line_start..line_end);
        self.dirty = true;
        if self.cursor.line >= self.line_count() && self.cursor.line > 0 {
            self.cursor.line -= 1;
        }
        self.cursor.col = 0;
        self.pref_col = 0;
    }

    pub fn uncut(&mut self) {
        let Some(text) = self.clipboard.last_cut.clone() else {
            return;
        };
        let line_start = self.rope.line_to_char(self.cursor.line);
        self.rope.insert(line_start, &text);
        self.dirty = true;
        let lines_in_paste = text.matches('\n').count();
        if lines_in_paste > 0 {
            self.cursor.line += lines_in_paste;
            self.cursor.col = 0;
        } else {
            self.cursor.col = text.chars().count();
        }
        self.pref_col = self.cursor.col;
    }

    pub fn search_forward(&self, needle: &str) -> Option<CursorPos> {
        let start = self.cursor_char_idx();
        search::search_forward(&self.rope, needle, start)
    }

    pub fn replace_one(&mut self, needle: &str, replacement: &str) -> bool {
        let Some(pos) = self.search_forward(needle) else {
            return false;
        };
        let start_char = self.rope.line_to_char(pos.line) + pos.col;
        let needle_chars = needle.chars().count();
        self.rope.remove(start_char..start_char + needle_chars);
        self.rope.insert(start_char, replacement);
        self.dirty = true;
        true
    }

    pub fn replace_all(&mut self, needle: &str, replacement: &str) -> usize {
        if needle.is_empty() {
            return 0;
        }
        let saved = self.cursor;
        self.move_cursor_to(CursorPos { line: 0, col: 0 });
        let mut count = 0;
        while self.replace_one(needle, replacement) {
            count += 1;
        }
        self.move_cursor_to(saved);
        count
    }

    pub fn snapshot_for_undo(&mut self) {
        self.undo.push(Snapshot {
            rope: self.rope.clone(),
            cursor: self.cursor,
        });
    }

    pub fn undo(&mut self) {
        let Some(prev) = self.undo.past.pop() else {
            return;
        };
        let now = Snapshot {
            rope: self.rope.clone(),
            cursor: self.cursor,
        };
        self.undo.future.push(now);
        self.rope = prev.rope;
        self.cursor = prev.cursor;
        self.pref_col = self.cursor.col;
        self.dirty = true;
    }

    pub fn redo(&mut self) {
        let Some(next) = self.undo.future.pop() else {
            return;
        };
        let now = Snapshot {
            rope: self.rope.clone(),
            cursor: self.cursor,
        };
        self.undo.past.push(now);
        self.rope = next.rope;
        self.cursor = next.cursor;
        self.pref_col = self.cursor.col;
        self.dirty = true;
    }
}
