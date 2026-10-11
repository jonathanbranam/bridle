//! Notices new comments in documents under review and wakes that document's agent
//! (x8jt slice 2). The format is in `workflow/base/roles/document-reviewer.md`.
//!
//! A document is "under review" when its repo-relative path is a line in
//! `.bridle/review-documents.txt` (`bridle review add`). A thread is *pending* when its last
//! reply is the human's. Pending text that has been still for the quiet period goes to the
//! document's agent as one batch; the agent's own replies end the pending state, so its edits
//! don't wake it again, and a restart doesn't resend answered threads.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bridle_api::types::{AgentState, MessageKind, SpawnRequest, When, Workdir};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc, Weekday};
use sha2::Digest;

use crate::config::ReviewConfig;
use crate::store::Store;
use crate::supervisor::{AgentManager, ToTarget, system_principal};

pub const REGISTRY: &str = ".bridle/review-documents.txt";
const ROLE: &str = "document-reviewer";

/// The registered paths (blank lines and `#` comments skipped).
pub fn read_registry(repo: &Path) -> Vec<String> {
    std::fs::read_to_string(repo.join(REGISTRY))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect()
}

/// Adds (`on`) or removes a path; true when the file changed.
pub fn set_registered(repo: &Path, path: &str, on: bool) -> std::io::Result<bool> {
    let mut paths = read_registry(repo);
    let had = paths.iter().any(|p| p == path);
    if had == on {
        return Ok(false);
    }
    if on {
        paths.push(path.to_string());
    } else {
        paths.retain(|p| p != path);
    }
    let file = repo.join(REGISTRY);
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(
        file,
        paths.iter().map(|p| format!("{p}\n")).collect::<String>(),
    )?;
    Ok(true)
}

/// Agent names are at most 40 characters (`worktree.rs`).
const NAME_MAX: usize = 40;
/// The ticket ID alphabet (`bridle ticket new`).
const ID_ALPHABET: &str = "abcdefghjkmnpqrstuvwxyz23456789";

/// The agent's name for a document. A ticket (stem ending `-<id>`) is `doc-<id>`: short and
/// stable. Any other file is `doc-<slug>-<hash>`, the slug cut to fit and the hash of the path
/// keeping two long names apart.
pub fn agent_name(path: &str) -> String {
    let stem = Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let slug: String = stem
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let slug = slug.trim_matches('-');
    if let Some((_, id)) = slug.rsplit_once('-')
        && id.len() == 4
        && id.chars().all(|c| ID_ALPHABET.contains(c))
    {
        return format!("doc-{id}");
    }
    let hash: String = sha2::Sha256::digest(path.as_bytes())[..3]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let room = NAME_MAX - "doc-".len() - "-".len() - hash.len();
    let cut = &slug[..slug.len().min(room)];
    format!("doc-{}-{hash}", cut.trim_end_matches('-'))
}

/// Every entry (the `[!comment]` header or a `> **who, when:**` reply) may end with one status
/// mark, `[pending|sent|read YYYY-MM-DD HH:MM EDT]`: plain ASCII in the document, the latest
/// status only (git keeps the history). The format is in `workflow/base/roles/document-reviewer.md`.
/// Nothing parses the times: state comes from which mark is present.
const STAMP_LEN: usize = "2026-10-04 21:14 EDT".len();
/// The old mark, `" · sent 2026-10-04 21:14"` (Eastern, no zone); read as `sent` and rewritten.
const LEGACY: &str = " \u{b7} sent ";
const LEGACY_STAMP_LEN: usize = "2026-10-04 21:14".len();

/// A line split into its text and its status (`pending`, `sent` or `read`), if it has a mark.
fn split_mark(line: &str) -> (&str, Option<&'static str>) {
    if let Some(i) = line.rfind(LEGACY)
        && line.len() - i - LEGACY.len() == LEGACY_STAMP_LEN
    {
        return (&line[..i], Some("sent"));
    }
    if let Some(inner) = line.strip_suffix(']')
        && let Some(i) = inner.rfind(" [")
    {
        let mark = &inner[i + 2..];
        for state in ["pending", "sent", "read"] {
            if let Some(stamp) = mark.strip_prefix(state).and_then(|r| r.strip_prefix(' '))
                && stamp.len() == STAMP_LEN
            {
                return (&inner[..i], Some(state));
            }
        }
    }
    (line, None)
}

/// The line with its mark replaced by `[state stamp]`.
fn with_mark(line: &str, state: &str, stamp: &str) -> String {
    format!("{} [{state} {stamp}]", split_mark(line).0)
}

/// One comment thread: the line numbers of its quoted block, and which one is its newest entry
/// (the `[!comment]` header or the last `> **author, time:**` reply).
struct Thread {
    lines: Vec<usize>,
    newest: usize,
    resolved: bool,
}

fn scan(all: &[&str]) -> Vec<Thread> {
    let mut threads: Vec<Thread> = Vec::new();
    for (i, line) in all.iter().enumerate() {
        if line.starts_with("> [!comment]") {
            threads.push(Thread {
                lines: vec![i],
                newest: i,
                resolved: false,
            });
        } else if line.starts_with('>')
            && let Some(t) = threads.last_mut()
        {
            t.lines.push(i);
            if line.starts_with("> **") {
                t.newest = i;
                t.resolved |= line.starts_with("> **resolved by ");
            }
        }
    }
    threads
}

