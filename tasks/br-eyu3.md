+++
id = "br-eyu3"
title = "Focus nudge is shared by every session on the machine: one session's prompt uses up the 5-minute nudge and the session the human is typing in stays silent"
kind = "bug"
state = "open"
created_at = "2026-10-08T12:53:50.556Z"
updated_at = "2026-10-08T12:53:50.599735Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "S"
priority = "high"
priority_at = "2026-10-08T12:53:50.557Z"
+++

Seen 2026-10-08 in the "work" quiet period. ~/.bridle/focus-nudge is one file per machine (crates/bridle/src/focus.rs, STATE_FILE; gate() and nudge_due()). The bridle aide session prompted at 12:50:24Z and got the nudge (the file says 1791463824 work); the orchestrator session the human then typed in (12:50:54Z, 12:53:11Z, 12:53:22Z, per prompts.jsonl) got none, and the human noticed: "seems like you aren't prompting me to get back to work". With aides, the orchestrator and advisors all in quiet hours, whichever session prompts first takes the nudge.

Fix: keep the last-nudge time per session (the hook input's session_id, already read for prompts.jsonl): e.g. one line per session in focus-nudge, pruning entries older than a day, or a focus-nudge.d/<session> file. First prompt of a period in a session nudges; again after NUDGE_EVERY_SECS in that same session. No session_id: fall back to today's machine-wide behaviour. Write atomically (several sessions run the hook at once).
Tests: two sessions in one period each get their first nudge; the 5-minute repeat is per session; a missing session id behaves as before. Update the focus-hours doc (ticket cvaq / docs/design) to say per session.

## Thread

### note · external:orchestrator · 2026-10-08T12:53:50.557Z
priority: normal -> high
