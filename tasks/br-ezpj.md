+++
id = "br-ezpj"
title = "Run the mail bridge as a service: 'bridle mail install' for launchd (macOS) and systemd (Linux)"
kind = "feature"
state = "planned"
created_at = "2026-10-08T00:43:32.504Z"
updated_at = "2026-10-08T01:01:53.626850Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "ezpj"
+++

Ticket: docs/tickets/open/run-the-mail-bridge-as-a-service-bridle-mail-install-for-lau-ezpj.md (read it; the human's ask, verbatim: "let's implement the launchd service and linux support for the mail daemon tonight"). The human wants this built tonight.
Goal: `bridle mail install [--project P] [--force]` writes a service definition that runs `bridle mail run` for one project at login/boot and restarts it on a crash: a LaunchAgent plist on macOS, a systemd user unit on Linux.
Model it on the two existing installers; read them first and copy their shape, helpers and tests, do not invent a third style:
- crates/bridle/src/launchd.rs (`install`, `render`, `label`, `commands`, `project_name`, `xml_escape`; tests at the bottom) and `bridle gateway install` (crates/bridle/src/gateway.rs, its launchd half in the same area, and the gateway install tests);
- crates/bridle/src/systemd.rs (`check_os`, the user unit text, the linger hint).
Behaviour:
1. The service runs `bridle mail run` for the project, with BRIDLE_AS=mail (the principal `external:mail`; its token comes from ~/.bridle/credentials.toml as `bridle mail run` already reads it), in the project's workspace or with --project, and with HOME set so the standard AWS chain finds ~/.aws/credentials. One service per project (the bridge serves one project): label/unit name includes the project, as launchd install does.
2. Logs to a file under ~/.bridle/ named per project (same convention as the gateway's), stdout and stderr both.
3. Keep-alive: restart on crash (launchd KeepAlive / RunAtLoad; systemd Restart=on-failure, WantedBy=default.target). Same OS detection: launchd on macOS, systemd on Linux; refuse clearly on anything else.
4. Like the others: print the load/enable commands (`launchctl bootstrap ...` / `systemctl --user enable --now ...` and, on Linux, `sudo loginctl enable-linger <user>`); never run them. Refuse to overwrite an existing file without --force. Refuse with a clear message naming the fix when ~/.bridle/config.toml has no `[mail]` section or the `mail` token is missing (check how `bridle mail run` finds both: crates/bridle/src/commands/orchestrator.rs `mail_run`, and docs/design/mail.md).
5. `bridle mail uninstall` only if the gateway and daemon installers have an uninstall of the same shape (they do: launchd uninstall); mirror whichever they have, no more.
Docs: docs/design/mail.md ("how it runs": replace "started by hand or by a service the human sets up" with the command), docs/design/cli.md, CHANGELOG.
Other tasks touch the same files: br-843g (mail routing) and br-gdyy (mail run logging) edit orchestrator.rs and bridle-mail. This task goes FIRST; keep your edits to the install code in new/adjacent functions and to the cli.rs `mail` subcommand enum so merges stay trivial.
Out of scope: the logging fix (br-gdyy), mail routing (br-843g), starting or loading the service, Windows.
Migration: none (new command; existing projects keep running the bridge by hand until the human installs it).
Acceptance: just check passes; tests mirror launchd's and systemd's: valid plist and unit text written (contains the project, BRIDLE_AS=mail, HOME, the log path, keep-alive), overwrite refused without --force and allowed with it, missing [mail] and missing token refused with a message naming the fix. Model: Sonnet.

## Thread

### note · external:aide · 2026-10-08T00:43:32.799Z
From the human, via aide (~9:05 PM ET): "let's implement the launchd service and linux support for the mail daemon tonight. No reason to wait IMO." Filed ticket ezpj / br-ezpj ('bridle mail install', launchd + systemd, modelled on gateway install). Please ready it and get it built tonight.

### note · external:orchestrator · 2026-10-08T00:43:44.144Z
Readied by orchestrator. The human, ~9:05 PM ET via aide: "let's implement the launchd service and linux support for the mail daemon tonight. No reason to wait IMO." Takes the next free worker slot tonight (two workers max).

### note · external:orchestrator · 2026-10-08T00:43:44.193Z
From orchestrator: br-ezpj is ready and the human wants it built tonight. Plan it now (Sonnet, modelled on gateway install) and put it at the front of the queue, ahead of br-843g and br-gdyy.

### note · external:orchestrator · 2026-10-08T00:43:44.300Z
From orchestrator: br-ezpj ('bridle mail install', launchd + systemd) is the human's tonight priority. Give it the next free worker slot once pm-1 plans it; two workers max.

### note · agent:manager-2 · 2026-10-08T00:52:53.949Z
br-ezpj is ready and a slot is free, but the daemon refuses the spawn: load 7.7 per core (threshold 2.5), mostly my br-4yc8 land check. Do you want me to use --ignore-budget for this one (the human's tonight priority), or wait for load to fall? Recommend: wait a few minutes, the land should finish soon; override only if you say so.

### note · external:aide · 2026-10-08T01:01:53.626Z
From the human, via aide (~9:40 PM ET), after the machine-level bridge question went to kuw2: "yes, finish ezpj for now, no problem".
