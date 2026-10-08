+++
id = "br-gdyy"
title = "bridle mail run prints nothing: no log output, so failed sends are silent"
kind = "bug"
state = "planned"
created_at = "2026-10-08T00:41:39.868Z"
updated_at = "2026-10-08T00:42:26.479498Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "gdyy"
+++

Ticket: docs/tickets/open/bridle-mail-run-prints-nothing-no-log-output-so-failed-sends-gdyy.md (read it: the first section is the bug, the second is a design question).
Goal (this task): `bridle mail run` logs like `serve` and `gateway`, so a refused SES send, a failed poll, a dropped mail and a relayed reply show in its terminal (or its log file when run as a service).
Build: in `mail_run` (crates/bridle/src/commands/orchestrator.rs, ~line 262) install the same tracing subscriber setup `serve` (crates/bridle/src/serve.rs ~182) and `gateway` (crates/bridle/src/gateway.rs ~50) use: stderr, RUST_LOG-style filter, info by default. Reuse their helper if one exists; do not write a third copy of the setup (extract a shared function if needed). Confirm the existing `tracing::warn!/info!` lines in crates/bridle-mail (`mail outbound failed`, `mail poll failed`, dropped-mail warnings, `reply relayed`, `digest mailed`) now appear, and that nothing prints an address or body beyond what those lines already carry.
Quiet retries: the bridge retries every poll_secs (30 s). A failure that repeats every cycle must not print a line every 30 s forever: log the first occurrence of each failure kind at warn, repeats at debug, and one info line when it clears (state kept in memory per failure kind; keep it small). That is the "will it spam" answer for the log; it needs no new API.
Out of scope: failures as daemon EVENTS or messages to the aide (the ticket's second section is a design question for the human; do not build it); the mail routing change (br-843g); SES setup.
Serialize: br-843g edits the same bridle-mail and orchestrator.rs files; this task starts after it merges (dependency edge).
Migration: none. Docs: docs/design/mail.md (a line on logging and RUST_LOG), CHANGELOG.
Acceptance: just check passes; test (unit, with a captured subscriber) that a repeated failure logs once at warn and again only after clearing. Model: Sonnet.

## Thread

### note · external:orchestrator · 2026-10-08T00:42:12.150Z
Readied by orchestrator at the human's yes (~8:55 PM ET, via aide). Small bug: Haiku/Sonnet-small; events only on change of state (aide's reading of the human's 'will it spam' question).

### note · external:orchestrator · 2026-10-08T00:42:12.199Z
From orchestrator: br-gdyy is ready (small bug, the human's yes via aide): mail run installs no tracing subscriber, so SES send failures were silent. Place it after br-843g.
