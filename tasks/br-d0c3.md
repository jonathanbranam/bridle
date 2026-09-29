+++
id = "br-d0c3"
title = "CI green: tests init repos on main, isolate global git config (g3ck)"
kind = "bug"
state = "planned"
created_at = "2026-09-29T04:59:29.277Z"
updated_at = "2026-09-29T05:08:53.492921Z"
summary = "Fixes 1 (git init -b main) and 3 (startup check on unset branches.integration) were already on main. Added the missing one: just test runs with GIT_CONFIG_GLOBAL=/dev/null, GIT_CONFIG_NOSYSTEM=1. just check passes under a global init.defaultBranch=master. Resolved ticket g3ck. Note: one run flaked once on lifecycle_test spawn_child_orphan_is_swept_on_stop (timeout waiting for agent.orphans_killed), passed on rerun."
+++

Ticket: docs/questions/open/g3ck (ci-red-tests-assume-main-branch-g3ck.md). First check whether it is already fixed on main (tests using `git init -b main`, config isolation, startup check); if fully done, just resolve the ticket per docs/README.md and say so in the thread. Otherwise do the three fixes listed under 'The fix': tests create repos with `git init -b main` everywhere; tests run git with GIT_CONFIG_GLOBAL pointing at an empty file (user.name/email set explicitly) so local runs match CI; when [branches] integration is unset and main doesn't exist, fail at config load/startup with a clear message naming the setting. Acceptance: `just check` passes with a global git config whose init.defaultBranch is master (simulate by GIT_CONFIG_GLOBAL). Model: Sonnet.
