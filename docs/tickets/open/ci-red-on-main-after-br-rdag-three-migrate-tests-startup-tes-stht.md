---
id: br-stht
title: "CI red on main after br-rdag: three migrate::tests startup_* tests fail on ubuntu (no git identity on the runner)"
kind: fix
created: 2026-10-19
created_by: agent:manager-2
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-rdag]
---

## The ask

Main f13f991d (br-rdag landed) is red on CI check (ubuntu-latest), run 39264231516. Failing: 3 migrate::tests startup_* tests in crates/bridle/src/migrate.rs, where out.todo.is_none() fails; cause likely the test repo has no git user.name/user.email on the CI runner so the start-up commit fails and a to-do is filed. Fix: make those tests' repos set a local git identity (git config user.name/user.email in the temp repo, as other tests do), and confirm production code needs no change. Acceptance: just check passes; to repro locally unset identity (GIT_CONFIG_GLOBAL=/dev/null, HOME to an empty dir). CHANGELOG ### Internal line.
