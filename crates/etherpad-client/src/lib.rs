//! Reusable client for Etherpad's socket.io + changeset protocol.
//!
//! See `README.md` for usage. Conformance with Etherpad's JS reference is
//! verified by the test suite under `tests/conformance/`.

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod changeset;
pub mod error;
pub mod ot;
pub mod presence;
pub mod session;
pub mod socket;

mod attrib_pool;
mod reconnect;

pub use error::ClientError;
// pub use session::PadSession; // lands with Task 12
// pub use reconnect::Reconnect; // lands with Task 13
