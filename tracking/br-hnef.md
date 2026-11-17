+++
id = "br-hnef"
title = "CI red: ticket_file integrating test fails on ubuntu (resolve commit has no git identity)"
kind = "fix"
state = "integrated"
created_at = "2026-11-17T18:10:53.048Z"
updated_at = "2026-11-17T18:47:50.573476Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:hnef",
]
branch = "bridle/hnef"
commit = "13aa62e81c7817ec567cc72e4ec3708d7dbe4ea2"
summary = "The ticket_file test helper repo_with_open_ticket now sets user.name/user.email in the temp repo's config, so resolve_and_commit's commit (which runs git without the helper's -c identity) works on CI runners with no global git identity. Test-only change; no production code touched. The other test in the file (stale_open_files...) uses the same helper and is covered too. Reproduced with HOME=empty temp dir, GIT_CONFIG_GLOBAL and GIT_CONFIG_SYSTEM=/dev/null: both ticket_file tests pass. CHANGELOG Internal line added."
ticket = "hnef"
+++

docs/tickets/open/ci-red-ticket-file-integrating-test-fails-on-ubuntu-resolve-hnef.md

## Thread

### note · external:orchestrator · 2026-11-17T18:11:05.115Z
readied by external:orchestrator

### note · external:orchestrator · 2026-11-17T18:11:05.229Z
Filed and readied by orchestrator: critical, main is red (rule ready-without-asking; CI on main is critical).

### note · agent:manager-2 · 2026-11-17T18:22:15.610Z
br-hnef (critical, CI red on main): needs planning and a Tier 1 queue slot so a worker can be spawned. Spawn is refused: ticket is Open, not planned.

### note · agent:hnef · 2026-11-17T18:47:40.510Z
built: 72036fdeb514eb2de91774ec491de2fc84ca27d3 (branch bridle/hnef)

### note · agent:hnef · 2026-11-17T18:47:40.558Z
done: set git identity in ticket_file test repo helper (test-only, plus CHANGELOG Internal line); clean-env just check exit 0, 2015 tests run, 2015 passed, 6 skipped; commit 72036fdeb514eb2de91774ec491de2fc84ca27d3 (branch bridle/hnef, pushed). Repro with empty HOME and GIT_CONFIG_GLOBAL/SYSTEM=/dev/null: both ticket_file tests pass.

### note · agent:manager-2 · 2026-11-17T18:47:50.573Z
integrated: 13aa62e81c7817ec567cc72e4ec3708d7dbe4ea2 (branch bridle/hnef)
