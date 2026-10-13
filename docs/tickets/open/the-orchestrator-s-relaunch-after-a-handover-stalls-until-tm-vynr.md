---
id: br-vynr
title: The orchestrator's relaunch after a handover stalls until tmux gets focus, and no incident fires
kind: fix
created: 2026-10-13
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Critical (orchestrator, 2026-10-13): after a handover the orchestrator's relaunch can stall until a human gives the tmux client focus, so every daemon on the machine runs with no orchestrator until someone notices. Reported by the meta-notes aide on vesta, approved by the human ("Yes ok but report."), relayed by aide@nuc (m-11894).

What happened (vesta, daemons then on 0.6.3):

1. 2026-10-12 03:31:48 UTC: handover h-0034 (meta-notes, context 151K) replied "handover marked done; the session will be relaunched shortly". No relaunch. At 03:42 the human found the pane: "the tmux pane that I saw did not show claude at all and had a few ^M control characters in it. Looked like a tty issue or something. I did force shut it down with C-C because it wasn't rresponding and looked dead." The only exit in ~/.bridle/orchestrator.exits is that Ctrl-C (03:42:08Z, exit 130).
2. 2026-10-12 15:42 UTC: handover h-0035 (12 h uptime). Claude exited and printed `Resume this session with: claude --resume "orch-meta-notes"`. The relaunch then typed `.bridle/claude-orchestrator` + Enter into pane 0:1.3 five times; each shows as `.bridle/claude-orchestrator^M` and none ran. Nothing in orchestrator.exits. All four vesta daemons (meta-notes, notes, meta-notes-ui, dotfiles-local) had no orchestrator for ~6.5 h. At 22:13:04 UTC the human focused tmux ("it just came up when I gave focus to tmux") and the launcher started that same second.

Guess (the reporter's, unchecked): the shell or Claude's terminal teardown waits for a terminal reply (a focus or terminal-query response) that only comes while an attached client has focus; meanwhile keystrokes echo as ^M (the tty is still in raw/no-icrnl mode).

Second problem: no incident during the 6.5 h. The aide's own waiter made `waiter_open` true; between its waits status showed `waiter_open:false` and no incident (checked 2:30, 4:00, 5:30 PM ET). The no-waiter incident should count only the orchestrator's waiter, not any external session's.

Code: the relaunch's send-keys in `crates/bridle-daemon/src/orchestrator.rs` (and `crates/bridle/src/session.rs`); the waiter tracking and its 15-minute incident in `crates/bridle-daemon/src/server.rs`.

Evidence on vesta: /tmp/orch-pane-2026-10-12-1300.txt (pane capture), ~/.bridle/orchestrator.exits, handovers h-0034 and h-0035 on meta-notes. Related: br-5j35 (restart from inside a session never relaunches), rule tmux-shell-first.

Verify: a relaunch into a detached or unfocused tmux pane starts the orchestrator (e.g. reset the pane's tty, `stty sane`, or `tmux respawn-pane`, before typing; or check that the launcher is running and retry another way, raising an incident if it isn't); a test covers the relaunch path with no attached client where it can. With only an aide waiter open, the no-orchestrator-waiter incident still fires after 15 minutes.
