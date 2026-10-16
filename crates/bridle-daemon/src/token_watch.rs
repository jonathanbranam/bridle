//! br-36a3: role tokens the credentials file holds but the database rejects (a database replaced
//! or rebuilt under a running daemon). One incident names the fix, `bridle token pair`, whether
//! it is found at start or as a burst of 401s at runtime. Random-token 401s never count: only a
//! token the credentials file holds for this project does.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::orchestrator::Incidents;

/// How many distinct held role tokens must be rejected inside [`WINDOW`] to raise.
const THRESHOLD: usize = 3;
const WINDOW: Duration = Duration::from_secs(5 * 60);
/// After raising, stay quiet this long.
const QUIET: Duration = Duration::from_secs(60 * 60);

pub fn incident_text(roles: &str) -> String {
    format!(
        "Role tokens rejected: the credentials file holds tokens for {roles} that this \
         daemon's database does not accept (a database replaced or rebuilt?). Those roles get a \
         bare 401. Run `bridle token pair` to mint them again."
    )
}

/// Tells the human and the orchestrator (the daemon's existing incident notice, not a ticket).
pub async fn raise(incidents: &impl Incidents, roles: &str) {
    let text = incident_text(roles);
    incidents.record(&text).await;
    incidents.ask_orchestrator(&text).await;
}

#[derive(Default)]
pub struct TokenWatch(Mutex<Inner>);

#[derive(Default)]
struct Inner {
    rejected: Vec<(String, Instant)>,
    human_ok: Option<Instant>,
    raised: Option<Instant>,
}

impl TokenWatch {
    /// The human token authenticated: the database is the right one and only the roles are lost.
    pub fn human_authenticated(&self, now: Instant) {
        if let Ok(mut g) = self.0.lock() {
            g.human_ok = Some(now);
        }
    }

    /// A held role token was rejected. Returns the roles to name when this tips the count over
    /// the threshold (and not again for [`QUIET`]).
    pub fn role_rejected(&self, role: &str, now: Instant) -> Option<String> {
        let mut g = self.0.lock().ok()?;
        g.rejected.retain(|(_, t)| now.duration_since(*t) < WINDOW);
        match g.rejected.iter_mut().find(|(r, _)| r == role) {
            Some(e) => e.1 = now,
            None => g.rejected.push((role.to_string(), now)),
        }
        let human_ok = g.human_ok.is_some_and(|t| now.duration_since(t) < WINDOW);
        let quiet = g.raised.is_some_and(|t| now.duration_since(t) < QUIET);
        if g.rejected.len() < THRESHOLD || !human_ok || quiet {
            return None;
        }
        g.raised = Some(now);
        let mut roles: Vec<&str> = g.rejected.iter().map(|(r, _)| r.as_str()).collect();
        roles.sort_unstable();
        Some(roles.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_names_the_roles_and_the_fix() {
        let t = incident_text("aide, advisor");
        assert!(t.contains("aide, advisor") && t.contains("`bridle token pair`"));
    }

    #[test]
    fn burst_of_three_held_roles_raises_once() {
        let w = TokenWatch::default();
        let t = Instant::now();
        w.human_authenticated(t);
        assert_eq!(w.role_rejected("aide", t), None);
        assert_eq!(w.role_rejected("aide", t), None, "same role twice is one");
        assert_eq!(w.role_rejected("advisor", t), None);
        assert_eq!(
            w.role_rejected("orchestrator", t).as_deref(),
            Some("advisor, aide, orchestrator")
        );
        assert_eq!(w.role_rejected("orchestrator", t), None, "deduped");
    }

    #[test]
    fn needs_the_human_token_and_the_window() {
        let w = TokenWatch::default();
        let t = Instant::now();
        for r in ["a", "b", "c"] {
            assert_eq!(w.role_rejected(r, t), None, "human never authenticated");
        }
        w.human_authenticated(t);
        let later = t + WINDOW + Duration::from_secs(1);
        assert_eq!(w.role_rejected("d", later), None, "old rejections aged out");
    }
}