/// The thread ID in a header line (`> [!comment] c3 human, ...`).
fn header_id(header: &str) -> Option<u32> {
    let first = header
        .strip_prefix("> [!comment]")?
        .split_whitespace()
        .next()?;
    first.strip_prefix('c')?.parse().ok()
}

impl Thread {
    /// The quoted block, without marks.
    fn block(&self, all: &[&str]) -> String {
        self.lines
            .iter()
            .map(|&i| split_mark(all[i]).0)
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn id(&self, all: &[&str]) -> Option<u32> {
        header_id(all[self.lines[0]])
    }
}

/// A block without its thread ID, so a batch compares the same before and after IDs are added.
fn unid(block: &str) -> String {
    let Some(rest) = block.strip_prefix("> [!comment] ") else {
        return block.to_string();
    };
    let (first, tail) = rest.split_once(' ').unwrap_or((rest, ""));
    if first
        .strip_prefix('c')
        .is_some_and(|n| n.parse::<u32>().is_ok())
    {
        format!("> [!comment] {tail}")
    } else {
        block.to_string()
    }
}

/// A batch (blocks joined by a blank line) with every thread ID removed.
fn unid_batch(batch: &str) -> String {
    batch
        .split("\n\n")
        .map(unid)
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Comment threads whose newest entry is the human's with no mark or `[pending]`, each as its
/// quoted block. Threads already marked sent or read are left out unless `resend`; resolved
/// threads never go.
pub fn pending_threads(text: &str, resend: bool) -> Vec<String> {
    let all: Vec<&str> = text.split('\n').collect();
    scan(&all)
        .iter()
        .filter(|t| !t.resolved && is_human(all[t.newest]))
        .filter(|t| resend || matches!(split_mark(all[t.newest]).1, None | Some("pending")))
        .map(|t| t.block(&all))
        .collect()
}

/// The text with `[state stamp]` on the newest entry of each thread whose block is in `sent`
/// (a mark already there is replaced).
fn mark_threads(text: &str, sent: &[String], state: &str, stamp: &str) -> String {
    let all: Vec<&str> = text.split('\n').collect();
    let mut out: Vec<String> = all.iter().map(|l| l.to_string()).collect();
    for t in scan(&all) {
        if is_human(all[t.newest]) && sent.contains(&t.block(&all)) {
            out[t.newest] = with_mark(all[t.newest], state, stamp);
        }
    }
    out.join("\n")
}

/// Gives each open thread without an ID the next free one (highest in the file plus one).
fn assign_ids(text: &str) -> String {
    let all: Vec<&str> = text.split('\n').collect();
    let threads = scan(&all);
    let first = threads.iter().filter_map(|t| t.id(&all)).max().unwrap_or(0) + 1;
    let mut out: Vec<String> = all.iter().map(|l| l.to_string()).collect();
    let open = threads
        .iter()
        .filter(|t| !t.resolved && t.id(&all).is_none());
    for (next, t) in (first..).zip(open) {
        let h = t.lines[0];
        out[h] = out[h].replacen("> [!comment] ", &format!("> [!comment] c{next} "), 1);
    }
    out.join("\n")
}

/// The agent has read what was sent: every human entry marked `sent` becomes `read`. Old
/// marks are rewritten to the new form on the way. None when nothing changed.
fn mark_read(text: &str, stamp: &str) -> Option<String> {
    let all: Vec<&str> = text.split('\n').collect();
    let mut out: Vec<String> = all.iter().map(|l| l.to_string()).collect();
    for t in scan(&all) {
        for &i in t
            .lines
            .iter()
            .filter(|&&i| all[i].starts_with("> [!comment]") || all[i].starts_with("> **"))
        {
            if is_human(all[i]) && split_mark(all[i]).1 == Some("sent") {
                out[i] = with_mark(all[i], "read", stamp);
            }
        }
    }
    let out = out.join("\n");
    (out != text).then_some(out)
}

/// Old `sent` marks rewritten to the ASCII form, in the zone they were written in. None when
/// there are none.
fn migrate_marks(text: &str) -> Option<String> {
    if !text.contains(LEGACY) {
        return None;
    }
    let out: Vec<String> = text
        .split('\n')
        .map(|l| match l.rfind(LEGACY) {
            Some(i) if l.len() - i - LEGACY.len() == LEGACY_STAMP_LEN => {
                let at = &l[i + LEGACY.len()..];
                let zone = chrono::NaiveDateTime::parse_from_str(at, "%Y-%m-%d %H:%M")
                    .map(|d| zone_of(d.and_utc() + Duration::hours(5)))
                    .unwrap_or("EST");
                format!("{} [sent {at} {zone}]", &l[..i])
            }
            _ => l.to_string(),
        })
        .collect();
    Some(out.join("\n"))
}

/// Appends `**resolved by <by>, <stamp>**` to thread `id` (`c3`). Errors when there is no such
/// thread or it is already resolved.
pub fn resolve_thread(text: &str, id: &str, by: &str, stamp: &str) -> Result<String, String> {
    let n: u32 = id
        .strip_prefix('c')
        .and_then(|n| n.parse().ok())
        .ok_or_else(|| format!("{id} is not a thread ID (c1, c2, ...)"))?;
    let all: Vec<&str> = text.split('\n').collect();
    let t = scan(&all)
        .into_iter()
        .find(|t| t.id(&all) == Some(n))
        .ok_or_else(|| format!("no thread {id}"))?;
    if t.resolved {
        return Err(format!("{id} is already resolved"));
    }
    let mut out: Vec<String> = all.iter().map(|l| l.to_string()).collect();
    let last = *t.lines.last().expect("a thread has its header");
    out.insert(last + 1, format!("> **resolved by {by}, {stamp}**"));
    out.insert(last + 1, ">".to_string());
    Ok(out.join("\n"))
}

/// The human's entries are written by `human` or `human via <agent>`; anything else is an agent.
fn is_human(entry: &str) -> bool {
    let who = if let Some(h) = entry.strip_prefix("> [!comment]") {
        let h = h.trim();
        let h = match h.split_once(' ') {
            Some((first, rest)) if header_id(entry).is_some() && first.starts_with('c') => rest,
            _ => h,
        };
        h.split(',').next()
    } else if let Some(r) = entry.strip_prefix("> **") {
        r.split(',').next()
    } else {
        None
    };
    let who = who.unwrap_or("").trim().to_lowercase();
    who == "human" || who.starts_with("human via ")
}

/// US Eastern's zone abbreviation at `at`: EDT in daylight time (the US rule since 2007), else EST.
fn zone_of(at: DateTime<Utc>) -> &'static str {
    if eastern_offset(at) == Duration::hours(-4) {
        "EDT"
    } else {
        "EST"
    }
}

/// UTC offset of US Eastern at `at`: -4h in daylight time, else -5h.
fn eastern_offset(at: DateTime<Utc>) -> Duration {
    let year = at.year();
    let transition = |month, nth, hour| {
        NaiveDate::from_weekday_of_month_opt(year, month, Weekday::Sun, nth)
            .and_then(|d| d.and_hms_opt(hour, 0, 0))
            .map(|d| d.and_utc())
    };
    // 2:00 EST = 07:00 UTC in March; 2:00 EDT = 06:00 UTC in November.
    match (transition(3, 2, 7), transition(11, 1, 6)) {
        (Some(start), Some(end)) if at >= start && at < end => Duration::hours(-4),
        _ => Duration::hours(-5),
    }
}

/// `YYYY-MM-DD HH:MM EDT` in US Eastern, the human's time. Deliberately not UTC, unlike the
/// rest of what bridle records: the human reads and types these (ticket ehv6).
pub fn stamp(now: DateTime<Utc>) -> String {
    format!(
        "{} {}",
        (now + eastern_offset(now)).format("%Y-%m-%d %H:%M"),
        zone_of(now)
    )
}

#[derive(Default)]
struct Doc {
    /// Pending threads at the last look; `changed_at` is when this last differed.
    seen: String,
    changed_at: Option<DateTime<Utc>>,
    /// What the agent was last given.
    delivered: String,
}

/// A document's batch, ready to go.
#[derive(Debug, PartialEq)]
pub struct Due {
    pub path: String,
    pub batch: String,
}

/// The debounce state, with time passed in so tests need no clock.
#[derive(Default)]
pub struct Core {
    docs: HashMap<String, Doc>,
}

impl Core {
    pub fn observe(&mut self, path: &str, text: &str, now: DateTime<Utc>) {
        let seen = pending_threads(text, false).join("\n\n");
        let d = self.docs.entry(path.to_string()).or_default();
        if d.seen != seen {
            d.seen = seen;
            d.changed_at = Some(now);
        }
    }

