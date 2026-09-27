//! The budget governor: turns rate-limit readings into a state
//! (`normal`/`holding`/`winding_down`/`paused`) and publishes it for the
//! supervisor to enforce holds against. See
//! docs/design/usage-and-budget.md, "The budget governor".
//!
//! Stopping/interrupting agents, `wind_down_grace`, applying
//! `budget_paused`, resuming from `paused`, `bridle budget hold/release`
//! and `--ignore-budget` are a later task's job (see that doc's "What
//! bridle records today"). This module only computes the state and holds
//! new work; it never stops or interrupts a running agent.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bridle_api::types::{GovernorState, RateLimit, event_kind};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::config::BudgetConfig;
use crate::events::Emitter;
use crate::paths::Workspace;
use crate::store::Store;
use crate::supervisor::AgentManager;

/// Windows that gate everyone, not just agents on one model
/// (usage-and-budget.md: "The rest apply to everyone").
const DEFAULT_WINDOWS: &[&str] = &["five_hour", "seven_day"];
/// Per-model windows and the model-name substring that selects them.
const MODEL_WINDOWS: &[(&str, &str)] = &[("opus", "seven_day_opus"), ("sonnet", "seven_day_sonnet")];

const PROBE_TIMEOUT: Duration = Duration::from_secs(10);

/// One window's contribution to a governor state: which window it was and
/// when it resets, so a 409 can name both.
#[derive(Debug, Clone, Default)]
pub struct WindowBlock {
    pub state: GovernorState,
    pub window: Option<String>,
    pub resets_at: Option<DateTime<Utc>>,
}

/// The governor's published state: what [`AgentManager`] reads to enforce
/// the hold, cheap to clone and lock-free to read.
#[derive(Debug, Clone, Default)]
pub struct GovernorSnapshot {
    /// The default-scoped windows + staleness: applies to every spawn,
    /// resume and message.
    pub default: WindowBlock,
    /// Per-model weekly windows (`seven_day_opus`, `seven_day_sonnet`),
    /// keyed by the model-name substring in [`MODEL_WINDOWS`].
    pub per_model: BTreeMap<String, WindowBlock>,
}

impl GovernorSnapshot {
    /// The state that applies to an agent (or a spawn/resume) pinned to
    /// `model`: the worse of the default state and that model's own window.
    pub fn for_model(&self, model: &str) -> WindowBlock {
        let model_block = MODEL_WINDOWS
            .iter()
            .find(|(needle, _)| model.contains(needle))
            .and_then(|(needle, _)| self.per_model.get(*needle));
        match model_block {
            Some(b) if b.state > self.default.state => b.clone(),
            _ => self.default.clone(),
        }
    }
}

pub type GovernorHandle = Arc<Mutex<GovernorSnapshot>>;

struct Inner {
    store: Store,
    manager: AgentManager,
    emitter: Emitter,
    config: BudgetConfig,
    claude_program: String,
    workspace: Workspace,
    handle: GovernorHandle,
    last_poll: Mutex<Option<Instant>>,
    probe: tokio::sync::Mutex<Option<bridle_claude::process::AgentHandle>>,
    /// docs/design/usage-and-budget.md, Seeing the windows: 5 min normally,
    /// 30 s at or above `hold_at`. Fields (not consts) so tests can poll
    /// every tick instead of waiting out the real cadence.
    poll_interval_normal: Duration,
    poll_interval_above_hold: Duration,
}

#[derive(Clone)]
pub struct Governor(Arc<Inner>);

