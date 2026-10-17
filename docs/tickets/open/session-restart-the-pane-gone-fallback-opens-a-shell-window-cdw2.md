---
id: br-cdw2
title: "session restart: the pane-gone fallback opens a shell window first (tmux-shell-first), follow-up to 5j35"
kind: fix
created: 2026-10-17
created_by: agent:manager-2
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

br-5j35 landed (bc988fc3) before the orchestrator's review arrived. Remaining: in crates/bridle/src/session.rs the pane-gone fallback must open a shell window (tmux new-window -d -P) then send-keys the relaunch command into it (rule tmux-shell-first). The fix exists as commit fbd96e5a (16 lines in session.rs, on top of bc988fc3); cherry-pick it onto main, add a CHANGELOG note only if the 5j35 line needs it, just check. Related: br-5j35.
