use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuthorId(pub String);

impl AuthorId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 0..=6 — 7 distinct foreground colors matching Etherpad's palette.
/// Derived deterministically from `hash(AuthorId) mod 7`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColorId(pub u8);

impl ColorId {
    pub fn from_author(author: &AuthorId) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        author.hash(&mut h);
        ColorId((h.finish() % 7) as u8)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorInfo {
    pub author_id: AuthorId,
    pub display_name: Option<String>,
    pub color: ColorId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorPos {
    /// 0-based char offset into the document.
    pub offset: u32,
}

#[derive(Debug, Clone)]
pub enum PresenceEvent {
    Join(AuthorInfo),
    Leave(AuthorId),
    Cursor { author: AuthorId, pos: CursorPos },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_is_deterministic() {
        let a = AuthorId::new("a.xyz");
        let c1 = ColorId::from_author(&a);
        let c2 = ColorId::from_author(&a);
        assert_eq!(c1, c2);
        assert!(c1.0 < 7);
    }

    #[test]
    fn different_authors_can_collide_but_distribute() {
        let mut buckets = [0u32; 7];
        for i in 0..100 {
            let id = AuthorId::new(format!("a.author{i}"));
            let c = ColorId::from_author(&id);
            buckets[c.0 as usize] += 1;
        }
        for b in buckets {
            assert!(b > 0, "every color bucket should see at least one hit over 100 authors");
        }
    }
}