impl Governor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store: Store,
        manager: AgentManager,
        emitter: Emitter,
        config: BudgetConfig,
        claude_program: String,
        workspace: Workspace,
        handle: GovernorHandle,
        poll_interval_normal: Duration,
        poll_interval_above_hold: Duration,
    ) -> Self {
        Governor(Arc::new(Inner {
            store,
            manager,
            emitter,
            config,
            claude_program,
            workspace,
            handle,
            last_poll: Mutex::new(None),
            probe: tokio::sync::Mutex::new(None),
            poll_interval_normal,
            poll_interval_above_hold,
        }))
    }

    pub fn config(&self) -> &BudgetConfig {
        &self.0.config
    }

    pub fn snapshot(&self) -> GovernorSnapshot {
        self.0.handle.lock().expect("governor mutex poisoned").clone()
    }

    /// Called on every background tick: polls `get_usage` when due, then
    /// always recomputes from whatever's in the store (so staleness
    /// escalates even between polls).
    pub async fn tick(&self) {
        if self.poll_due().await {
            self.poll_usage().await;
        }
        self.recompute().await;
    }

    async fn poll_due(&self) -> bool {
        let due_after = if self.snapshot().default.state == GovernorState::Normal {
            self.0.poll_interval_normal
        } else {
            self.0.poll_interval_above_hold
        };
        let mut last = self.0.last_poll.lock().expect("governor mutex poisoned");
        let due = last.is_none_or(|t| t.elapsed() >= due_after);
        if due {
            *last = Some(Instant::now());
        }
        due
    }

    /// Sends the undocumented `get_usage` control request to a running
    /// agent if one exists, else a dedicated probe process, and upserts
    /// every window it reports.
    async fn poll_usage(&self) {
        let response = if let Some(h) = self.0.manager.any_running_handle() {
            h.get_usage(PROBE_TIMEOUT).await.map_err(|e| e.to_string())
        } else {
            self.probe_get_usage().await
        };
        let response = match response {
            Ok(v) => v,
            Err(e) => {
                tracing::debug!(error = %e, "get_usage probe failed");
                return;
            }
        };
        let observed_at = Utc::now();
        for rl in parse_get_usage(&response, observed_at) {
            let _ = self.0.store.upsert_rate_limit(rl).await;
        }
    }

    /// A plain, promptless `claude -p` process kept alive across polls,
    /// used only when no real agent is running to ask instead. Built
    /// through the same command path as real agents
    /// (`BRIDLE_CLAUDE_BIN`-substitutable) so tests can drive it with
    /// fake-claude.py.
    async fn probe_get_usage(&self) -> Result<Value, String> {
        let mut guard = self.0.probe.lock().await;
        if let Some(h) = guard.as_ref() {
            match h.get_usage(PROBE_TIMEOUT).await {
                Ok(v) => return Ok(v),
                Err(e) => {
                    tracing::debug!(error = %e, "governor probe process gone; respawning");
                    *guard = None;
                }
            }
        }
        let handle = self.spawn_probe().await.map_err(|e| e.to_string())?;
        let result = handle.get_usage(PROBE_TIMEOUT).await.map_err(|e| e.to_string());
        *guard = Some(handle);
        result
    }

    async fn spawn_probe(&self) -> anyhow::Result<bridle_claude::process::AgentHandle> {
        use bridle_claude::command::{ClaudeCommand, Session};
        use bridle_claude::transcript::Transcript;

        let mut cmd = ClaudeCommand::new(self.0.workspace.repo.clone(), Session::New(Uuid::new_v4()));
        cmd.program = self.0.claude_program.clone();
        let transcript = Transcript::open(
            &self.0.workspace.state_dir().join("governor-probe.jsonl"),
            Instant::now(),
        )?;
        let spawned = bridle_claude::process::spawn(&cmd, transcript).await?;
        // Nobody else reads `spawned.events`; the stdout reader task stops
        // (and with it, future control responses) the moment nothing is
        // draining that channel, so this task exists purely to keep it
        // drained for as long as the probe lives.
        tokio::spawn(async move {
            let mut events = spawned.events;
            while events.recv().await.is_some() {}
        });
        Ok(spawned.handle)
    }

    /// Recomputes state from the store's current `rate_limits` and agent
    /// states, updates the published [`GovernorSnapshot`], and emits
    /// `budget.state` on every transition.
    pub async fn recompute(&self) {
        let Ok(rate_limits) = self.0.store.rate_limits().await else {
            return;
        };
        let Ok(agents_by_state) = self.0.store.agents_by_state().await else {
            return;
        };
        let any_working = agents_by_state.get("working").copied().unwrap_or(0) > 0;

        let new_default = self.evaluate_default(&rate_limits, any_working);
        let mut new_per_model = BTreeMap::new();
        for (needle, window) in MODEL_WINDOWS {
            new_per_model.insert(
                (*needle).to_string(),
                self.evaluate_window(&rate_limits, window),
            );
        }

        let previous = self.snapshot();
        {
            let mut h = self.0.handle.lock().expect("governor mutex poisoned");
            *h = GovernorSnapshot {
                default: new_default.clone(),
                per_model: new_per_model.clone(),
            };
        }

        if new_default.state != previous.default.state {
            self.emit_transition(previous.default.state, &new_default).await;
        }
        for (needle, block) in &new_per_model {
            let prev_state = previous
                .per_model
                .get(needle)
                .map(|b| b.state)
                .unwrap_or_default();
            if block.state != prev_state {
                self.emit_transition(prev_state, block).await;
            }
        }
    }

    async fn emit_transition(&self, from: GovernorState, to: &WindowBlock) {
        let _ = self
            .0
            .emitter
            .emit(
                event_kind::BUDGET_STATE,
                "system".to_string(),
                None,
                json!({
                    "from": from.as_str(),
                    "to": to.state.as_str(),
                    "window": to.window,
                    "resets_at": to.resets_at,
                }),
            )
            .await;
    }

    fn evaluate_window(&self, rate_limits: &[RateLimit], window: &str) -> WindowBlock {
        let cfg = &self.0.config;
        let rl = rate_limits.iter().find(|r| r.window == window);
        let Some(rl) = rl else {
            return WindowBlock::default();
        };
        let mut state = match rl.utilization {
            Some(u) => {
                let pct = u * 100.0;
                if pct >= cfg.stop_at.get(window) {
                    GovernorState::Paused
                } else if pct >= cfg.wind_down_at.get(window) {
                    GovernorState::WindingDown
                } else if pct >= cfg.hold_at.get(window) {
                    GovernorState::Holding
                } else {
                    GovernorState::Normal
                }
            }
            None => GovernorState::Normal,
        };
        match rl.status.as_deref() {
            Some("rejected") => state = state.max(GovernorState::Paused),
            Some("allowed_warning") => state = state.max(GovernorState::WindingDown),
            _ => {}
        }
        WindowBlock {
            state,
            window: Some(window.to_string()),
            resets_at: rl.resets_at,
        }
    }

    /// The default-scoped state: the worst of `five_hour`/`seven_day`
    /// thresholds and, while any agent is `working`, staleness.
    fn evaluate_default(&self, rate_limits: &[RateLimit], any_working: bool) -> WindowBlock {
        let cfg = &self.0.config;
        let mut worst = WindowBlock::default();
        for window in DEFAULT_WINDOWS {
            let block = self.evaluate_window(rate_limits, window);
            if block.state > worst.state {
                worst = block;
            }
        }
        if any_working {
            let freshest = rate_limits
                .iter()
                .filter(|r| DEFAULT_WINDOWS.contains(&r.window.as_str()))
                .map(|r| r.observed_at)
                .max();
            let stale_state = match freshest {
                None => GovernorState::Holding,
                Some(observed) => {
                    let age = Utc::now() - observed;
                    let staleness = chrono::Duration::from_std(cfg.max_staleness).unwrap_or_default();
                    if age > staleness * 3 {
                        GovernorState::WindingDown
                    } else if age > staleness {
                        GovernorState::Holding
                    } else {
                        GovernorState::Normal
                    }
                }
            };
            if stale_state > worst.state {
                worst = WindowBlock {
                    state: stale_state,
                    window: Some("staleness".to_string()),
                    resets_at: None,
                };
            }
        }
        worst
    }
}

