+++
id = "br-54sp"
title = "The daemon's automatic push of main raises a false 'push failed' alarm when the manager already pushed"
kind = "bug"
state = "integrated"
created_at = "2026-10-13T00:14:34.643Z"
updated_at = "2026-10-16T01:30:34.512148Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:54sp",
]
branch = "bridle/54sp"
commit = "d0fad88d764758d32e074b56bb94240a07109c43"
summary = "A rejected push of the integration branch (autopush tick and bridle push) now re-fetches and checks whether origin/<branch> contains local (autopush::origin_has_local); if so autopush returns Ok(false) and bridle push returns success, with a debug log and no push.failed or message. Any doubt (fetch fails) still alarms. Tests: lost-race case quiet, existing rejecting-origin case still errors. Docs: operating-model.md, CHANGELOG."
ticket = "54sp"
+++

docs/tickets/open/the-daemon-s-automatic-push-of-main-raises-a-false-push-fail-54sp.md

## Thread

### note · external:orchestrator · 2026-10-13T04:12:22.584Z
Recurred 2026-10-13 04:12Z (m-11943): 'cannot lock ref refs/heads/main: is at 4849f58b but expected 7f628d85' after manager-2 landed br-tmmj; local main and origin/main were both 4849f58b, so the push had gone through. (orchestrator)

### note · external:advisor/product-manager · 2026-10-15T22:51:58.278Z
Readied by the human via advisor product-manager, 2026-10-15 ~6:51 PM ET: "Yes ready approve 3-5."

### note · agent:pm-1 · 2026-10-15T22:52:36.278Z
Brief (pm-1). Model: Sonnet. Size: S. Fix per the ticket body. Code: crates/bridle-daemon/src/server.rs ~3483 (the "Push of <branch> to origin failed ... landing is on the local branch only" message) and the push function just above it. On a rejected push: `git fetch origin <branch>`, then if origin's tip equals or contains local (`git merge-base --is-ancestor <local> origin/<branch>`) log at debug and send no message; alarm only when local has commits origin lacks after the re-fetch (keep the existing text). The one-pusher hook (rule one-pusher-for-the-integration-branch) is unaffected: do not touch hooks. Tests: remote already at the local tip (pushed by someone else between fetch and push) gives no message; local ahead and the push refused still alarms. CHANGELOG; docs/design/agent-host only if the push behaviour is described there. Acceptance: just check. Out of scope: why two pushers race (the manager's own push stays).

### note · agent:54sp · 2026-10-16T01:05:50.640Z
built: c0370ef6 (branch bridle/54sp)

### note · agent:54sp · 2026-10-16T01:05:50.689Z
done: re-fetch before alarming on a rejected push; check exit 0, 1723 tests passed; c0370ef6

### note · agent:manager-2 · 2026-10-16T01:29:52.352Z
integrated: d0fad88d764758d32e074b56bb94240a07109c43 (branch bridle/54sp)

### note · agent:manager-2 · 2026-10-16T01:30:34.512Z
cleanup: removed agent 54sp, branch bridle/54sp
