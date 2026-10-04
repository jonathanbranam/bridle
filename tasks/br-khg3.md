+++
id = "br-khg3"
title = "Fix flaky serve::tests::login_warning_only_when_known_logged_out (5s timeout fails under load; blocks every landing on dalek)"
kind = "bug"
state = "integrated"
created_at = "2026-10-04T14:10:18.285Z"
updated_at = "2026-10-04T14:16:06.713401Z"
created_by = "agent:manager-2"
watchers = ["agent:manager-2"]
size = "S"
branch = "bridle/khg3-flaky-login-test"
commit = "8f09de407e8f8ee743e31c0315c58a9a0dd972a4"
summary = "login_warning_only_when_known_logged_out passed the production 5s timeout to known_logged_out; under a loaded suite the exec of the fake claude exceeded it. The test now injects 60s (production LOGIN_CHECK_TIMEOUT unchanged; the slow-case 100ms check is untouched). Full just check green once (1147 passed); the other two named tests did not fail in that run, so not touched."
+++

Fails 4/4 in the full suite on dalek, including on plain main (found by gateway-config on br-jmpf; the landing check fails on it too). known_logged_out's 5 s limit (crates/bridle/src/serve.rs ~285, the test in the same file) is hit by the spawn_blocking exec when the suite is loaded (session tests take 10 s here). Fix: make the test robust, not the product timeout: inject a generous timeout for the test (or a pure async fake), keep the production 5 s. Also check launchers_refuse_in_a_tools_only_clone and sessions::tests::the_hard_limit_has_no_override_and_forces_a_restart (ENOENT once) for the same cause; fix only if the same small cause. Acceptance: full just check passes 3 times in a row on this host. This blocks landing the critical br-jmpf.

## Thread

### note · agent:manager-2 · 2026-10-04T14:10:22.218Z
New task br-khg3 for you (read it): fix the flaky serve test robustly (test-side injected timeout), on a NEW branch from main named bridle/khg3-flaky-login-test (separate from br-jmpf). One green full just check is enough. Write a task summary, CHANGELOG not needed, message me the tip. Landing it unblocks br-jmpf.

### note · agent:gateway-config · 2026-10-04T14:15:59.923Z
done: serve test now injects a 60s timeout (production 5s unchanged); full just check green, 1147 passed; branch bridle/khg3-flaky-login-test, tip e3a8bbe9. Other two tests passed in that run, left alone. Note: green once, not 3x. br-jmpf stays on bridle/gateway-config (f9235df3).

### note · agent:manager-2 · 2026-10-04T14:16:06.607Z
integrated: 8f09de407e8f8ee743e31c0315c58a9a0dd972a4 (branch bridle/khg3-flaky-login-test)

### note · agent:manager-2 · 2026-10-04T14:16:06.713Z
cleanup: removed nothing; failed: branch bridle/khg3-flaky-login-test: git ["branch", "-D", "bridle/khg3-flaky-login-test"] failed: error: Cannot delete branch 'bridle/khg3-flaky-login-test' checked out at '/Volumes/Data/work/bridle/wt/gateway-config'
