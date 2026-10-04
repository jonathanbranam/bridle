+++
id = "br-2qrk"
title = "Fix tools_only_test launchers for the 3397 project resolver (red main)"
kind = "bug"
state = "integrated"
created_at = "2026-10-04T19:06:10.124Z"
updated_at = "2026-10-04T19:58:45.763309Z"
created_by = "agent:manager-2"
watchers = ["agent:manager-2"]
size = "S"
priority = "high"
branch = "bridle/fix-red-main"
commit = "6cd412ac436f1ef0087079e33d2b1b83520ce679"
summary = "Fixed br-3397 regression: tests now set BRIDLE_PROJECT=bridle explicitly. Failing tests launchers_start_elsewhere and launchers_run_under_a_bridle_agent_with_the_test_flag depended on BRIDLE_PROJECT in the environment after br-3397 removed the bridle fallback. Updated run_script_env in crates/bridle/tests/tools_only_test.rs to set BRIDLE_PROJECT. All 1170 tests pass; tested with env -u BRIDLE_PROJECT to verify the fix is robust."
+++

Main is red at d3803242 (br-3397): bridle::tools_only_test launchers_start_elsewhere (tools_only_test.rs:184) and launchers_run_under_a_bridle_agent_with_the_test_flag (:207) fail 'assertion failed: started' on CI. 3397 removed bridle session's 'bridle' fallback; these tests run the launch scripts with a temp BRIDLE_HOME and no --project/BRIDLE_PROJECT, so locally the repo's workspace daemon.json resolves the project but CI has none. Fix: in run_script / run_script_as_agent set BRIDLE_PROJECT (or --project) so the launch doesn't depend on the host workspace. Test file only. Acceptance: just check passes; also run the two tests with HOME and workspace discovery unable to resolve a project if practical. Critical: jumps the queue.

## Thread

### note · agent:manager-2 · 2026-10-04T19:06:10.124Z
priority: normal -> high

### note · agent:fix-red-main · 2026-10-04T19:45:13.128Z
done: Set BRIDLE_PROJECT explicitly in test launcher scripts; c97799aa2d8e1a406e3a9b28702c237d6f8a2eca

### note · agent:manager-2 · 2026-10-04T19:58:45.763Z
integrated: 6cd412ac436f1ef0087079e33d2b1b83520ce679 (branch bridle/fix-red-main)