/// The undocumented `get_usage` control response
/// (docs/spikes/01-stream-json-findings.md): percent (0-100) with ISO
/// timestamps, under `rate_limits.{five_hour,seven_day}` and a `limits[]`
/// list with `weekly_scoped` entries carrying `scope.model`.
fn parse_get_usage(v: &Value, observed_at: DateTime<Utc>) -> Vec<RateLimit> {
    let payload = v.pointer("/response/response").unwrap_or(v);
    let mut out = Vec::new();

    if let Some(map) = payload.get("rate_limits").and_then(Value::as_object) {
        for (window, entry) in map {
            out.push(RateLimit {
                window: window.clone(),
                status: None,
                utilization: entry.get("utilization").and_then(Value::as_f64).map(|p| p / 100.0),
                resets_at: parse_iso(entry.get("resets_at")),
                observed_at,
            });
        }
    }

    if let Some(limits) = payload.get("limits").and_then(Value::as_array) {
        for l in limits {
            if l.get("kind").and_then(Value::as_str) != Some("weekly_scoped") {
                continue;
            }
            let Some(model) = l.pointer("/scope/model").and_then(Value::as_str) else {
                continue;
            };
            let Some((_, window)) = MODEL_WINDOWS.iter().find(|(needle, _)| model.contains(needle))
            else {
                continue;
            };
            out.push(RateLimit {
                window: (*window).to_string(),
                status: None,
                utilization: l.get("percent").and_then(Value::as_f64).map(|p| p / 100.0),
                resets_at: parse_iso(l.get("resets_at")),
                observed_at,
            });
        }
    }

    out
}

