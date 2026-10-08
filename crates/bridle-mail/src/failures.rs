use std::collections::HashSet;
use std::sync::Mutex;

/// Keeps a failure that repeats every poll from printing every poll: the first failure of a
/// kind logs at warn, repeats at debug, and the pass that clears it logs one info line.
#[derive(Default)]
pub struct Failures {
    failing: Mutex<HashSet<String>>,
}

impl Failures {
    pub fn failed(&self, kind: &str, err: &anyhow::Error) {
        if self
            .failing
            .lock()
            .expect("failures lock")
            .insert(kind.to_string())
        {
            tracing::warn!("{kind} failed: {err:#}");
        } else {
            tracing::debug!("{kind} failed again: {err:#}");
        }
    }

    pub fn ok(&self, kind: &str) {
        if self.failing.lock().expect("failures lock").remove(kind) {
            tracing::info!("{kind} works again");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[derive(Clone, Default)]
    struct Buf(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for Buf {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn a_repeated_failure_warns_once_and_again_only_after_clearing() {
        let buf = Buf::default();
        let writer = buf.clone();
        let sub = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_ansi(false)
            .with_writer(move || writer.clone())
            .finish();
        let f = Failures::default();
        let err = anyhow::anyhow!("ses refused");
        tracing::subscriber::with_default(sub, || {
            f.failed("outbound", &err);
            f.failed("outbound", &err);
            f.failed("outbound", &err);
            f.ok("outbound");
            f.ok("outbound");
            f.failed("outbound", &err);
        });
        let out = String::from_utf8(buf.0.lock().unwrap().clone()).unwrap();
        assert_eq!(out.matches("WARN").count(), 2, "{out}");
        assert_eq!(out.matches("works again").count(), 1, "{out}");
        assert_eq!(
            out.matches("failed again").count(),
            0,
            "debug is filtered: {out}"
        );
    }
}
