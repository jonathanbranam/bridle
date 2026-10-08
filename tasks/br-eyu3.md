+++
id = "br-eyu3"
title = "Focus nudge is shared by every session on the machine: one session's prompt uses up the 5-minute nudge and the session the human is typing in stays silent"
kind = "bug"
state = "planned"
created_at = "2026-10-08T12:53:50.556Z"
updated_at = "2026-10-08T16:44:43.062713Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "S"
priority = "high"
priority_at = "2026-10-08T12:53:50.557Z"
summary = "The quiet-hours focus nudge is now recorded per session: gate() takes the hook's session_id and keeps '<secs> <period>' in $BRIDLE_HOME/focus-nudge.d/<session> (atomic temp+rename; files idle >1 day pruned), so each session gets its first nudge and its own 5-minute repeat. No usable session id falls back to the machine-wide focus-nudge file. Old-format machine-wide file is simply left/overwritten; no migration. Tests added for per-session and no-session cases; cli.md and CHANGELOG updated. Caveat: full just check not yet green here, see task comment."
+++

Seen 2026-10-08 in the "work" quiet period. ~/.bridle/focus-nudge is one file per machine (crates/bridle/src/focus.rs, STATE_FILE; gate() and nudge_due()). The bridle aide session prompted at 12:50:24Z and got the nudge (the file says 1791463824 work); the orchestrator session the human then typed in (12:50:54Z, 12:53:11Z, 12:53:22Z, per prompts.jsonl) got none, and the human noticed: "seems like you aren't prompting me to get back to work". With aides, the orchestrator and advisors all in quiet hours, whichever session prompts first takes the nudge.

Fix: keep the last-nudge time per session (the hook input's session_id, already read for prompts.jsonl): e.g. one line per session in focus-nudge, pruning entries older than a day, or a focus-nudge.d/<session> file. First prompt of a period in a session nudges; again after NUDGE_EVERY_SECS in that same session. No session_id: fall back to today's machine-wide behaviour. Write atomically (several sessions run the hook at once).
Tests: two sessions in one period each get their first nudge; the 5-minute repeat is per session; a missing session id behaves as before. Update the focus-hours doc (ticket cvaq / docs/design) to say per session.

## Thread

### note · external:orchestrator · 2026-10-08T12:53:50.557Z
priority: normal -> high

### note · agent:pm-1 · 2026-10-08T12:54:24.554Z
pm-1: Model Sonnet. Acceptance: just check passes. Migration: none (the old machine-wide focus-nudge file is just ignored/overwritten; say what happens to it in the done note). Out of scope: other focus-hours behaviour.

### note · agent:focusnudge · 2026-10-08T16:44:36.911Z
Checks: fmt/clippy pass, new focus tests written. Full nextest failed once on bridle-daemon upgrade_test a_long_drain_wakes_the_orchestrator_once (unrelated, timing); reruns were killed by timeouts because machine load average is ~44 (tests SIGTERM at 20s). Need a quieter machine for a green just check.

### note · agent:focusnudge · 2026-10-08T16:44:43.062Z
implemented per-session focus nudge, commit ac74cbb6 (merged main, nothing new). just check NOT green: one unrelated daemon upgrade_test failure then reruns timed out at load avg ~44. Fmt/clippy fine. Can you land with your own check, or tell me to retry later?
