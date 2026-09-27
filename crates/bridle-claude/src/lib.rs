//! bridle-claude: a client for one headless `claude -p` process driven over
//! stream-json. This crate has no knowledge of the bridle daemon — it just
//! spawns and drives a single agent process. See docs/agent-host.md §4 for
//! the protocol it implements, and
//! docs/spikes/01-stream-json-findings.md for the evidence behind it.
//!
//! - [`command`]: builds the argv/env for a `claude` invocation.
//! - [`process`]: spawns it and hands back a driveable [`process::AgentHandle`]
//!   plus an event stream and exit outcome.
//! - [`events`]: the tolerant stdout event model.
//! - [`transcript`]: the JSONL record of everything sent and received.

pub mod command;
pub mod events;
pub mod process;
pub mod transcript;

pub use command::{ClaudeCommand, Session};
pub use events::{Event, EventKind};
pub use process::{AgentHandle, ExitOutcome, Spawned, spawn};
pub use transcript::Transcript;
