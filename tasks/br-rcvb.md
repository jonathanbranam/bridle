+++
id = "br-rcvb"
title = "A daily 'what happened' report: in the mail digest, on request, and logged in docs/"
kind = "feature"
state = "planned"
created_at = "2026-10-08T23:01:17.444Z"
updated_at = "2026-10-08T23:08:00.014335Z"
created_by = "external:aide"
watchers = ["external:aide"]
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