    /// Documents whose pending threads have been still for `quiet`, oldest first.
    pub fn due(&self, now: DateTime<Utc>, quiet: Duration) -> Vec<Due> {
        let mut v: Vec<(DateTime<Utc>, Due)> = self
            .docs
            .iter()
            .filter(|(_, d)| !d.seen.is_empty() && d.seen != d.delivered)
            .filter_map(|(p, d)| {
                let at = d.changed_at?;
                (now - at >= quiet).then(|| {
                    (
                        at,
                        Due {
                            path: p.clone(),
                            batch: d.seen.clone(),
                        },
                    )
                })
            })
            .collect();
        v.sort_by_key(|(at, _)| *at);
        v.into_iter().map(|(_, d)| d).collect()
    }

    /// Why each observed document with something to send is not in `due` (debug log).
    fn log_held_back(&self, due: &[Due], now: DateTime<Utc>, quiet: Duration) {
        for (p, d) in &self.docs {
            if d.seen.is_empty() || d.seen == d.delivered || due.iter().any(|x| &x.path == p) {
                continue;
            }
            let still_for = d.changed_at.map(|at| now - at);
            tracing::debug!(path = %p, ?still_for, ?quiet, "pending comments held back: not still long enough");
        }
    }

    pub fn delivered(&mut self, path: &str) {
        if let Some(d) = self.docs.get_mut(path) {
            d.delivered = d.seen.clone();
        }
    }
}

fn due_paths_not_admitted(due: &[String], go: &[Due]) -> Vec<String> {
    due.iter()
        .filter(|p| !go.iter().any(|g| &g.path == *p))
        .cloned()
        .collect()
}

/// Which due documents go now: one whose agent is already running always does; one that needs
/// a start takes a slot, and waits (stays due) when the cap is full.
pub fn admit(
    due: Vec<Due>,
    is_running: impl Fn(&str) -> bool,
    mut running: usize,
    cap: usize,
) -> Vec<Due> {
    let mut out = Vec::new();
    for d in due {
        if is_running(&agent_name(&d.path)) {
            out.push(d);
        } else if running < cap {
            running += 1;
            out.push(d);
        }
    }
    out
}

/// An idle document agent past its expiry stops (resumed when its next batch comes).
pub fn expired(now: DateTime<Utc>, last_active: DateTime<Utc>, idle: Duration) -> bool {
    now - last_active >= idle
}

fn prompt(repo: &Path, path: &str, batch: &str) -> String {
    format!(
        "New comments to answer in {} (the document under review). Read it, answer each thread \
         below as your role says, and commit.\n\n{batch}",
        repo.join(path).display()
    )
}

/// How long a document stays still after bridle's last edit to it before the edits are committed,
/// so a burst (ids, then `sent`, then `read`) is one commit.
const COMMIT_QUIET: Duration = Duration::seconds(10);

/// Commits the edits bridle itself makes to documents under review (fd8e), through the commit
/// path the human's comments use. Only the owner clone commits, and only a file that was clean
/// before bridle's first edit: anything else is someone's work in progress and is left alone.
#[derive(Default)]
struct EditCommits {
    /// Files bridle has edited and not yet committed, with the time of its latest edit.
    ours: HashMap<String, DateTime<Utc>>,
    /// Files already reported as left alone, so the log says it once.
    warned: std::collections::HashSet<String>,
}

impl EditCommits {
    /// Call before bridle writes `path`: true when the write may be committed later.
    fn before_write(&mut self, repo: &Path, owner: bool, path: &str) -> bool {
        if !owner {
            return false;
        }
        if self.ours.contains_key(path) || bridle_docs::documents::is_clean(repo, path) {
            self.warned.remove(path);
            return true;
        }
        if self.warned.insert(path.to_string()) {
            tracing::warn!(
                path,
                "document has uncommitted edits by others; bridle's edit to it is left uncommitted"
            );
        }
        false
    }

