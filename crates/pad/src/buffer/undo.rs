use super::CursorPos;
use ropey::Rope;

#[derive(Clone)]
pub struct Snapshot {
    pub rope: Rope,
    pub cursor: CursorPos,
}

#[derive(Default)]
pub struct UndoStack {
    pub past: Vec<Snapshot>,
    pub future: Vec<Snapshot>,
    pub cap: usize,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            past: Vec::new(),
            future: Vec::new(),
            cap: 200,
        }
    }

    pub fn push(&mut self, snap: Snapshot) {
        self.past.push(snap);
        if self.past.len() > self.cap {
            self.past.remove(0);
        }
        self.future.clear();
    }
}
