+++
id = "br-8usa"
title = "Re-pairing doesn't move a visitor's recorded home when its home project changes (part 2 of x8c8)"
kind = "fix"
state = "planned"
created_at = "2026-10-14T21:56:44.715Z"
updated_at = "2026-10-14T22:18:09.211818Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "8usa"
+++

docs/tickets/open/re-pairing-doesn-t-move-a-visitor-s-recorded-home-when-its-h-8usa.md

## Thread

### note · agent:pm-1 · 2026-10-14T21:57:16.726Z
Brief (pm-1). Model: Sonnet. Size: S. The ticket body is the spec. Start: how a visitor token records its home (store.visitor_home; the mint request path in crates/bridle-daemon/src/server.rs) and where token_pair.rs mints visitor tokens; check first what a plain `token pair` does for an existing visitor token whose home changed (it likely "keeps" it: see br-su62's token check and keep-or-mint code, which landed before this). Make re-pairing update the recorded home (replace-on-mint, as xqwy did for peer tokens, or an update call); a kept token must be compared on home too. Tests: re-pair with a new home updates it and mail forwards there. Docs: cli.md (token pair), add-a-machine.md home-project and move steps (shared with br-8uxe, br-pmdh: run after them), CHANGELOG.md. Run after br-su62 and br-g9qm (same file). No migration. Acceptance: just check. If simple update isn't possible, say so on the thread and document the command to run instead.
