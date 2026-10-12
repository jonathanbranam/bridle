+++
id = "br-ajp5"
title = "A release daemon guesses bridle's repo from the project's origin: wrong workflow fetch and upgrades, and git prompts"
kind = "bug"
state = "claimed"
created_at = "2026-10-12T03:03:27.217Z"
updated_at = "2026-10-12T04:31:30.832835Z"
created_by = "external:advisor/nuc-rename"
watchers = [
    "external:advisor/nuc-rename",
    "agent:wajp5",
]
priority = "high"
priority_at = "2026-10-12T03:03:48.275619Z"
ticket = "ajp5"
+++

docs/tickets/open/a-release-daemon-guesses-bridle-s-repo-from-the-project-s-or-ajp5.md

## Thread

### note · external:advisor/product-manager · 2026-10-12T03:03:48.275Z
priority: normal -> high

### note · agent:pm-1 · 2026-10-12T03:13:35.097Z
Build brief (pm-1). Model: Sonnet. High: the human sets up remus (WSL2) on 2026-10-12 and needs this first. Build the "Wanted" list above, nothing more.

Do:
1. A release daemon never takes bridle's repo from the project's origin: with no `release_repo`, default to bridle's own GitHub repo (the same default `daemon init` uses for workflow_url, cli.md:74; reuse that constant, do not repeat the string). Change crates/bridle-daemon/src/lib.rs (~456-490, ensure_own_workflow) and wherever the release check takes the same fallback (release.rs, upgrade.rs). An explicit `release_repo` still wins. Keep the workflow fetch's URL form consistent with that repo (workflow_checkout.rs:116 is https; fine for a public repo).
2. Every git network call the daemon makes sets GIT_TERMINAL_PROMPT=0 (workflow_checkout::ensure and any other clone/fetch; ci.rs already does, share its helper if there is one) so a failure is an error in the log, not a prompt.
3. `bridle doctor` flags a release daemon with no workflow checkout, or whose release repo is not bridle's (warn, with the fix line).
4. Docs: docs/context/add-a-machine.md and windows-wsl2-host.md (a working [daemon] block, or none needed now), daemon.md "Release upgrade" and "Workflow checkout", CHANGELOG.

Acceptance: just check passes; tests: no release_repo -> bridle's repo, not the project's origin; explicit release_repo wins; clone/fetch commands carry GIT_TERMINAL_PROMPT=0; doctor warns for an empty workflow dir and for a foreign release repo. No real network.
Migration: none; machines with a hand-set release_repo keep working. Out of scope: the NUC's hand fix (already done), the one-token-per-machine design, a release-repo setting per project.

### note · agent:wajp5 · 2026-10-12T04:31:30.832Z
wajp5: code, tests and docs done on bridle/wajp5; targeted tests pass; full just check running (slow build, ~20 min). Will mark built and report when it exits 0.
