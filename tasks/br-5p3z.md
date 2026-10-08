+++
id = "br-5p3z"
title = "Flaky/red: upgrade_test a_drain_holds_new_turns_and_delivers_them_after_the_restart fails in just check"
kind = "bug"
state = "planned"
created_at = "2026-10-08T01:02:18.912Z"
updated_at = "2026-10-08T01:02:44.532124Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
size = "S"
+++

Evidence: two workers' `just check` runs on docs-only branches failed on this one test, crates/bridle-daemon/tests/upgrade_test.rs, test `a_drain_holds_new_turns_and_delivers_them_after_the_restart`: br-xv2n (handover2, 2026-10-07) and br-at2j (wsl2guide, 2026-10-08, exit 100, 1 failed). Neither branch touched code. Read both threads for the panic line.
Goal: find why it fails and fix the test or the code so `just check` is reliably green. First decide whether it is a race in the test (timing, a fixed sleep, a port or file shared with another test under nextest parallelism), a real bug in the drain/restart path, or a regression from a recent landing (compare with git log -n 20 for crates/bridle-daemon/src upgrade and drain code, e.g. br-2672 wake changes, br-3kdc handover). Reproduce with `cargo nextest run -p bridle-daemon a_drain_holds` repeated (loop it 20 times in one-command-per-call form or with nextest --retries 0 --run-ignored as needed) and state the failure rate in the done note.
Fix the cause, not the symptom: no blanket retry, no #[ignore]. If it is a real product bug, say so on the task before changing behaviour.
Acceptance: the test passes 20 of 20 runs alone and in the full `just check`; done note names the cause. Model: Sonnet.
Out of scope: other flaky tests (file them separately).