    /// Call after a write that `before_write` allowed.
    fn wrote(&mut self, path: &str, now: DateTime<Utc>) {
        self.ours.insert(path.to_string(), now);
    }

    /// Commits each file that has been still for `COMMIT_QUIET`.
    fn flush(&mut self, repo: &Path, now: DateTime<Utc>) {
        let due: Vec<String> = self
            .ours
            .iter()
            .filter(|(_, at)| now - **at >= COMMIT_QUIET)
            .map(|(p, _)| p.clone())
            .collect();
        for p in due {
            self.ours.remove(&p);
            if bridle_docs::documents::is_clean(repo, &p) {
                continue;
            }
            let msg = format!("review: bridle updates comment marks on {p}");
            if let Err(e) = bridle_docs::documents::commit_file(repo, &p, &msg) {
                tracing::warn!(path = %p, error = %e, "could not commit bridle's edit to a document");
            }
        }
    }
}

#[derive(Clone)]
pub struct DocWatcher {
    store: Store,
    manager: AgentManager,
    repo: PathBuf,
    cfg: ReviewConfig,
    core: std::sync::Arc<std::sync::Mutex<Core>>,
    /// Held while a batch is read, delivered and marked, so a tick and a review-now never send
    /// the same thread twice.
    sending: std::sync::Arc<tokio::sync::Mutex<()>>,
    /// False on a clone that is not the project's owner: it edits, but never commits.
    owner: bool,
    edits: std::sync::Arc<std::sync::Mutex<EditCommits>>,
}

/// What `review now` did.
#[derive(Debug, PartialEq, Eq)]
pub struct Reviewed {
    pub agent: String,
    /// Threads sent; 0 when none were pending (or unsent).
    pub threads: usize,
}

impl DocWatcher {
    pub fn new(store: Store, manager: AgentManager, repo: PathBuf, cfg: ReviewConfig) -> Self {
        Self {
            store,
            manager,
            repo,
            cfg,
            core: Default::default(),
            sending: Default::default(),
            owner: true,
            edits: Default::default(),
        }
    }

    /// Whether this clone is the project's owner, the only one that commits (fd8e).
    pub fn with_owner(mut self, owner: bool) -> Self {
        self.owner = owner;
        self
    }

    /// Writes `text` to `path` (repo-relative), noting it for the debounced commit.
    fn write_marked(&self, path: &str, text: &str, now: DateTime<Utc>) -> std::io::Result<()> {
        let mut edits = self.edits.lock().expect("doc edits lock");
        let track = edits.before_write(&self.repo, self.owner, path);
        std::fs::write(self.repo.join(path), text)?;
        if track {
            edits.wrote(path, now);
        }
        Ok(())
    }

    pub async fn tick(&self) {
        self.tick_at(Utc::now()).await;
    }

