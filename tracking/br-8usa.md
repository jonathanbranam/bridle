+++
id = "br-8usa"
title = "Re-pairing doesn't move a visitor's recorded home when its home project changes (part 2 of x8c8)"
kind = "fix"
state = "built"
created_at = "2026-10-14T21:56:44.715Z"
updated_at = "2026-10-14T23:57:31.663237Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:8usa",
]
branch = "bridle/8usa"
commit = "dab5ebd2"
summary = "Part 2 of x8c8. The hidden `token pair-check` helper takes `--home <p>` (with `--machine`) and fails when the visitor token's recorded home (`visitor_home` in /v1/status) differs; `token_pair.rs` passes the home it would mint with now (home_for) both in `one` (keep-or-mint) and in `token check`. A kept token with a stale home is therefore re-minted and replaces the holder's entry; no new daemon API, no migration. Test: repairing_a_visitor_whose_home_changed_records_the_new_home. Docs: cli.md (token pair), context/add-a-machine.md (home step), CHANGELOG. Caveat: after the orchestrator moves, run `bridle token pair` once the home project is enrolled."
ticket = "8usa"
+++

docs/tickets/open/re-pairing-doesn-t-move-a-visitor-s-recorded-home-when-its-h-8usa.md

## Thread

### note · agent:pm-1 · 2026-10-14T21:57:16.726Z
Brief (pm-1). Model: Sonnet. Size: S. The ticket body is the spec. Start: how a visitor token records its home (store.visitor_home; the mint request path in crates/bridle-daemon/src/server.rs) and where token_pair.rs mints visitor tokens; check first what a plain `token pair` does for an existing visitor token whose home changed (it likely "keeps" it: see br-su62's token check and keep-or-mint code, which landed before this). Make re-pairing update the recorded home (replace-on-mint, as xqwy did for peer tokens, or an update call); a kept token must be compared on home too. Tests: re-pair with a new home updates it and mail forwards there. Docs: cli.md (token pair), add-a-machine.md home-project and move steps (shared with br-8uxe, br-pmdh: run after them), CHANGELOG.md. Run after br-su62 and br-g9qm (same file). No migration. Acceptance: just check. If simple update isn't possible, say so on the thread and document the command to run instead.

### note · agent:8usa · 2026-10-14T23:52:14.696Z
done: token pair re-mints visitor tokens whose recorded home differs (pair-check --home); just check exit 0, 1687 tests passed; dab5ebd2

### note · agent:manager-2 · 2026-10-14T23:57:31.663Z
built: dab5ebd2 (branch bridle/8usa)
