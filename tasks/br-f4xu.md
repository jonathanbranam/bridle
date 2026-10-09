+++
id = "br-f4xu"
title = "Flaky on macOS CI: process_test sigterm_via_signal_group_exits_143 exits 1, not 143"
kind = "bug"
state = "integrated"
created_at = "2026-10-09T12:58:04.639Z"
updated_at = "2026-10-09T15:49:40.028784Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/f4xu"
commit = "fa22d829faa23e934c1a2114792b232c441f6098"
summary = "Flake cause: fake-claude's SIGTERM handler used sys.exit(143), so the exit code went through normal interpreter shutdown while the stdin reader thread was blocked, which can change the code (saw 1). Handler now os._exit(143) (stdout already flushed per emit). Test unchanged. Looped sigterm_via_signal_group 30 of 30 green; just check exit 0, 1384 tests. Cause is inferred from the failure, not reproduced before the fix."
ticket = "f4xu"
+++

docs/tickets/open/flaky-on-macos-ci-process-test-sigterm-via-signal-group-exit-f4xu.md

## Thread

### note · external:orchestrator · 2026-10-09T12:58:04.969Z
settle skipped by external:orchestrator: critical: a flaky test broke CI on main (run 37931990047)

### note · agent:f4xu · 2026-10-09T14:23:52.789Z
done: fake-claude SIGTERM handler now os._exit(143) (was sys.exit, shutdown race with stdin reader thread); looped 30 of 30 green; just check exit 0, 1384 tests; tip b8ebd263

### note · external:orchestrator · 2026-10-09T14:24:18.493Z
orchestrator: read the diff (b8ebd263): fake-claude.py SIGTERM handler sys.exit(143) -> os._exit(143), skipping interpreter shutdown that raced the stdin reader thread. Real behaviour change in the test fake only; OK to land. CI on main (macOS) is the proof.

### note · agent:manager-2 · 2026-10-09T15:49:14.555Z
integrated: fa22d829faa23e934c1a2114792b232c441f6098 (branch bridle/f4xu)

### note · agent:manager-2 · 2026-10-09T15:49:40.028Z
cleanup: removed agent f4xu, branch bridle/f4xu
