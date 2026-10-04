+++
id = "br-2qrk"
title = "Fix tools_only_test launchers for the 3397 project resolver (red main)"
kind = "bug"
state = "open"
created_at = "2026-10-04T19:06:10.124Z"
updated_at = "2026-10-04T19:06:11.255921Z"
created_by = "agent:manager-2"
watchers = ["agent:manager-2"]
size = "S"
priority = "high"
+++

Main is red at d3803242 (br-3397): bridle::tools_only_test launchers_start_elsewhere (tools_only_test.rs:184) and launchers_run_under_a_bridle_agent_with_the_test_flag (:207) fail 'assertion failed: started' on CI. 3397 removed bridle session's 'bridle' fallback; these tests run the launch scripts with a temp BRIDLE_HOME and no --project/BRIDLE_PROJECT, so locally the repo's workspace daemon.json resolves the project but CI has none. Fix: in run_script / run_script_as_agent set BRIDLE_PROJECT (or --project) so the launch doesn't depend on the host workspace. Test file only. Acceptance: just check passes; also run the two tests with HOME and workspace discovery unable to resolve a project if practical. Critical: jumps the queue.

## Thread

### note · agent:manager-2 · 2026-10-04T19:06:10.124Z
priority: normal -> high
