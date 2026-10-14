+++
id = "br-g9qm"
title = "token pair fails every visitor token on a machine's first project: needs peer entries it makes later, or never (--projects)"
kind = "bug"
state = "integrated"
created_at = "2026-10-14T17:53:10.687Z"
updated_at = "2026-10-14T22:03:28.734324Z"
created_by = "external:advisor/wsl2"
watchers = [
    "external:advisor/wsl2",
    "agent:g9qm",
]
branch = "bridle/g9qm"
commit = "7d5be990e852758c7e29e8b484f2222922f93693"
summary = "token pair now runs the peer phase before the role phase, and Plan.all_up lets peer entries point at every project even under --projects, so a machine's first project pairs in one run. Daemon 'no peer token' error now names 'bridle token pair --tokens peer'. Test projects_filter_still_makes_peer_entries_first; cli.md and CHANGELOG updated. Caveat: the sim doesn't model the daemon's forward check, so the test checks order and peer entries, not the original failure."
ticket = "g9qm"
+++

docs/tickets/open/token-pair-fails-every-visitor-token-on-a-machine-s-first-pr-g9qm.md

## Thread

### note · agent:pm-1 · 2026-10-14T17:53:59.039Z
Brief (pm-1). Model: Sonnet. Size: S-M. The ticket body (What's wanted, Why with file pointers) is the spec. Blocks the home-project setup on remus (br-8uxe) as a clean path (a workaround exists: full `token pair` twice), so after kf7t. Same file as br-sh8y and br-su62 (crates/bridle/src/token_pair.rs ~216-404 and ~300): run after su62 and use its shared check code. Change: peer phase before the role phase (or mint the peer entries the role phase needs first), including the holder entries P's daemon needs to forward to the visitors' home projects even when those projects are outside `--projects`; the peer-entry error from the daemon (crates/bridle-daemon/src/outbox.rs ~228) names `bridle token pair`, not `token create --peer`. Tests: a machine with no `[peer]` entries and `--projects P`: one run ends with failed 0 and peer entries made; second run keeps all. Docs: cli.md (token pair), CHANGELOG.md. No migration. Acceptance: just check. Out of scope: rotation, token check (su62).

### note · agent:g9qm · 2026-10-14T21:06:17.695Z
g9qm worker: code, test, docs, CHANGELOG written on bridle/g9qm (uncommitted); just check running in background, will commit and report on green.

### note · agent:g9qm · 2026-10-14T21:43:45.090Z
done: peer phase first, --projects keeps needed peer entries, error names token pair; just check exit 0, 1684 tests passed; checked commit 7da784fc, merge of main (docs only, no re-run) is 2db4b9a2e974e22e05c2b2f95b698546d618516b

### note · agent:manager-2 · 2026-10-14T21:43:51.121Z
built: 2db4b9a2e974e22e05c2b2f95b698546d618516b (branch bridle/g9qm)

### note · agent:manager-2 · 2026-10-14T22:03:28.734Z
integrated: 7d5be990e852758c7e29e8b484f2222922f93693 (branch bridle/g9qm)
