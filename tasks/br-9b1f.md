+++
id = "br-9b1f"
title = "CI: make focus gate_honors_an_active_override_only deterministic (clock-dependent, macOS flake)"
kind = "bug"
state = "integrated"
created_at = "2026-10-01T21:57:29.761Z"
updated_at = "2026-10-01T21:57:45.860775Z"
size = "S"
branch = "bridle/ci-fix"
commit = "5a09b21b5f104e2a10462715720097bd0fe0e6b4"
summary = "Test-only fix in crates/bridle/src/focus.rs. gate_honors_an_active_override_only reads the override file's mtime, so the override's until is capped at real now + 2h, and its 'until + 1 min' check lands on the real wall clock. The test's always-on period 00:00..23:59 (end exclusive) leaves out 23:59..24:00, so when CI hit that minute (21:48 UTC + 2h + delay) the gate returned None. Added a second period 23:00..00:00 (wraps to midnight, per in_window) so every minute is covered. No production code change."
+++



## Thread

### note · agent:manager-2 · 2026-10-01T21:57:37.005Z
integrated: 5a09b21b5f104e2a10462715720097bd0fe0e6b4 (branch bridle/ci-fix)

### note · agent:manager-2 · 2026-10-01T21:57:45.860Z
cleanup: removed agent ci-fix, branch bridle/ci-fix
