//! Typed representation of an Etherpad changeset.
//!
//! Wire format: `Z:<old_len>(<>|<|=)<delta>(<ops>)$<char_bank>`
//! Ops: `*<n>` attribute, `|<n>` newlines, `+<n>` insert n chars from bank,
//!      `-<n>` delete n chars, `=<n>` keep n chars. All numbers in base36.
//!
//! See `https://github.com/ether/etherpad-lite/blob/develop/src/static/js/Changeset.js`.

pub mod parser;
pub mod serializer;

use crate::attrib_pool::AttribPool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Insert, // +
    Delete, // -
    Keep,   // =
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Op {
    pub opcode: OpCode,
    /// Number of characters this op operates on.
    pub chars: u32,
    /// Number of newlines in this op's run.
    pub lines: u32,
    /// Attribute IDs (base36-decoded) applied to this op.
    pub attribs: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Changeset {
    /// Length of the document this changeset applies to, in chars.
    pub old_len: u32,
    /// Net change in length. Positive = doc grows.
    pub net_delta: i64,
    pub ops: Vec<Op>,
    pub char_bank: String,
}

impl Changeset {
    pub fn new_len(&self) -> u32 {
        (self.old_len as i64 + self.net_delta) as u32
    }

    /// True if applying this changeset to any text-of-length-old_len is a no-op.
    pub fn is_identity(&self) -> bool {
        self.net_delta == 0
            && self.char_bank.is_empty()
            && self.ops.iter().all(|op| matches!(op.opcode, OpCode::Keep))
    }
}

/// A changeset paired with the attribute pool it references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangesetWithPool {
    pub cs: Changeset,
    pub pool: AttribPool,
}
