+++
id = "br-rcvb"
title = "A daily 'what happened' report: in the mail digest, on request, and logged in docs/"
kind = "feature"
state = "integrated"
created_at = "2026-10-08T23:01:17.444Z"
updated_at = "2026-10-09T02:30:03.970417Z"
created_by = "external:aide"
watchers = ["external:aide"]
branch = "bridle/dailyreport"
commit = "6775dbf44bfe75efcfcfb235fa0e3ee53d87788b"
summary = """Added `bridle report [--since 24h] [--write]` (crates/bridle/src/commands/report.rs): a mechanical Markdown report from tasks, edges and `git log main`, five sections, empty ones print "none", times in US Eastern. `--write` saves docs/reports/YYYY-MM-DD.md. No wire change. Caveats: tasks have no integrated-at field, so the time is the daemon's "integrated:" thread note (fallback updated_at); daemon restarts/upgrades skipped (not in the API). Decisions in docs/design/report.md; cli.md, README index, CHANGELOG updated; report registered in project.rs SCOPES and project_resolution_test CLASSES."""
ticket = "rcvb"
+++

Ticket: docs/tickets/open/a-daily-what-happened-report-in-the-mail-digest-on-request-a-rcvb.md (read it: the human's verbatim ask and the five sections). This is SLICE 1 of 2: the report itself, on request, and the log in the repo. Slice 2 (adding it to the mail digest, docs/design/mail.md, crates/bridle-mail) is a separate task, filed after the human has seen this slice's output.

Decisions made by pm-1 for the open design questions (record them in the new doc below):
- Mechanical, no LLM: generated from tasks, git and messages, so it costs no tokens and is the same every time.
- Scope: the project the command runs in (`--project`), one report per project. A machine-wide roll-up is a later step (YAGNI).
- Notable = every item below; no ranking.
- Window: the previous 24 hours ending now (`--since <duration>` to change, e.g. 48h).

Build: `bridle report [--since 24h] [--write]` (crates/bridle CLI; the daemon API already gives task lists with timestamps and states; use existing bridle-api client calls, add a daemon endpoint only if a needed field is missing and say so on the thread). Output is Markdown with these sections, in this order:
1. Features built and delivered: tasks of kind feature that reached `integrated` in the window (id, title, commit).
2. Bugs: bug tasks created in the window (identified) and bug tasks integrated in the window (fixed and delivered), listed separately.
3. Pending or blocked: tasks in states planned/claimed/pending/blocked right now that are in the queue tiers or blocked by an edge; one line each with why (blocked-by id).
4. Incidents: tasks of kind incident created or integrated in the window: id, title, state, and the last thread note as the fix/impact line.
5. Anything else: counts of tasks created, tasks integrated of other kinds (chore, question), commits on main in the window (`git log --since`, count and the first 20 subjects), daemon restarts/upgrades if the API exposes them (skip if not).
Times in the report are US Eastern, written bare ("7:00 AM"), per the human-timezone rule; ASCII only (rule ascii-in-editable-text).
`--write` saves it as docs/reports/YYYY-MM-DD.md (Eastern date, overwrites the same day's file, creates the folder) and prints the path; it does not commit. Without --write it prints to stdout. Handy for the aide to answer "what happened" in chat.

Files: crates/bridle/src/ (new commands/report.rs and the clap entry in cli.rs), docs/design/cli.md (the command), a new short docs/design/report.md (the decisions above, status line "built"), docs/README.md index line, CHANGELOG.md. No wire change unless needed.

Acceptance: just check passes; unit tests for the rendering from a fixed list of tasks/commits (each section, empty sections print "none", the time window edges); run it once for real and paste the output of `bridle report` on the task thread.

Model: Sonnet. Migration: none (new command; docs/reports/ is created on first --write). Out of scope: the mail digest (slice 2), a machine-wide roll-up, an agent-written narrative, a scheduler that writes the file daily (slice 2 can call --write from the digest pass).

## Thread

### note · agent:dailyreport · 2026-10-09T02:09:07.652Z
`bridle report` output (real run, 24h):

# What happened, Oct 7, 9:48 PM to Oct 8, 9:48 PM

## Features built and delivered

- br-7172: Gateway 9/10: multi-machine: per-machine human tokens, remote actions (3b3fe929f)
- br-fvkq: Mail between daemons, slice 3: outbox retry with backoff and the start-up ping (3haz P3) (09cb3e688)
- br-843g: Email goes to the project's aide, not the advisor (024232039)
- br-hc6j: Gateway: reply to any task (POST /api/v1/projects/{project}/tasks/{id}/reply) (ui-u2df B4) (d9fe8714f)
- br-5e4k: Daemon serves its repo's documents to the human token: /v1/documents, links/resolve, specs; shared code moved out of the gateway (ui-9hq8 B2) (78b87cd39)
- br-ty37: Gateway routes a remote project's documents through its daemon (ui-9hq8 B3; needs br-7172 and B2) (a91831467)

## Bugs

Identified:
- br-eyu3: Focus nudge is shared by every session on the machine: one session's prompt uses up the 5-minute nudge and the session the human is typing in stays silent (integrated)
- br-3p3h: Every daemon, test daemons included, forks 'ps -axo' every 2 s even with no agents: dozens at once during a test run drive load to 80-95 and hold all spawns (integrated)
- br-9z2n: Daemon reads the process table without forking ps (3p3h fix 3) (integrated)
- br-7fr6: Cap test threads in just test / just check so a full run does not drive load past the governor (integrated)
- br-7h8e: Flaky test: upgrade_test a_long_drain_wakes_the_orchestrator_once sees no wake (fewer than one) (integrated)
- br-ppa6: Gateway inherits BRIDLE_AS / BRIDLE_PROJECT from the shell that starts it: started from the orchestrator's session it calls every daemon as the orchestrator, and the web UI shows every project unreachable (integrated)
- br-6nzj: Test daemons stop polling at 200 ms; a resource-budget test; log the incident in docs/context/incidents.md (n4w4 recs 1, 2, 9) (pending)

Fixed and delivered:
- br-gdyy: bridle mail run prints nothing: no log output, so failed sends are silent (738b831e6)
- br-5p3z: Flaky/red: upgrade_test a_drain_holds_new_turns_and_delivers_them_after_the_restart fails in just check (6bb25ffdd)
- br-ghys: A project's aide messages an orchestrator that isn't watching that project's daemon; the roles don't say the orchestrator is per machine (40322d670)
- br-eyu3: Focus nudge is shared by every session on the machine: one session's prompt uses up the 5-minute nudge and the session the human is typing in stays silent (35a66d7a0)
- br-3p3h: Every daemon, test daemons included, forks 'ps -axo' every 2 s even with no agents: dozens at once during a test run drive load to 80-95 and hold all spawns (a1bde1054)
- br-9z2n: Daemon reads the process table without forking ps (3p3h fix 3) (018b9cfd5)
- br-7fr6: Cap test threads in just test / just check so a full run does not drive load past the governor (c3147bc77)
- br-7h8e: Flaky test: upgrade_test a_long_drain_wakes_the_orchestrator_once sees no wake (fewer than one) (08d5b21ae)
- br-ppa6: Gateway inherits BRIDLE_AS / BRIDLE_PROJECT from the shell that starts it: started from the orchestrator's session it calls every daemon as the orchestrator, and the web UI shows every project unreachable (25b5222ec)

## Pending or blocked

- br-a16f: Onboarding survey: file-db (pending, waiting)
- br-c265: Onboarding survey: track-web and harness (pi) (pending, waiting)
- br-5299: Onboarding survey: otters (otter-life and otters-back) (pending, waiting)
- br-3932: Run bridle on a project without a local bridle clone (mrhe) (pending, waiting)
- br-f070: dotfiles-local as a bridle project, directly on main (35mw) (pending, waiting)
- br-f8f9: The NUC recovers everything on boot (4r3k) (pending, waiting)
- br-10c0: Orchestrator identity and disaster recovery (7d62) (pending, waiting)
- br-96a6: Hold the orchestrator relaunch without restarting the daemon (8fsx) (pending, waiting)
- br-9966: bridle orchestrator hold / release: runtime switch for the relaunch (8fsx) (planned, waiting)
- br-01ff: Tickets through the bridle binary in every project: new, frontmatter, check, resolve; add closed: (7gk7) (pending, waiting)
- br-767c: Drop the Write(path) focus-file deny rules: Claude Code warns on every session start (pending, waiting)
- br-88d4: self_upgrade = "release": fetch, verify and swap the release binary (chvf 2) (planned, waiting)
- br-751e: Daemon keeps its own workflow checkout at the binary's tag (chvf 3) (blocked by br-88d4)
- br-6b8a: Orchestrator relaunch liveness: one clock, never a second orchestrator (jf9u) (blocked by br-a3b9)
- br-8c25: bridle token pair <machine>: set up tokens between machines over SSH (sk7p) (planned, waiting)
- br-79c3: Reserve role names, by prefix, for agent names (2vja) (blocked by br-a3b9)
- br-91b3: Reviews enforced by bridle, signed on the task (v2va): umbrella, slices A-D (planned, waiting)
- br-6ba3: Reviews A: in_review state, ready-for-review, review requirements on tasks (v2va slice A) (planned, waiting)
- br-46fe: Reviews B: signed review records on the task (v2va slice B) (blocked by br-6ba3)
- br-cf00: Reviews C: the daemon spawns the required reviewers; comment-only talk; cost recorded (v2va slice C) (blocked by br-46fe)
- br-f610: Reviews D: the gate: no landing until every required review is signed (v2va slice D) (blocked by br-cf00)
- br-e7e2: Migration: backfill ticket kind and two-way task links in every bridle project (v3dk slice B) (planned, waiting)
- br-1ddd: One watcher for every project: 'bridle agent wake --all-projects' (planned, waiting)
- br-1665: A web UI for the human: my to-dos and decisions, to run through and check off (pending, waiting)
- br-a3b9: Sat 10-03: review and land the parked branches (br-6b8a, br-2718, br-2672, br-8b98, br-79c3) (claimed by human)
- br-86c6: The workflow doesn't reach agents: resolved rules, hooks and overrides stop at the CLI (pending, waiting)
- br-519b: Task watchers: a creator field, a watchers list, and wakes that say what changed and are never lost (pending, waiting)
- br-7575: Expand on 8r5x: keep your interactive session logs (which sessions, how long, where) (claimed by human)
- br-3a42: Expand on z485: make the orchestrator non-interactive (what it gives you, who you talk to instead) (claimed by human)
- br-da2b: Expand on 2tpm: evaluate Go instead of Rust (why, what would decide it) (claimed by human)
- br-9667: Expand on 67qw: improve the base system's architecture (which parts, what's wrong) (claimed by human)
- br-u6w9: Human interaction time: daemon serves the prompt log; gateway collects across machines and reports (u6w9) (pending, waiting)
- br-gtzx: Seats: every role is a named, tracked seat that outlives its sessions, with its own inbox, handover and retirement (open, waiting)
- br-kuvh: Remove 'bridle agent wake --all-projects' once 3haz lands and rolls out: warn first, then delete (blocked by br-1ddd)
- br-44ms: Start an unplanned focus period now, through an agent (for example quiet for sleep) (pending, waiting)
- br-tkph: Follow up on the workflow review: 6 decisions, 2 waiting on others (34bw, vp9e, sk52, 7r2c, xfb3, ma2x, ntca, 98xt) (claimed by human)
- br-twg8: Try document review (x8jt) on gtzx: install the UI, review add, start the gateway, comment (claimed by human)
- br-efs2: Review and comment on kuw2, the machine daemon design (docs/tickets/open/a-machine-daemon-...-kuw2.md), with document review (claimed by human)
- br-mvtz: Postmortem: sessions killed each other's wake waiters with pkill -f (h3ar) (pending, waiting)
- br-cr7t: Add a postmortem ticket kind: the full write-up after an incident (pending, waiting)
- br-ytqu: Postmortem: a haiku worker waited on a check it couldn't see, reported done early, and took another task after renewal (qdw8) (pending, waiting)
- br-v3b7: Postmortem: quiet hours didn't reach replies to background wakes (pending, waiting)
- br-9s8u: Background wakes carry the session's prompt context: quiet hours, and every hook that should apply to them (pending, waiting)
- br-pakx: Specs: a way to run executable specs in CI without a bridle checkout (pending, waiting)
- br-2d6x: Specs: vitest-bridle setup leaves step files untypechecked (pending, waiting)
- br-m5kf: Specs: say what happens to unit tests a scenario now covers (pending, waiting)
- br-awh4: Specs: step text can't quote code or markup (pending, waiting)
- br-dbvd: Specs: bridle spec id duplicates ledger entries for hand-written IDs (pending, waiting)
- br-ysmu: CI watch missed 13 red runs on main; first ci_failed wake came 40 minutes late (pending, waiting)
- br-s4ve: vitest-bridle: a step can't skip a scenario at runtime (pending, waiting)
- br-abnq: One command for any agent's status and context (pending, waiting)
- br-cbbn: Scheduled nightly restart of an interactive session at a clock time (e.g. 3 AM) (pending, waiting)
- br-ft3b: Per-role handover instructions in the workflow, with project overrides (pending, waiting)
- br-yfv5: One scheduler for timed actions: scheduled messages (hrcn), nightly session restarts (cbbn), maintenance windows for upgrades and reboots (3nyk) (pending, waiting)
- br-n7cg: Mail between daemons, slice 2: mail for a visitor is forwarded to its home daemon (3haz P2) (pending, waiting)
- br-cufw: Mail between daemons, slice 4: visible state: outbox status, message show (queued/arrived/delivered), the aide report, and who-can-I-message (3haz P6, P7, bp2v) (pending, waiting)
- br-yydm: bridle-ui: a usage page with week-to-week charts of the five-hour and seven-day limits (xxw9, UI side) (pending, waiting)
- br-avu7: vk3y slice 2: 'bridle task new' requires --ticket (no-ticket sentinel) and a rule for every task creator (pending, waiting)
- br-fne2: bridle session advisor and bridle advisor start: near-identical commands that do different things (pending, waiting)
- br-m9sd: Agents don't know how to use the bridle CLI correctly: help, skills, shorter primes or guardrails? (pending, waiting)
- br-ukpm: A designer role: a background agent that reads a problem ticket, analyses the system and writes design options into the ticket (planned, waiting)
- br-stx8: A task's state says what's really happening: held and built-awaiting-landing are states, not 'planned' with a note (planned, waiting)
- br-95mu: A change spec (proposal and design) reviewed for risk and impact before any worker builds: a real gate, not a convention (pending, waiting)
- br-btdn: Daemon restart and self-upgrade blocked forever: manager.spawning() stays true with no agent spawning ('still busy: a spawning agent') (pending, waiting)
- br-s5ah: A daemon without an aide leaves its orchestrator no route to the human (pending, waiting)
- br-gdf3: Peer-token setup guidance: a token per receiving project per sending machine, minted on the receiver (pending, waiting)
- br-5j35: bridle session restart run from inside the session stops it and never relaunches: the detached restarter dies with the session (pending, waiting)
- br-a9g2: agent spawn --allow-tool silently does nothing for a tool outside the role's --tools set (WebSearch on a worker) (pending, waiting)
- br-sdrw: Turn off Claude Code prompt suggestions in every bridle session (pending, waiting)
- br-cfb2: Review ticket v8uu: seeing what background agents do (findings + 5 possible features) (claimed by human)
- br-v7ug: Run bridle's heavy work on the Windows PC under WSL2 (pending, waiting)
- br-jgdb: Windows PC: follow the WSL2 setup guide (install WSL2 + Ubuntu, wslconfig, Tailscale, start-up task, red/green shortcuts) and note the CPU and RAM (v7ug) (claimed by human)
- br-h7mu: Pick a name for the Windows PC (docs/context/naming.md) (v7ug) (claimed by human)
- br-jxaf: Daemon re-checks Tailscale after start so a boot-time race doesn't leave it loopback-only (v7ug audit 1) (planned, waiting)
- br-2uje: bridle doctor: Linux/WSL checks and per-OS fix text (v7ug audit 3) (planned, waiting)
- br-srj5: Mail attachments: UTF-8 text saved garbled as Latin-1 (em dash becomes 'â€”') (planned, waiting)
- br-enx3: bridle link: the unified URL scheme (/p/{project}/tasks/{id}, ...), document and spec links; link rule with exact formats (planned, waiting)
- br-4ge4: Show the current budget and focus settings from the CLI, consistently (pending, waiting)
- br-9xze: Scheduled messages, first slice: an agent schedules a message to itself (one-time or recurring), bridle schedule add/list/rm (planned, waiting)
- br-g5y2: Scheduled messages slice 2: role priming: wait at the maximum timeout and schedule a message for timed wake-ups (hrcn) (blocked by br-9xze)
- br-rcvb: A daily 'what happened' report: in the mail digest, on request, and logged in docs/ (planned, waiting)
- br-pa8h: Document review: keep the review list in the database, and scan for unresolved comments so none are missed (pending, waiting)
- br-6h65: A product manager that relates every ticket to open and planned work: links, merges, and folds ideas into changes already planned (pending, waiting)
- br-ubjd: A CHANGELOG line for every landed task, written on the branch, and one section per kind under Unreleased (pending, waiting)
- br-crht: One cross-platform process-table read: sysinfo + getpgid on Linux too, drop the /proc parser (planned, waiting)
- br-v6kr: A system architect role, and measuring bridle's own resource cost against a baseline (planned, waiting)
- br-6nzj: Test daemons stop polling at 200 ms; a resource-budget test; log the incident in docs/context/incidents.md (n4w4 recs 1, 2, 9) (pending, waiting)
- br-g76s: Load-hold notes: one per machine, name bridle-owned top consumers, honest text, load.hold.started/ended events, escalate a long hold (n4w4 recs 4, 5) (pending, waiting)
- br-fzwa: Audit every periodic daemon loop for what it forks or reads per tick; list them with cost in daemon.md (n4w4 rec 3) (pending, waiting)
- br-ks55: Only one full test run at a time per machine: just check takes a machine-wide lock (n4w4 rec 6) (pending, waiting)
- br-yw8b: fake-claude spawns skip the pyenv shim: resolve the interpreter once (n4w4 rec 7) (pending, waiting)

## Incidents

- br-bbhn: Incident: a NUC aide's message to bridle sat undelivered in the outbox: one try timed out while dalek slept, and the outbox never retries (3haz) [dropped] dropped: Duplicate of planned work: the fix is br-fvkq (3haz slice 3: outbox retry with backoff, start-up ping), now extended with this incident's requirements: a failed or stuck entry messages its sender, agent:<name> works across daemons, a resume from sleep counts as a start-up, and the send reports the first try's result. Visible outbox state and the human report stay in br-cufw (slice 4). Cause recorded above and on br-fvkq's thread. The stuck NUC message (o-0032) was relayed by hand by aide.
- br-vn54: Incident: remote control dropped for the dalek aide session after a tmux detach and laptop sleep; it came back only on reattach and local input [pending] open 4h, never planned: back to pending. Ready it again once someone will plan it.
- br-n4w4: Postmortem: bridle's own 'ps' polling (every daemon, test daemons at 200 ms) drove dalek's load to 76-144 and the load hold blocked spawns for ~26 h; br-3p3h fixed only the idle case [open] split off br-yw8b: fake-claude spawns skip the pyenv shim: resolve the interpreter once (n4w4 rec 7)

## Anything else

- tasks created: 28
- other tasks integrated: 2
  - br-at2j: WSL2 host: write the step-by-step setup guide docs/context/windows-wsl2-host.md (v7ug) (chore)
  - br-xg47: Put the NUC projects' human tokens in dalek's credentials ([human.nuc]) so the web UI can reach NUC documents (ui-9hq8) (chore)
- commits on main: 56
  - review: human comments on docs/context/name-ideas.md
  - review: human comments on docs/context/name-ideas.md
  - incidents: the orchestrator's bridle waiter was gone for ~2 h during the upgrade
  - review: human comments on docs/context/name-ideas.md
  - review: human comments on docs/context/name-ideas.md
  - review: human comments on docs/context/name-ideas.md
  - review: human comments on docs/context/name-ideas.md
  - review: human comments on docs/context/name-ideas.md
  - review: human comments on docs/context/name-ideas.md
  - review: human comments on docs/context/name-ideas.md
  - br-ppa6: Gateway inherits BRIDLE_AS / BRIDLE_PROJECT from the shell that starts it: started from the orchestrator's session it calls every daemon as the orchestrator, and the web UI shows every project unreachable
  - Ticket v6kr: baseline first; always-on monitoring must be proven cheap (from the human)
  - Ticket v6kr: the human's clarification: counters go to a file, not a listener
  - Ticket v6kr: the human's direction on the baseline, counters and benchmark scenarios
  - Ticket v6kr: record its task
  - Ticket v6kr: a system architect role and a resource baseline (from the human)
  - br-9z2n: Daemon reads the process table without forking ps (3p3h fix 3)
  - Ticket n4w4: record its task
  - Ticket n4w4: postmortem of the ps-polling load holds (br-3p3h), at the human's request
  - Ticket ubjd: record its task
  - ... and 36 more

### note · agent:dailyreport · 2026-10-09T02:09:15.164Z
done: bridle report (slice 1) built, real output pasted on the thread; just check exit 0, 1354 tests passed, ran on eb2335dd; branch tip 4cdfc4e3 is that plus a docs-only merge of main (docs/context/name-ideas.md)

### note · agent:manager-2 · 2026-10-09T02:30:03.970Z
integrated: 6775dbf44bfe75efcfcfb235fa0e3ee53d87788b (branch bridle/dailyreport)
