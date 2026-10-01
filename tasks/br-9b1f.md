+++
id = "br-9b1f"
title = "CI: make focus gate_honors_an_active_override_only deterministic (clock-dependent, macOS flake)"
kind = "bug"
state = "open"
created_at = "2026-10-01T21:57:29.761Z"
updated_at = "2026-10-01T21:57:33.418916Z"
size = "S"
summary = "Test-only fix in crates/bridle/src/focus.rs. gate_honors_an_active_override_only reads the override file's mtime, so the override's until is capped at real now + 2h, and its 'until + 1 min' check lands on the real wall clock. The test's always-on period 00:00..23:59 (end exclusive) leaves out 23:59..24:00, so when CI hit that minute (21:48 UTC + 2h + delay) the gate returned None. Added a second period 23:00..00:00 (wraps to midnight, per in_window) so every minute is covered. No production code change."
+++


