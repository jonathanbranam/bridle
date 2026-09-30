//! The inbound half of `bridle mail run` (docs/questions/open/email-and-texting-for-bridle-and-track-web-rs7p.md).
//!
//! Kept out of the daemon on purpose: the AWS SDK and the parsing of untrusted MIME live here,
//! and the only thing that reaches the daemon is `POST /v1/messages` as `external:mail`.

mod bridge;
mod config;
mod parse;
mod store;

pub use bridge::{Bridge, ClientSink, Outcome, Sink};
pub use config::MailConfig;
pub use parse::{Accepted, Attachment, Rejection, Route, evaluate};
pub use store::{FakeStore, MailStore, S3Store};
