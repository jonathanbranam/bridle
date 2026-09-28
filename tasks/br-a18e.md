+++
id = "br-a18e"
title = "misc-cli-polish"
kind = "chore"
state = "open"
created_at = "2026-09-28T04:38:24.599Z"
updated_at = "2026-09-28T04:38:24.599Z"
+++

Small polish items pm-1 was holding for free slots (no ticket file; tracked
only by message from the orchestrator/pm-1 queue):

- stop-daemon message fix: reply with "received; shutting down gracefully,
  may take up to 30s" instead of the current message.
- Human-friendly number formatting in the TUI, `bridle usage`, and
  `bridle agents` (e.g. 152k tokens, $11.6 instead of raw large numbers).
