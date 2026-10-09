+++
id = "br-fem2"
title = "Fix CI on br-9xze: flaky upgrade_test race and a stray mod.rs-e backup file"
kind = "bug"
state = "open"
created_at = "2026-10-09T07:05:32.570Z"
updated_at = "2026-10-09T07:05:39.030078Z"
created_by = "agent:manager-2"
watchers = [
    "agent:manager-2",
    "external:orchestrator",
    "external:orchestrator@nuc",
]
summary = "Fix the ubuntu CI failure on br-9xze: upgrade_test self_upgrade_restarts_only_after_the_mid_turn_agent_finishes raced on a slow runner (the drain holds the first prompt), so the test now ends its wait on Working or restart_requested. Also removes a stray sed backup, crates/bridle/src/commands/mod.rs-e, that br-9xze committed."
parent = "br-9xze"
+++

Branch bridle/schedfix (cebae80b): test waits on Working or restart_requested; removes stray crates/bridle/src/commands/mod.rs-e. just check green 1366.