    pub async fn tick_at(&self, now: DateTime<Utc>) {
        let Ok(agents) = self.store.list_agents(false).await else {
            return;
        };
        let docs: Vec<&_> = agents.iter().filter(|a| a.role == ROLE).collect();
        let idle = Duration::from_std(self.cfg.idle).unwrap_or(Duration::hours(4));
        for a in docs.iter().filter(|a| a.state == AgentState::Idle) {
            let last = a.last_event_at.unwrap_or(a.updated_at);
            if expired(now, last, idle) {
                let _ = self.manager.stop(&a.id, false, &system_principal()).await;
            }
        }

        self.sweep_marks(&docs, now).await;
        self.edits
            .lock()
            .expect("doc edits lock")
            .flush(&self.repo, now);

        let due = {
            let mut core = self.core.lock().expect("doc watch lock");
            for p in read_registry(&self.repo) {
                if let Ok(text) = std::fs::read_to_string(self.repo.join(&p)) {
                    core.observe(&p, &text, now);
                }
            }
            let quiet = Duration::from_std(self.cfg.quiet).unwrap_or(Duration::minutes(7));
            let due = core.due(now, quiet);
            core.log_held_back(&due, now, quiet);
            due
        };
        let running = docs.iter().filter(|a| a.state.is_running()).count();
        let go_paths: Vec<String> = due.iter().map(|d| d.path.clone()).collect();
        let go = admit(
            due,
            |n| docs.iter().any(|a| a.name == n && a.state.is_running()),
            running,
            self.cfg.max_agents as usize,
        );
        for d in &due_paths_not_admitted(&go_paths, &go) {
            tracing::debug!(path = %d, "document is due but waits for a free agent slot");
        }
        for d in go {
            tracing::debug!(path = %d.path, "document is due; sending");
            // `Ok(0)`: the text changed since it was observed; the next tick looks again.
            if let Ok(n) = self.send(&d.path, false, Some(&d.batch), now).await
                && n > 0
            {
                self.core.lock().expect("doc watch lock").delivered(&d.path);
            }
        }
    }

    /// Housekeeping on the documents under review: old marks are rewritten to the ASCII form,
    /// and a `[sent]` thread becomes `[read]` once its agent has no unread message (a message
    /// is read when the agent lists or wakes on it; a started agent was given the text as its
    /// prompt). A thread stuck at `sent` means the agent is busy, down or out of budget.
    async fn sweep_marks(&self, docs: &[&bridle_api::types::Agent], now: DateTime<Utc>) {
        let _guard = self.sending.lock().await;
        for p in read_registry(&self.repo) {
            let file = self.repo.join(&p);
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };
            let mut text = text;
            let mut changed = false;
            if let Some(m) = migrate_marks(&text) {
                text = m;
                changed = true;
            }
            if let Some(a) = docs.iter().find(|a| a.name == agent_name(&p))
                && matches!(self.store.unread_count(&a.id).await, Ok(0))
                && let Some(m) = mark_read(&text, &stamp(now))
            {
                text = m;
                changed = true;
            }
            if changed && let Err(e) = self.write_marked(&p, &text, now) {
                tracing::warn!(path = %p, error = %e, "could not update comment marks");
            }
        }
    }

