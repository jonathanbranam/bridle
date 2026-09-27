//! Bridle daemon API: wire types shared by the daemon and every client, plus
//! an async HTTP/SSE client and daemon discovery. See docs/agent-host.md §6.

pub mod client;
pub mod discovery;
pub mod types;

pub use client::{Client, ClientError};
pub use types::*;
