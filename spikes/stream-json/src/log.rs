//! Transcript writer: every stdin line written and every stdout/stderr line
//! read, as `{"t_ms":…, "dir":"in|out|err|note", "line":…}` JSONL.

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone)]
pub struct Transcript {
    inner: Arc<Mutex<File>>,
    start: Instant,
}

impl Transcript {
    /// Appends, so several processes in one scenario share one transcript.
    pub fn open(path: &Path, start: Instant) -> anyhow::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self { inner: Arc::new(Mutex::new(file)), start })
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// `line` is stored as a string (not re-parsed) so the fixture keeps the
    /// exact bytes claude emitted.
    pub fn record(&self, dir: &str, line: &str) {
        let entry = serde_json::json!({ "t_ms": self.elapsed_ms(), "dir": dir, "line": line });
        let mut f = self.inner.lock().unwrap();
        let _ = writeln!(f, "{entry}");
    }

    /// Harness annotations (e.g. "sent SIGTERM"), so fixtures read as a timeline.
    pub fn note(&self, text: &str) {
        self.record("note", text);
    }
}
