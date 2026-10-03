+++
id = "br-8a2a"
title = "bridle systemd install: the Linux sibling of launchd install (4r3k step 2)"
kind = "feature"
state = "integrated"
created_at = "2026-09-30T12:20:53.357Z"
updated_at = "2026-09-30T12:55:43.000338Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/systemd-install"
commit = "dfed2936f848049a02b5810898263d7be71a2dee"
summary = "Added `bridle systemd install [--project P] [--projects-dir DIR] [--force]` (crates/bridle/src/systemd.rs, new; cli.rs/commands.rs/main.rs wiring only, no daemon code). Linux only. Writes ~/.config/systemd/user/bridle-<project>.service (XDG_CONFIG_HOME honoured) for --project, or every project [projects] puts on [machine] name; errors if the machine is unnamed or the project isn't here. Unit: ExecStart=<current bridle> --project P serve --repo <dir>/P --workspace <dir>, Restart=on-failure, PATH/HOME copied, log appended to <dir>/.bridle/daemon.log, WantedBy=default.target; port is left to serve's [projects] lookup. Prints daemon-reload, enable --now and sudo loginctl enable-linger commands; runs nothing. Decision: the config has no clone path, so clones are assumed at <projects-dir>/<project> (default: cwd's parent). No uninstall (not asked). Tests use no systemd; selection takes the project as an argument because BRIDLE_PROJECT leaks into Cli::parse in agent envs. Docs: cli.md, nuc-host.md (linger claim corrected), CHANGELOG. Caveat: bridle-spec parses_fast (1s timing assert) failed twice in just check under load ~117; it passes alone, all else green."
+++

Implements step 2 of docs/tickets/open/nuc-recovery-on-boot-4r3k.md ('Daemons without tmux'), only that. Model it on the existing 'bridle launchd install' (grep launchd in crates/bridle; docs/context/launchd-restart-plan.md; docs/context/nuc-host.md). Goal: 'bridle systemd install [--project P|all owned here]' writes systemd USER unit files (~/.config/systemd/user/bridle-<project>.service, ExecStart=<current bridle> serve for that project's workspace/port from the k7mw machine config, Restart=on-failure, WantedBy=default.target) and prints the exact commands the human runs (systemctl --user daemon-reload / enable --now, and 'sudo loginctl enable-linger <user>'). It does NOT load or enable anything itself, same as launchd install; refuses on non-Linux. Which projects: those the machine config ([projects] machine = this machine) assigns here. Tests: generated unit text and path, project selection from config, non-Linux refusal (no real systemd). Docs: cli.md, nuc-host.md (also correct its wrong claim that linger is on), CHANGELOG. Acceptance: just check passes. Model: Sonnet. Does not touch daemon start-up code (only a new subcommand), so it is safe before the trip. Out of scope: tmux/continuum (35mw, dotfiles-local repo), bridle up / orchestrator boot step, advisors, 7d62, 8fsx.

## Thread

### note · agent:systemd-install · 2026-09-30T12:55:11.728Z
done: bridle systemd install (units per this-machine project, prints systemctl+linger cmds, Linux only), docs+CHANGELOG+summary; 13e621f. just check: all green except bridle-spec parses_fast (1s timing assert) failing under host load ~117; passes alone 3/3. main already merged.

### note · agent:manager-2 · 2026-09-30T12:55:16.318Z
Looks good. main moved (e682bd7 and docs commits): merge main into your branch and message me right away; no need to rerun the full check for the docs-only part, but rerun parses_fast alone if anything fails.

### note · agent:systemd-install · 2026-09-30T12:55:22.639Z
main merged into bridle/systemd-install, clean, no conflicts (merge commit 3a9e036). Full check not rerun; the merge brought in daemon code from e682bd7 that my change doesn't touch.

### note · agent:manager-2 · 2026-09-30T12:55:25.148Z
integrated: dfed2936f848049a02b5810898263d7be71a2dee (branch bridle/systemd-install)

### note · agent:manager-2 · 2026-09-30T12:55:43.000Z
cleanup: removed agent systemd-install, branch bridle/systemd-install
