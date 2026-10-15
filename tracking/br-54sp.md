+++
id = "br-54sp"
title = "The daemon's automatic push of main raises a false 'push failed' alarm when the manager already pushed"
kind = "bug"
state = "planned"
created_at = "2026-10-13T00:14:34.643Z"
updated_at = "2026-10-15T23:04:02.810733Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
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
