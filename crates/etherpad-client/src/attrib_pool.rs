//! Numeric ID ↔ (key, value) attribute string pool.
//!
//! Wire format (JSON):
//! ```json
//! { "numToAttrib": { "0": ["author", "a.xyz"], "1": ["bold", "true"] }, "nextNum": 2 }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttribPool {
    #[serde(rename = "numToAttrib")]
    pub num_to_attrib: BTreeMap<String, (String, String)>,
    #[serde(rename = "nextNum")]
    pub next_num: u32,
}

impl AttribPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, n: u32) -> Option<&(String, String)> {
        self.num_to_attrib.get(&n.to_string())
    }

    pub fn put_attrib(&mut self, key: String, value: String) -> u32 {
        for (k, v) in &self.num_to_attrib {
            if v.0 == key && v.1 == value {
                return k.parse().expect("attrib pool keys are u32");
            }
        }
        let n = self.next_num;
        self.num_to_attrib.insert(n.to_string(), (key, value));
        self.next_num += 1;
        n
    }
}
