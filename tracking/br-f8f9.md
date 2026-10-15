+++
id = "br-f8f9"
title = "The NUC recovers everything on boot (4r3k)"
kind = "feature"
state = "claimed"
created_at = "2026-09-30T12:20:01.124Z"
updated_at = "2026-10-15T02:46:13.510808Z"
created_by = "external:advisor"
watchers = [
    "external:advisor",
    "external:advisor/product-manager",
    "agent:f8f9",
]
size = "M"
summary = "Slice 1 only. New `bridle up` (crates/bridle/src/up.rs, `UpArgs` in cli.rs): finds the pane tagged `@bridle=orchestrator`, else creates a detached tmux session `bridle` (no server) or a window (server running) running `bridle session orchestrator` and tags its pane; second run starts nothing. `~/.bridle/no-autostart` starts nothing, exit 0. `--install [--force]` writes a launchd plist / systemd oneshot user unit (opt-in, prints load + enable-linger commands, runs nothing). `--print-tmux-hooks` prints the two tmux-resurrect hooks (tags saved to ~/.bridle/pane-tags); no tmux config touched. Tests use a fake tmux script (reuse, create+tag, second-run no-op, window when server has no tag, kill switch). Docs: cli.md, add-a-machine.md, windows-wsl2-host.md, nuc-host.md (linger), CHANGELOG. Not built (need follow-up tickets, PdM readies): per-project [boot] sessions, startup cap/stagger beyond the kill switch, orchestrator starting aides. Not installed into `project start` (separate `up --install`). The reboot drill is the human's."
ticket = "4r3k"
+++

docs/tickets/open/nuc-recovery-on-boot-4r3k.md

## Thread

### note · system · 2026-10-05T15:24:32.089Z
open 4h, never planned: back to pending. Ready it again once someone will plan it.

### note · external:advisor/product-manager · 2026-10-09T11:04:36.626Z
watching the task

### note · agent:pm-1 · 2026-10-15T01:02:51.009Z
Brief (pm-1). Model: Sonnet. Size: M, FIRST SLICE ONLY (the human: "pretty urgent"; also "don't over-architect"). Build item 1 of "Scope now": one orchestrator per machine at boot. Items 2 (per-project [boot] sessions), the startup cap/stagger/kill switch beyond the single no-autostart file below, and 4 (orchestrator starts aides) are NOT in this ticket: tell the manager they need follow-up tickets (new tickets need the PdM to ready them) and stop.

Slice 1:
- A command (e.g. `bridle up`, name per cli.md conventions; check `git grep -n "session orchestrator" crates docs`) that, once tmux is up, finds the pane tagged `@bridle orchestrator`; if none, creates a tmux session or window and tags a pane; then runs `bridle session orchestrator` there. Idempotent: running it twice never starts a second orchestrator (one per machine, rule ma8e). No tmux server running: start a detached one.
- Pane tags survive tmux-resurrect: provide the two hook snippets (`@resurrect-hook-post-save-all` saves the tags to a file under ~/.bridle, `@resurrect-hook-post-restore-all` puts them back) as a documented snippet and/or `bridle up --print-tmux-hooks`. DO NOT edit the human's dotfiles-local project or any tmux config on their machines (rule existing-projects): ship the command and the snippet; the human or aide applies it.
- The kill switch: if `~/.bridle/no-autostart` exists, the command starts nothing, says so, exits 0.
- A boot hook: a systemd user unit (Linux/WSL; `bridle systemd install` sibling if it exists) and a launchd entry that runs the command after tmux is up; installable by `bridle project start` or a separate install command, whichever matches how the daemon units are installed today (find in project_start.rs / mail_install.rs). Linger must be on for no-login boot (docs/context/nuc-host.md says it is; check/enable with `loginctl enable-linger`, document, do not run).
Docs: docs/design/cli.md, docs/context/add-a-machine.md and windows-wsl2-host.md (boot step + drill), docs/context/nuc-host.md linger note, CHANGELOG.md. No migration (opt-in install). Tests: fake tmux script: tag found -> reuse; none -> create+tag; second run no-op; kill switch. Acceptance: just check. The drill (reboot remus) is the human's, not a worker's.
