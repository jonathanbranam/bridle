+++
id = "br-myw9"
title = "A new machine needed two claude logins before the daemon's agents were authenticated"
kind = "bug"
state = "dropped"
created_at = "2026-10-14T18:13:41.891Z"
updated_at = "2026-10-14T18:15:14.214551Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "myw9"
+++

docs/tickets/open/a-new-machine-needed-two-claude-logins-before-the-daemon-s-a-myw9.md

## Thread

### note · agent:pm-1 · 2026-10-14T18:14:09.050Z
Brief for when it is readied (pm-1). Model: Sonnet. Size: S, investigation first; low priority (remus works; worst case one extra login per new machine). Step 1: find where the daemon decides "not logged in" (NOT_LOGGED_IN_REASON in crates/bridle-daemon/src/supervisor.rs, the login check in serve.rs) and where a spawned `claude` finds credentials on Linux/WSL2 (credentials file vs keychain, HOME and CLAUDE_CONFIG_DIR as the systemd unit sets them). Likely cause to test: the daemon service runs with a different HOME/config dir or environment than the login shell, or the check caches a result from before the login. Report the finding on the thread. Step 2: if it is a small bug, fix it with a test; otherwise add the answer to docs/context/add-a-machine.md (which login, run as which user, restart order) and CHANGELOG if code changed. No migration. Acceptance: just check. Out of scope: changing how Claude Code authenticates.

### note · external:orchestrator · 2026-10-14T18:15:01.791Z
orchestrator: cause found by the human (m-12839, via orchestrator@remus): the daemon was started before Claude was logged in on the new machine. The fix is docs: br-pmdh (filed and readied on the human's approval). This bug can be resolved once pmdh lands, or now if the PdM prefers.

### note · agent:pm-1 · 2026-10-14T18:15:14.214Z
dropped: Cause found: new-machine docs put the login after the first serve; fixed by docs ticket br-pmdh
