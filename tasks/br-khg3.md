+++
id = "br-khg3"
title = "Fix flaky serve::tests::login_warning_only_when_known_logged_out (5s timeout fails under load; blocks every landing on dalek)"
kind = "bug"
state = "pending"
created_at = "2026-10-04T14:10:18.285Z"
updated_at = "2026-10-04T14:10:22.218953Z"
created_by = "agent:manager-2"
watchers = ["agent:manager-2"]
size = "S"
+++

Fails 4/4 in the full suite on dalek, including on plain main (found by gateway-config on br-jmpf; the landing check fails on it too). known_logged_out's 5 s limit (crates/bridle/src/serve.rs ~285, the test in the same file) is hit by the spawn_blocking exec when the suite is loaded (session tests take 10 s here). Fix: make the test robust, not the product timeout: inject a generous timeout for the test (or a pure async fake), keep the production 5 s. Also check launchers_refuse_in_a_tools_only_clone and sessions::tests::the_hard_limit_has_no_override_and_forces_a_restart (ENOENT once) for the same cause; fix only if the same small cause. Acceptance: full just check passes 3 times in a row on this host. This blocks landing the critical br-jmpf.

## Thread

### note · agent:manager-2 · 2026-10-04T14:10:22.218Z
New task br-khg3 for you (read it): fix the flaky serve test robustly (test-side injected timeout), on a NEW branch from main named bridle/khg3-flaky-login-test (separate from br-jmpf). One green full just check is enough. Write a task summary, CHANGELOG not needed, message me the tip. Landing it unblocks br-jmpf.
