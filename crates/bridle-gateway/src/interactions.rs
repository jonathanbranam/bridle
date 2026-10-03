//! The wire types of the human-time API under `/api/v1/interactions/*`: how long the human spent
//! talking to agents, from each machine's prompt log. Types only for now; collection, interval
//! math and handlers follow (ticket u6w9, docs/design/human-web-ui.md "Human time"). Times are
//! RFC 3339 strings in UTC; the UI shows them in US Eastern. Minutes are fractional.

use serde::Serialize;
use ts_rs::TS;

/// What a report's totals are grouped by. `Agent` is the prompt's role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum InteractionGroup {
    Project,
    Agent,
    Machine,
}

/// A report's bucket size; days split at US Eastern midnight, weeks start Monday.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum InteractionBucket {
    Day,
    Week,
}

/// One interval of one session: from a prompt to the next one (capped at `gap`), or `tail` after
/// the last of a run. The raw form, so the UI can roll up its own way.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct SessionInterval {
    pub machine: String,
    pub project: String,
    /// The session's role (`orchestrator`, `advisor`, ...).
    pub agent: String,
    pub session: String,
    pub start: String,
    pub end: String,
}

/// `GET /interactions/intervals?from&to`
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct IntervalsReport {
    pub intervals: Vec<SessionInterval>,
    /// Machines that couldn't be polled; their data is missing until they're back.
    pub unreachable: Vec<String>,
}

/// One group's minutes in one bucket.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct GroupMinutes {
    /// The project, agent or machine name, per the report's `group`.
    pub key: String,
    /// The group's own sessions' intervals, unioned within the group. Groups can add up to more
    /// than `human_minutes`: two conversations at once count once there.
    pub minutes: f64,
}

/// One day or week of a report.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct ReportBucket {
    /// The bucket's first day, `YYYY-MM-DD` in US Eastern.
    pub start: String,
    /// The union of every session's intervals in the bucket.
    pub human_minutes: f64,
    pub groups: Vec<GroupMinutes>,
}

/// `GET /interactions/report?from&to&group&bucket`
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct InteractionReport {
    pub group: InteractionGroup,
    pub bucket: InteractionBucket,
    pub buckets: Vec<ReportBucket>,
    /// Over the whole range.
    pub human_minutes: f64,
    pub groups: Vec<GroupMinutes>,
    pub unreachable: Vec<String>,
}

/// One session's intervals in a day, for a timeline row.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct SessionTimeline {
    pub machine: String,
    pub project: String,
    pub agent: String,
    pub session: String,
    pub intervals: Vec<TimeSpan>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct TimeSpan {
    pub start: String,
    pub end: String,
}

/// `GET /interactions/day?date=`
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct DayReport {
    /// `YYYY-MM-DD` in US Eastern.
    pub date: String,
    /// First and last moment of human time that day; absent on a day with none.
    pub first_start: Option<String>,
    pub last_end: Option<String>,
    pub human_minutes: f64,
    pub sessions: Vec<SessionTimeline>,
    /// Moments where two or more sessions overlap, to shade on the timeline.
    pub overlaps: Vec<TimeSpan>,
    /// The most sessions covering one moment.
    pub peak_concurrency: u32,
    pub minutes_at_1: f64,
    pub minutes_at_2: f64,
    pub minutes_at_3_plus: f64,
    pub unreachable: Vec<String>,
}

/// `GET /interactions/hours?from&to&days=`
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
pub struct HoursReport {
    /// How many days in the range matched `days`.
    pub day_count: u32,
    /// 24 entries, hour 0 (US Eastern) first: human minutes in that hour, averaged over the
    /// matching days.
    pub minutes_per_hour: Vec<f64>,
    pub unreachable: Vec<String>,
}
