//! Transcript writer/reader: every stdin line written and every stdout/stderr
//! line read, as `{"t_ms":…, "dir":"in|out|err|note", "line":…}` JSONL. This
//! is `.bridle/agents/<id>/transcript.jsonl` in docs/design/agent-host/agents.md.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::Deserialize;

#[derive(Clone)]
pub struct Transcript {
    inner: Arc<Mutex<File>>,
    start: Instant,
}

impl Transcript {
    /// Appends, so a resumed agent's process shares one transcript with the
    /// one before it.
    pub fn open(path: &Path, start: Instant) -> io::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(file)),
            start,
        })
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// `line` is stored as a string (not re-parsed), so the transcript keeps
    /// the exact bytes claude wrote or bridle sent.
    pub fn record(&self, dir: &str, line: &str) {
        let entry = serde_json::json!({ "t_ms": self.elapsed_ms(), "dir": dir, "line": line });
        // A transcript write failure shouldn't take the agent down; the
        // caller has no useful recovery for it either. Lock poisoning is the
        // only realistic failure mode and would mean a prior panic already
        // corrupted this process's state.
        if let Ok(mut f) = self.inner.lock() {
            let _ = writeln!(f, "{entry}");
        }
    }

    /// Host annotations (e.g. "spawn: claude …", "pid 1234"), so the
    /// transcript reads as a timeline even without re-parsing every line.
    pub fn note(&self, text: &str) {
        self.record("note", text);
    }
}

/// One decoded line from a transcript file, numbered from 1.
#[derive(Debug, Clone, PartialEq)]
pub struct TranscriptEntry {
    pub n: usize,
    pub t_ms: u64,
    pub dir: String,
    pub line: String,
}

#[derive(Deserialize)]
struct RawEntry {
    t_ms: u64,
    dir: String,
    line: String,
}

/// Reads transcript entries after line number `since` (1-based; `0` reads
/// from the start), up to `limit` entries. Malformed lines are skipped
/// rather than failing the whole read: the writer controls the format, so a
/// bad line means truncation mid-write, not a caller bug to surface.
pub fn read_lines(path: &Path, since: u64, limit: usize) -> io::Result<Vec<TranscriptEntry>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        let n = i + 1;
        if (n as u64) <= since {
            continue;
        }
        let Ok(raw) = serde_json::from_str::<RawEntry>(&line) else {
            continue;
        };
        out.push(TranscriptEntry {
            n,
            t_ms: raw.t_ms,
            dir: raw.dir,
            line: raw.line,
        });
        if out.len() >= limit {
            break;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn round_trips_and_paginates() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("transcript.jsonl");
        let t = Transcript::open(&path, Instant::now()).expect("open");
        t.note("spawn: claude ...");
        t.record("in", r#"{"type":"user"}"#);
        t.record("out", r#"{"type":"system","subtype":"init"}"#);

        let all = read_lines(&path, 0, 100).expect("read");
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].n, 1);
        assert_eq!(all[0].dir, "note");
        assert_eq!(all[1].dir, "in");
        assert_eq!(all[2].dir, "out");

        let after_first = read_lines(&path, 1, 100).expect("read");
        assert_eq!(after_first.len(), 2);
        assert_eq!(after_first[0].n, 2);

        let limited = read_lines(&path, 0, 1).expect("read");
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].n, 1);
    }

    #[test]
    fn skips_malformed_lines() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("transcript.jsonl");
        std::fs::write(
            &path,
            "not json\n{\"t_ms\":1,\"dir\":\"note\",\"line\":\"ok\"}\n",
        )
        .expect("write");
        let entries = read_lines(&path, 0, 10).expect("read");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].line, "ok");
        assert_eq!(entries[0].n, 2);
    }
}
