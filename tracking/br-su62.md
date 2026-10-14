+++
id = "br-su62"
title = "token pair kept a refused token, and doesn't confirm the tokens work afterwards"
kind = "bug"
state = "claimed"
created_at = "2026-10-14T11:35:15.533Z"
updated_at = "2026-10-14T19:34:17.850257Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:su62",
]
ticket = "su62"
+++

docs/tickets/open/token-pair-kept-a-refused-token-and-doesn-t-confirm-the-toke-su62.md

## Thread

### note · external:aide · 2026-10-14T11:35:21.285Z
From the human, via aide: "Can we improve token pair to actually check and see if the tokens are accepted?" Filed as br-su62 (bug): this morning's plain token pair reported 'kept' for the orchestrator's meta-notes token that vesta was refusing; --rotate fixed it. Related to br-nbgp (critical, in progress). Please plan.

### note · external:aide · 2026-10-14T11:48:43.473Z
From the human, via aide: "Yes please" to adding a read-only 'bridle token check' (every credentials entry checked from its holder against its daemon, accepted/refused per line, non-zero on refusal) that token pair also uses for keep-or-mint and as its final step. Design added to the ticket, with a lead on the cause (dalek's registry still says meta-notes is on 'nuc').

### note · external:advisor/product-manager · 2026-10-14T11:52:47.777Z
advisor (product-manager), 2026-10-14 ~7:55 AM ET: the 'registry still says nuc' lead doesn't hold now: dalek's ~/.bridle/config.toml [projects] has meta-notes = { machine = "vesta", port = 7402 } (and notes, dotfiles-local, meta-notes-ui on vesta). Look elsewhere for why plain pair reported 'kept' for a refused token (e.g. the check itself, or the nbgp token loss happening after the check).

### note · external:advisor/product-manager · 2026-10-14T11:53:23.700Z
The human, 2026-10-14 ~7:57 AM ET, verbatim, on the registry lead: "I fixed that first thing before any of the project rebuild or token rotating". So the stale registry was not the cause.

### note · agent:pm-1 · 2026-10-14T12:45:58.885Z
Brief (pm-1). Model: Sonnet. Size: M. The ticket body (ask + design) is the spec. Related: br-378y (landed 'kept 0' fix, read its landing first) and br-nbgp.

Step 1 (find why a refused token counted as kept; report the finding on the thread BEFORE building): the hidden `bridle token pair-check` runs over ssh on the holder, reads its credentials entry, resolves the project's URL from the holder's registry, calls GET /v1/status. The registry-says-nuc lead is dead (the human fixed the registry before the run). Look for: a check that returns true on a cached/earlier result, a visitor-vs-role entry mixup, the check hitting a different daemon than the one that refuses, or a 401 treated as pass. Reproduce in a test with a fake daemon that refuses the token.
Step 2: add `bridle token check [--machines m,..] [--projects p,..] [--roles r,..] [--json]` (read-only, one line per machine/role/project: accepted or refused with reason (invalid token, no home, unreachable, no entry), non-zero on any refusal). token pair uses the same code for keep-or-mint and as its final step (prints the same lines, non-zero on refusal).

Files: crates/bridle/src/token_pair.rs (br-sh8y also edits it: run after sh8y, and use its remote_bridle_cmd helper), cli.rs, docs/design/cli.md (token pair / token check), CHANGELOG.md. Tests: fake daemon refusing a token -> plain pair re-mints; check reports accepted/refused/unreachable lines and exit codes. No migration. Acceptance: just check. Stop point: if step 1 finds the cause is large, land the check command first and tell the manager. Out of scope: rotation policy changes.
