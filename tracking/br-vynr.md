+++
id = "br-vynr"
title = "The orchestrator's relaunch after a handover stalls until tmux gets focus, and no incident fires"
kind = "fix"
state = "built"
created_at = "2026-10-13T03:09:39.758Z"
updated_at = "2026-10-13T03:30:42.931656Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wvynr",
]
priority = "critical"
priority_at = "2026-10-13T03:09:48.923693Z"
branch = "bridle/wvynr"
commit = "cd2a16b1"
summary = "A: the relaunch now runs tmux respawn-pane -k (fresh pty, no client needed) before typing the launcher; a relaunch unconfirmed after 60 s files an incident, and the retry runs the launcher as the pane's process with no keystrokes. Cause not reproduced (tmux focus/raw tty is the reporter's guess; stated in a code comment). B: the code already counts only external:orchestrator waits in waiter_open (opened() is called only from the orchestrator wake path); added a daemon test that an advisor wait leaves waiter_open false. The vesta report may have been an older daemon or the aide waiting as the orchestrator identity; unverified. Tests use the fake tmux; no real-tmux test. Docs: orchestrator-supervision.md section 4, CHANGELOG."
ticket = "vynr"
+++

docs/tickets/open/the-orchestrator-s-relaunch-after-a-handover-stalls-until-tm-vynr.md

## Thread

### note · external:orchestrator · 2026-10-13T03:09:46.423Z
Readied by orchestrator as critical (my call under the human's 2026-10-09 rule: "for anything critical, you can go ahead"): it leaves a machine with no orchestrator until a human notices. Report approved by the human on vesta ("Yes ok but report."), relayed by aide@nuc m-11894.

### note · external:orchestrator · 2026-10-13T03:09:48.923Z
priority: normal -> critical

### note · agent:pm-1 · 2026-10-13T03:10:01.302Z
Brief (pm-1). Model: Sonnet. Size: M, two independent fixes in one branch (A first; B is small and stands alone if A needs a decision). CRITICAL: first in the queue.

A. The relaunch must not depend on tmux focus. Read the ask's evidence first. Start: crates/bridle-daemon/src/orchestrator.rs (the relaunch's send-keys), crates/bridle/src/session.rs, rule tmux-shell-first (workflow/base/rules). The reporter's guess (unchecked) is that Claude's terminal teardown waits for a terminal reply that comes only with an attached, focused client, leaving the tty in raw mode, so typed keys echo as ^M. Reproduce locally if you can: a detached tmux session (`tmux new-session -d`) running the same launch path, no attached client. Fix, simplest that is robust: before typing the launcher, make the pane safe (e.g. `tmux respawn-pane -k` running the launcher directly, which starts a fresh process with a clean tty and needs no keystrokes; or `stty sane` first), and verify the launcher actually started (the process or the orchestrator.exits/status), retry another way once, and if still not running raise an incident instead of staying silent. Say in a comment which cause you found and what you did not manage to verify; do not claim a cause you did not reproduce. A test covers the relaunch path with no attached client where tmux is available in the test environment (skip cleanly where it is not, per project test rules; no real claude: use the fake).

B. The no-waiter incident counts only the orchestrator's waiter: in crates/bridle-daemon/src/server.rs find the waiter tracking (`waiter_open`) and the 15-minute incident; an external session of another identity (an aide's waiter) must not satisfy it. Test: only an aide waiter open -> the incident still fires after the interval (use the existing clock/test hooks).

Docs: docs/design/agent-host/orchestrator-supervision.md (or wherever the relaunch and the incident are described), CHANGELOG.md. Migration: none. Out of scope: br-5j35 (restart from inside a session), the interactive-session redesign.

### note · external:orchestrator · 2026-10-13T03:10:24.204Z
settle skipped by external:orchestrator: critical: machines can be left with no orchestrator after a handover (orchestrator)

### note · agent:wvynr · 2026-10-13T03:30:40.602Z
built: cd2a16b1 (branch bridle/wvynr)

### note · agent:wvynr · 2026-10-13T03:30:42.931Z
done: relaunch respawns pane first, incident + keystroke-free retry when unconfirmed; waiter test added; just check exit 0, 1639 passed (332s), ran on code-identical parent of merge tip cd2a16b1 (main merge brought docs only). Caveats on ticket summary: cause not reproduced, B was already correct in code, no real-tmux test.
