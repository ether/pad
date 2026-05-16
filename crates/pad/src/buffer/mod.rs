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

    /// Insert a single char at the cursor and advance. Returns
    /// `(rope_offset_of_insert, text_actually_inserted)` so callers building
    /// outbound changesets can use the SAME bytes that landed in the rope —
    /// not the user's raw keystroke. Most of the time `text_actually_inserted`
    /// equals `c.to_string()`, but on the trailing-empty line we insert
    /// `c` followed by `\n` to preserve Etherpad's "doc always ends with \n"
    /// invariant (without that extra `\n`, the typed char strands the
    /// existing trailing `\n` mid-document and the browser's line assembler
    /// asserts "line assembler not finished").
    pub fn insert_char(&mut self, c: char) -> (u32, String) {
        let char_idx = self.cursor_char_idx();
        let total = self.rope.len_chars();
        let on_trailing_empty = c != '\n'
            && total > 0
            && char_idx == total
            && self.rope.char(total - 1) == '\n';
        if on_trailing_empty {
            // Synthesize the missing line terminator so the new content
            // becomes a proper line. The user typed `c`; we durably insert
            // `c\n` so the line carrying `c` ends with a newline AND a fresh
            // trailing-empty line still exists past it.
            self.rope.insert_char(char_idx, c);
            self.rope.insert_char(char_idx + 1, '\n');
            self.dirty = true;
            self.cursor.col = 1;
            self.pref_col = 1;
            let mut text = String::with_capacity(2);
            text.push(c);
            text.push('\n');
            return (char_idx as u32, text);
        }
        self.rope.insert_char(char_idx, c);
        self.dirty = true;
        if c == '\n' {
            self.cursor.line += 1;
            self.cursor.col = 0;
        } else {
            self.cursor.col += 1;
        }
        self.pref_col = self.cursor.col;
        (char_idx as u32, c.to_string())
    }

    /// Insert `s` at the cursor in one shot. Returns
    /// `(rope_offset_of_insert, text_actually_inserted)` — see
    /// [`insert_char`] for why the inserted text can differ from `s`.
    pub fn insert_str(&mut self, s: &str) -> (u32, String) {
        if s.is_empty() {
            return (self.cursor_char_idx() as u32, String::new());
        }
        let mut first_pos: Option<u32> = None;
        let mut acc = String::with_capacity(s.len() + 1);
        for c in s.chars() {
            let (pos, inserted) = self.insert_char(c);
            first_pos.get_or_insert(pos);
            acc.push_str(&inserted);
        }
        (
            first_pos.unwrap_or_else(|| self.cursor_char_idx() as u32),
            acc,
        )
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
        // Don't delete the trailing '\n' if it's the only '\n' the pad has
        // left — Etherpad's "pad always ends with \n" invariant must hold,
        // and downstream browser clients crash in offsetOfEntry on a fully
        // empty rep.lines.
        let total = self.rope.len_chars();
        if char_idx + 1 == total
            && self.rope.char(char_idx) == '\n'
            && !self.rope.slice(0..char_idx).chars().any(|c| c == '\n')
        {
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

    /// Length of the rope in chars (Etherpad changeset units).
    pub fn text_len(&self) -> u32 {
        self.rope.len_chars() as u32
    }

    /// Wipe and replace the rope content. Used by the share layer when
    /// applying a remote changeset whose effect was already computed against
    /// `text()`. Marks the buffer dirty.
    pub fn replace_all_text(&mut self, new_text: &str) {
        self.rope = Rope::from_str(new_text);
        self.dirty = true;
    }

    /// Cut the line under the cursor. Returns `(char_offset_of_cut_start,
    /// cut_text)` so the caller can emit a matching outbound Changeset when
    /// shared. Returns None if there was nothing to cut.
    ///
    /// **Invariant:** if cutting would empty the rope AND the final char is
    /// a `\n`, the trailing `\n` is preserved. Etherpad's pad text invariant
    /// is "always ends with `\n`"; an empty pad makes downstream browser
    /// clients crash in `applyChangesetToDocument:offsetOfEntry` trying to
    /// walk an empty `rep.lines`.
    pub fn cut_line(&mut self) -> Option<(u32, String)> {
        let line_idx = self.cursor.line;
        if line_idx >= self.rope.len_lines() {
            return None;
        }
        let line_start = self.rope.line_to_char(line_idx);
        let raw_end = if line_idx + 1 < self.rope.len_lines() {
            self.rope.line_to_char(line_idx + 1)
        } else {
            self.rope.len_chars()
        };
        if line_start == raw_end {
            return None;
        }
        let would_empty = line_start == 0 && raw_end == self.rope.len_chars();
        let last_is_nl = raw_end > 0 && self.rope.char(raw_end - 1) == '\n';
        let line_end = if would_empty && last_is_nl {
            raw_end - 1
        } else {
            raw_end
        };
        if line_start == line_end {
            return None;
        }
        let cut: String = self.rope.slice(line_start..line_end).into();
        self.clipboard.last_cut = Some(cut.clone());
        self.rope.remove(line_start..line_end);
        self.dirty = true;
        if self.cursor.line >= self.line_count() && self.cursor.line > 0 {
            self.cursor.line -= 1;
        }
        self.cursor.col = 0;
        self.pref_col = 0;
        Some((line_start as u32, cut))
    }

    /// Paste the most recent cut at the start of the current line. Returns
    /// `(char_offset_of_paste_start, pasted_text)` so the caller can emit a
    /// matching outbound Changeset when shared. Returns None if nothing to
    /// paste.
    pub fn uncut(&mut self) -> Option<(u32, String)> {
        let text = self.clipboard.last_cut.clone()?;
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
        Some((line_start as u32, text))
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
