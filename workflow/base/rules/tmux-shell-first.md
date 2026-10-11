---
id: tmux-shell-first
severity: must
roles: [orchestrator, aide, advisor]
---
When you open a tmux window for a session, open a shell window first, then type the command into
it: `tmux new-window -n <name>`, then `tmux send-keys -t <name> '<command>' Enter`. Never
`tmux new-window <command>`.

- A window started with a command is bound to that process: when the session exits (a handover, a
  crash), the window closes with it, and a relaunch has no pane to type into ("pane %22 is gone").
- A shell window outlives the session, so a restart can send-keys into the same pane.

Why: the human, 2026-10-11 ~11:50 AM ET (via aide, verbatim): "this is likely because
orchestrator launched that session and keeps using a tmux command that binds the pane to the
claude process". advisor/wsl2 handed over at 201k and its window closed with it, so the relaunch
failed. Recorded on ticket 5j35.
