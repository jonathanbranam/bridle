//! Bridle daemon API: wire types shared by the daemon and every client, plus
//! an async HTTP/SSE client and daemon discovery. See docs/design/agent-host/api.md.

pub mod client;
pub mod config_warn;
pub mod discovery;
pub mod machines;
pub mod types;

pub use client::{Client, ClientError};
pub use types::*;
