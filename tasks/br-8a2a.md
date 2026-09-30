+++
id = "br-8a2a"
title = "bridle systemd install: the Linux sibling of launchd install (4r3k step 2)"
kind = "feature"
state = "planned"
created_at = "2026-09-30T12:20:53.357Z"
updated_at = "2026-09-30T12:20:57.327140Z"
+++

Implements step 2 of docs/tickets/open/nuc-recovery-on-boot-4r3k.md ('Daemons without tmux'), only that. Model it on the existing 'bridle launchd install' (grep launchd in crates/bridle; docs/context/launchd-restart-plan.md; docs/context/nuc-host.md). Goal: 'bridle systemd install [--project P|all owned here]' writes systemd USER unit files (~/.config/systemd/user/bridle-<project>.service, ExecStart=<current bridle> serve for that project's workspace/port from the k7mw machine config, Restart=on-failure, WantedBy=default.target) and prints the exact commands the human runs (systemctl --user daemon-reload / enable --now, and 'sudo loginctl enable-linger <user>'). It does NOT load or enable anything itself, same as launchd install; refuses on non-Linux. Which projects: those the machine config ([projects] machine = this machine) assigns here. Tests: generated unit text and path, project selection from config, non-Linux refusal (no real systemd). Docs: cli.md, nuc-host.md (also correct its wrong claim that linger is on), CHANGELOG. Acceptance: just check passes. Model: Sonnet. Does not touch daemon start-up code (only a new subcommand), so it is safe before the trip. Out of scope: tmux/continuum (35mw, dotfiles-local repo), bridle up / orchestrator boot step, advisors, 7d62, 8fsx.
