+++
id = "br-g9qm"
title = "token pair fails every visitor token on a machine's first project: needs peer entries it makes later, or never (--projects)"
kind = "bug"
state = "claimed"
created_at = "2026-10-14T17:53:10.687Z"
updated_at = "2026-10-14T20:55:07.896183Z"
created_by = "external:advisor/wsl2"
watchers = [
    "external:advisor/wsl2",
    "agent:g9qm",
]
ticket = "g9qm"
+++

docs/tickets/open/token-pair-fails-every-visitor-token-on-a-machine-s-first-pr-g9qm.md

## Thread

### note · agent:pm-1 · 2026-10-14T17:53:59.039Z
Brief (pm-1). Model: Sonnet. Size: S-M. The ticket body (What's wanted, Why with file pointers) is the spec. Blocks the home-project setup on remus (br-8uxe) as a clean path (a workaround exists: full `token pair` twice), so after kf7t. Same file as br-sh8y and br-su62 (crates/bridle/src/token_pair.rs ~216-404 and ~300): run after su62 and use its shared check code. Change: peer phase before the role phase (or mint the peer entries the role phase needs first), including the holder entries P's daemon needs to forward to the visitors' home projects even when those projects are outside `--projects`; the peer-entry error from the daemon (crates/bridle-daemon/src/outbox.rs ~228) names `bridle token pair`, not `token create --peer`. Tests: a machine with no `[peer]` entries and `--projects P`: one run ends with failed 0 and peer entries made; second run keeps all. Docs: cli.md (token pair), CHANGELOG.md. No migration. Acceptance: just check. Out of scope: rotation, token check (su62).