    /// `bridle review add`. With `only_if_pending`, a document with no pending thread is left
    /// as it is. Adding one already under review changes nothing. Returns whether the document
    /// is under review afterwards.
    pub fn add(&self, path: &str, only_if_pending: bool) -> Result<bool, String> {
        if Path::new(path)
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err(format!("{path} is not a path inside the repo"));
        }
        let file = self.repo.join(path);
        let text = std::fs::read_to_string(&file)
            .map_err(|e| format!("{path} is not a readable file under the repo: {e}"))?;
        if only_if_pending && pending_threads(&text, false).is_empty() {
            return Ok(read_registry(&self.repo).iter().any(|p| p == path));
        }
        set_registered(&self.repo, path, true)
            .map_err(|e| format!("writing the review list: {e}"))?;
        Ok(true)
    }

    /// `bridle review now`: sends the document's pending threads at once, skipping the quiet
    /// period (and the agent cap: the human asked). Threads already marked sent stay out unless
    /// `resend`.
    pub async fn review_now(&self, path: &str, resend: bool) -> Result<Reviewed, String> {
        if !read_registry(&self.repo).iter().any(|p| p == path) {
            return Err(format!(
                "{path} is not under review (bridle review add {path})"
            ));
        }
        let threads = self.send(path, resend, None, Utc::now()).await?;
        Ok(Reviewed {
            agent: agent_name(path),
            threads,
        })
    }

    /// Reads the document, delivers its pending threads (all of them, with `resend`) and marks
    /// them sent. With `expect`, sends only if the pending text is still exactly that. Returns
    /// the number of threads delivered.
    async fn send(
        &self,
        path: &str,
        resend: bool,
        expect: Option<&str>,
        now: DateTime<Utc>,
    ) -> Result<usize, String> {
        let _guard = self.sending.lock().await;
        let file = self.repo.join(path);
        let text = std::fs::read_to_string(&file).map_err(|e| format!("{path}: {e}"))?;
        let threads = pending_threads(&text, resend);
        if threads.is_empty() {
            tracing::debug!(path, "document has nothing pending to send");
            return Ok(0);
        }
        // `expect` was read before IDs were assigned; compare without them.
        if expect.is_some_and(|e| unid_batch(e) != unid_batch(&threads.join("\n\n"))) {
            tracing::debug!(
                path,
                "document changed since it was observed; looking again next tick"
            );
            return Ok(0);
        }
        // The agent sees the thread IDs, so they go in before the batch does.
        let with_ids = assign_ids(&text);
        if with_ids != text {
            self.write_marked(path, &with_ids, now)
                .map_err(|e| format!("writing {path}: {e}"))?;
        }
        let threads = pending_threads(&with_ids, resend);
        let batch = threads.join("\n\n");
        if !self
            .deliver(&Due {
                path: path.to_string(),
                batch,
            })
            .await
        {
            return Err("the document's agent could not be started; see the daemon log".into());
        }
        // Read again: the agent may already have edited the file.
        let latest = std::fs::read_to_string(&file).map_err(|e| format!("{path}: {e}"))?;
        let marked = mark_threads(&latest, &threads, "sent", &stamp(now));
        self.write_marked(path, &marked, now)
            .map_err(|e| format!("marking {path}: {e}"))?;
        Ok(threads.len())
    }

    /// Start, resume or message the document's agent. False when it didn't go.
    async fn deliver(&self, d: &Due) -> bool {
        let name = agent_name(&d.path);
        let text = prompt(&self.repo, &d.path, &d.batch);
        let p = system_principal();
        let agent = self.store.get_agent(&name).await.ok().flatten();
        let r = match agent {
            None => self
                .manager
                .spawn(
                    SpawnRequest {
                        role: ROLE.to_string(),
                        name: Some(name),
                        prompt: Some(text),
                        workdir: Some(Workdir::Repo),
                        ..Default::default()
                    },
                    &p,
                )
                .await
                .map(|_| ()),
            Some(a) => {
                let resumed = if a.state.is_running() {
                    Ok(())
                } else {
                    self.manager.resume(&a.id, false, &p).await.map(|_| ())
                };
                match resumed {
                    Ok(()) => self
                        .manager
                        .send(
                            "system".to_string(),
                            ToTarget::Agent(a.id),
                            MessageKind::Note,
                            text,
                            When::Idle,
                            None,
                        )
                        .await
                        .map(|_| ()),
                    Err(e) => Err(e),
                }
            }
        };
        if let Err(e) = &r {
            tracing::warn!(path = %d.path, error = %e, "document agent not started; will retry");
        }
        r.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HUMAN: &str = "Line.\n\n> [!comment] human, 2026-10-03 14:05, on \"Line\"\n> Why?\n";
    const ANSWERED: &str = "Line.\n\n> [!comment] human, 2026-10-03 14:05, on \"Line\"\n> Why?\n>\n> **doc-3haz, 14:06:** @human Because.\n";

    fn t(min: i64) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(1_800_000_000, 0).unwrap() + Duration::minutes(min)
    }

    #[test]
    fn answered_threads_are_not_pending() {
        assert_eq!(pending_threads(HUMAN, false).len(), 1);
        assert!(pending_threads(ANSWERED, false).is_empty());
        let again = format!("{ANSWERED}>\n> **human, 14:20:** and?\n");
        assert_eq!(pending_threads(&again, false).len(), 1);
    }

    #[test]
    fn debounce_restarts_on_each_change() {
        let q = Duration::minutes(7);
        let mut c = Core::default();
        c.observe("a.md", HUMAN, t(0));
        assert!(c.due(t(6), q).is_empty());
        let more = format!("{HUMAN}>\n> **human, 14:09:** more\n");
        c.observe("a.md", &more, t(5));
        assert!(c.due(t(11), q).is_empty());
        let due = c.due(t(12), q);
        assert_eq!(due.len(), 1);
        assert!(due[0].batch.contains("more"));
    }

    #[test]
    fn batch_has_every_pending_thread_and_sends_once() {
        let two = format!("{HUMAN}\nOther.\n\n> [!comment] human, 14:07, on \"Other\"\n> Hm?\n");
        let mut c = Core::default();
        c.observe("a.md", &two, t(0));
        let due = c.due(t(7), Duration::minutes(7));
        assert_eq!(due[0].batch.matches("[!comment]").count(), 2);
        c.delivered("a.md");
        assert!(c.due(t(30), Duration::minutes(7)).is_empty());
        // The agent answers; no new batch, and a later human reply is one.
        c.observe("a.md", ANSWERED, t(31));
        assert!(c.due(t(60), Duration::minutes(7)).is_empty());
    }

    #[test]
    fn cap_holds_back_starts_not_running_agents() {
        let due = |p: &str| Due {
            path: p.to_string(),
            batch: "x".into(),
        };
        let go = admit(
            vec![due("a.md"), due("b.md"), due("c.md")],
            |n| n == agent_name("c.md"),
            2,
            3,
        );
        let paths: Vec<_> = go.iter().map(|d| d.path.as_str()).collect();
        assert_eq!(paths, ["a.md", "c.md"]);
    }

    #[test]
    fn idle_agents_expire() {
        let idle = Duration::hours(4);
        assert!(!expired(t(239), t(0), idle));
        assert!(expired(t(240), t(0), idle));
    }

    #[test]
    fn names_and_registry() {
        assert_eq!(
            agent_name("docs/tickets/open/My Ticket_x8jt.md"),
            "doc-x8jt"
        );
        let real = agent_name(
            "docs/tickets/open/daemons-deliver-mail-to-each-other-across-machines-store-and-3haz.md",
        );
        assert_eq!(real, "doc-3haz");
        assert!(crate::worktree::validate_agent_name(&real).is_ok());
        let (a, b) = (
            agent_name("docs/a-very-long-document-name-that-keeps-going-and-going-one.md"),
            agent_name("docs/a-very-long-document-name-that-keeps-going-and-going-two.md"),
        );
        assert!(a.len() <= 40 && b.len() <= 40 && a != b);
        assert!(crate::worktree::validate_agent_name(&a).is_ok());
        let short = agent_name("notes/Plan.md");
        assert!(
            short.starts_with("doc-plan-") && crate::worktree::validate_agent_name(&short).is_ok()
        );
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".bridle")).unwrap();
        std::fs::write(dir.path().join(REGISTRY), "# x\na.md\n\n b.md \n").unwrap();
        assert_eq!(read_registry(dir.path()), ["a.md", "b.md"]);
        assert!(set_registered(dir.path(), "c.md", true).unwrap());
        assert!(!set_registered(dir.path(), "c.md", true).unwrap());
        assert!(set_registered(dir.path(), "a.md", false).unwrap());
        assert_eq!(read_registry(dir.path()), ["b.md", "c.md"]);
    }

    #[test]
    fn sent_mark_goes_on_the_newest_human_entry_once() {
        let text =
            format!("{HUMAN}>\n> **docs agent, 14:06:** Because.\n>\n> **human, 14:20:** and?\n");
        let sent = pending_threads(&text, false);
        assert_eq!(sent.len(), 1);
        let marked = mark_threads(&text, &sent, "sent", "2026-10-04 21:14 EDT");
        assert!(marked.contains("> **human, 14:20:** and? [sent 2026-10-04 21:14 EDT]\n"));
        assert_eq!(marked.matches("[sent ").count(), 1);
        // Marked threads are left out, are still the human's (so the watcher's pending state
        // is unchanged by the mark), and come back with resend, without the old mark.
        assert!(pending_threads(&marked, false).is_empty());
        let again = pending_threads(&marked, true);
        assert_eq!(again, sent);
        let remarked = mark_threads(&marked, &again, "sent", "2026-10-05 08:00 EDT");
        assert!(remarked.contains("and? [sent 2026-10-05 08:00 EDT]\n"));
        assert_eq!(remarked.matches("[sent ").count(), 1);
        // A new human reply is unsent again.
        let reply = format!("{marked}>\n> **human, 14:30:** hello?\n");
        assert_eq!(pending_threads(&reply, false).len(), 1);
        // A mark on a header works too.
        let m = mark_threads(
            HUMAN,
            &pending_threads(HUMAN, false),
            "sent",
            "2026-10-04 21:14 EDT",
        );
        assert!(m.contains("on \"Line\" [sent 2026-10-04 21:14 EDT]\n"));
        assert!(pending_threads(&m, false).is_empty());
    }

    #[test]
    fn pending_marks_count_as_pending_and_read_as_done() {
        let pending = HUMAN.replace("\"Line\"\n", "\"Line\" [pending 2026-10-04 10:57 EDT]\n");
        assert_eq!(pending_threads(&pending, false).len(), 1);
        let read = pending.replace("[pending", "[read");
        assert!(pending_threads(&read, false).is_empty());
        assert_eq!(pending_threads(&read, true).len(), 1);
    }

    #[test]
    fn only_human_and_human_via_are_the_human() {
        let via = HUMAN.replace("human, 2026", "human via advisor, 2026");
        assert_eq!(pending_threads(&via, false).len(), 1);
        let advisor = HUMAN.replace("human, 2026", "advisor, 2026");
        assert!(pending_threads(&advisor, false).is_empty());
        let with_id = HUMAN.replace("] human", "] c3 human");
        assert_eq!(pending_threads(&with_id, false).len(), 1);
    }

    #[test]
    fn ids_are_next_highest_and_batch_compares_without_them() {
        let two = format!("{HUMAN}\nOther.\n\n> [!comment] c7 human, 14:07, on \"Other\"\n> Hm?\n");
        let with = assign_ids(&two);
        assert!(with.contains("> [!comment] c8 human, 2026-10-03"));
        assert!(with.contains("> [!comment] c7 human, 14:07"));
        assert_eq!(assign_ids(&with), with);
        assert_eq!(
            pending_threads(&two, false)
                .iter()
                .map(|b| unid(b))
                .collect::<Vec<_>>(),
            pending_threads(&with, false)
                .iter()
                .map(|b| unid(b))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn read_follows_sent_and_old_marks_migrate() {
        let sent = mark_threads(
            HUMAN,
            &pending_threads(HUMAN, false),
            "sent",
            "2026-10-04 21:14 EDT",
        );
        let read = mark_read(&sent, "2026-10-04 21:15 EDT").unwrap();
        assert!(read.contains("[read 2026-10-04 21:15 EDT]") && !read.contains("[sent"));
        assert!(mark_read(&read, "x").is_none());
        let old = HUMAN.replace("\"Line\"\n", "\"Line\" \u{b7} sent 2026-10-04 21:14\n");
        assert!(pending_threads(&old, false).is_empty());
        let migrated = migrate_marks(&old).unwrap();
        assert!(migrated.contains("on \"Line\" [sent 2026-10-04 21:14 EDT]\n"));
        assert!(migrated.is_ascii());
        assert!(migrate_marks(&migrated).is_none());
    }

    #[test]
    fn resolve_appends_the_closing_line_and_ends_pending() {
        let doc = format!(
            "{}\nAfter.\n",
            HUMAN.replace("] human", "] c3 human").trim_end()
        );
        let r = resolve_thread(&doc, "c3", "human", "2026-10-04 11:17 EDT").unwrap();
        assert!(r.contains("> Why?\n>\n> **resolved by human, 2026-10-04 11:17 EDT**\nAfter."));
        assert!(pending_threads(&r, true).is_empty());
        assert!(
            resolve_thread(&r, "c3", "human", "x")
                .unwrap_err()
                .contains("already")
        );
        assert!(
            resolve_thread(&doc, "c9", "human", "x")
                .unwrap_err()
                .contains("no thread")
        );
    }

    #[test]
    fn batch_with_ids_matches_the_same_batch_without() {
        let two = format!("{HUMAN}\nOther.\n\n> [!comment] c7 human, 14:07, on \"Other\"\n> Hm?\n");
        let with = assign_ids(&two);
        let join = |t: &str| pending_threads(t, false).join("\n\n");
        assert_eq!(unid_batch(&join(&two)), unid_batch(&join(&with)));
    }

    #[test]
    fn mark_does_not_retrigger_the_watcher() {
        let mut c = Core::default();
        c.observe("a.md", HUMAN, t(0));
        assert_eq!(c.due(t(7), Duration::minutes(7)).len(), 1);
        c.delivered("a.md");
        let marked = mark_threads(
            HUMAN,
            &pending_threads(HUMAN, false),
            "sent",
            "2026-10-04 21:14 EDT",
        );
        c.observe("a.md", &marked, t(8));
        assert!(c.due(t(60), Duration::minutes(7)).is_empty());
    }

    #[test]
    fn stamp_is_eastern() {
        // 2026-10-04 01:14 UTC is 21:14 the evening before in EDT; January is EST.
        let edt = DateTime::parse_from_rfc3339("2026-10-05T01:14:00Z")
            .unwrap()
            .to_utc();
        assert_eq!(stamp(edt), "2026-10-04 21:14 EDT");
        let est = DateTime::parse_from_rfc3339("2026-01-05T01:14:00Z")
            .unwrap()
            .to_utc();
        assert_eq!(stamp(est), "2026-01-04 20:14 EST");
    }

    fn git(dir: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .expect("git");
        assert!(out.status.success(), "{args:?}: {out:?}");
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn commits(dir: &Path) -> usize {
        git(dir, &["rev-list", "--count", "HEAD"]).parse().unwrap()
    }

    fn edit_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init", "-q", "-b", "main"]);
        git(dir.path(), &["config", "user.name", "t"]);
        git(dir.path(), &["config", "user.email", "t@t"]);
        git(dir.path(), &["config", "commit.gpgsign", "false"]);
        std::fs::write(dir.path().join("d.md"), "a [sent x]\n").unwrap();
        git(dir.path(), &["add", "."]);
        git(dir.path(), &["commit", "-q", "-m", "init"]);
        dir
    }

    /// One tracked write, as `DocWatcher::write_marked` does it.
    fn bridle_writes(e: &mut EditCommits, dir: &Path, owner: bool, text: &str, at: DateTime<Utc>) {
        let track = e.before_write(dir, owner, "d.md");
        std::fs::write(dir.join("d.md"), text).unwrap();
        if track {
            e.wrote("d.md", at);
        }
    }

    #[test]
    fn marker_rewrite_is_committed_once_after_a_burst() {
        let d = edit_repo();
        let mut e = EditCommits::default();
        bridle_writes(&mut e, d.path(), true, "a [read x]\n", t(0));
        bridle_writes(
            &mut e,
            d.path(),
            true,
            "a [read y]\n",
            t(0) + Duration::seconds(3),
        );
        e.flush(d.path(), t(0) + Duration::seconds(5));
        assert_eq!(commits(d.path()), 1, "still inside the quiet period");
        e.flush(d.path(), t(0) + Duration::seconds(30));
        assert_eq!(commits(d.path()), 2);
        assert!(git(d.path(), &["log", "-1", "--format=%s"]).starts_with("review: "));
        assert!(git(d.path(), &["status", "--porcelain"]).is_empty());
        e.flush(d.path(), t(0) + Duration::seconds(60));
        assert_eq!(commits(d.path()), 2);
    }

    #[test]
    fn a_file_with_others_uncommitted_edits_is_left_alone() {
        let d = edit_repo();
        std::fs::write(d.path().join("d.md"), "someone's draft\n").unwrap();
        let mut e = EditCommits::default();
        bridle_writes(&mut e, d.path(), true, "someone's draft [read x]\n", t(0));
        e.flush(d.path(), t(0) + Duration::seconds(60));
        assert_eq!(commits(d.path()), 1);
        assert!(!git(d.path(), &["status", "--porcelain"]).is_empty());
    }

    #[test]
    fn a_non_owner_clone_makes_no_commit() {
        let d = edit_repo();
        let mut e = EditCommits::default();
        bridle_writes(&mut e, d.path(), false, "a [read x]\n", t(0));
        e.flush(d.path(), t(0) + Duration::seconds(60));
        assert_eq!(commits(d.path()), 1);
    }
}