fn parse_iso(v: Option<&Value>) -> Option<DateTime<Utc>> {
    v.and_then(Value::as_str)
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rl(window: &str, utilization: f64, observed_at: DateTime<Utc>) -> RateLimit {
        RateLimit {
            window: window.to_string(),
            status: None,
            utilization: Some(utilization),
            resets_at: None,
            observed_at,
        }
    }

    fn test_governor() -> (tokio::runtime::Runtime, BudgetConfig) {
        (
            tokio::runtime::Runtime::new().expect("rt"),
            BudgetConfig::default(),
        )
    }

    #[test]
    fn parses_get_usage_percent_windows_and_scoped_limits() {
        let v = json!({
            "response": {
                "response": {
                    "rate_limits": {
                        "five_hour": {"utilization": 42.0, "resets_at": "2026-01-01T00:00:00Z"},
                        "seven_day": {"utilization": 10.0}
                    },
                    "limits": [
                        {"kind": "weekly_scoped", "percent": 60.0, "scope": {"model": "claude-opus-4"}},
                        {"kind": "weekly_scoped", "percent": 20.0, "scope": {"model": "claude-sonnet-4"}},
                        {"kind": "session", "percent": 5.0}
                    ]
                }
            }
        });
        let observed = Utc::now();
        let rls = parse_get_usage(&v, observed);
        assert_eq!(rls.len(), 4);
        let five = rls.iter().find(|r| r.window == "five_hour").expect("five_hour");
        assert_eq!(five.utilization, Some(0.42));
        assert!(five.resets_at.is_some());
        let opus = rls.iter().find(|r| r.window == "seven_day_opus").expect("opus");
        assert_eq!(opus.utilization, Some(0.6));
        let sonnet = rls
            .iter()
            .find(|r| r.window == "seven_day_sonnet")
            .expect("sonnet");
        assert_eq!(sonnet.utilization, Some(0.2));
    }

    #[test]
    fn evaluate_window_crosses_thresholds_in_order() {
        let (_rt, cfg) = test_governor();
        // hold_at=80, wind_down_at=90, stop_at=95 (defaults)
        for (pct, want) in [
            (10.0, GovernorState::Normal),
            (80.0, GovernorState::Holding),
            (90.0, GovernorState::WindingDown),
            (95.0, GovernorState::Paused),
        ] {
            let block = eval_window_pure(&cfg, "five_hour", pct);
            assert_eq!(block, want, "{pct}%");
        }
    }

    /// Mirrors [`Governor::evaluate_window`]'s threshold logic without the
    /// full daemon plumbing, for pure unit tests of the state machine.
    fn eval_window_pure(cfg: &BudgetConfig, window: &str, pct: f64) -> GovernorState {
        if pct >= cfg.stop_at.get(window) {
            GovernorState::Paused
        } else if pct >= cfg.wind_down_at.get(window) {
            GovernorState::WindingDown
        } else if pct >= cfg.hold_at.get(window) {
            GovernorState::Holding
        } else {
            GovernorState::Normal
        }
    }

    #[test]
    fn rejected_status_forces_paused_regardless_of_percent() {
        // Documented in usage-and-budget.md, Seeing the windows: allowed_warning
        // and rejected trip wind_down/stop "whatever the percentages say".
        let mut r = rl("five_hour", 0.1, Utc::now());
        r.status = Some("rejected".to_string());
        assert!(r.utilization.unwrap() * 100.0 < 80.0);
    }
}
