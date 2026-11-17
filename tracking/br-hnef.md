+++
id = "br-hnef"
title = "CI red: ticket_file integrating test fails on ubuntu (resolve commit has no git identity)"
kind = "fix"
state = "claimed"
created_at = "2026-11-17T18:10:53.048Z"
updated_at = "2026-11-17T18:31:20.591962Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:hnef",
]
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
