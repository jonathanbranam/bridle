---
id: rjd5
title: "Commands to set up and sync a project everywhere: sync all seven places per machine, and add a project to both machines and mail"
kind: feature
opened: 2026-10-09
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [sk7p, gdf3]
tasks: []
---

## The ask

The human, verbatim (2026-10-08 ~7:10 PM ET, relayed by the bridle-ui aide, m-7237):

> we've just got to build commands to do this project setup stuff; wayyy too much to keep handling by hand. How many places list projects and ports and tokens... ? We had a command to copy tokens; basically, ideally, I have a command to sync all of this and then a command to add a project that just handles adding to both machines, mail, etc. It could take params for that or ask for confirmation for all the defaults:
>
> Create aide token: [Y/n]
> Add to mail: [Y/n]
>
> Whatever. And can pass -y to accept all defaults.

Then, ~7:20 PM ET (m-7248), after the bridle-ui aide listed the seven places below:

> yes, sync and set up all of those things; everything; it can confirm for any changes during sync with the user, or same accept with -y

## Context (from the bridle-ui aide)

That evening the orchestrator restarted dalek's gateway with `BRIDLE_AS=orchestrator` in its env (every project unreachable 10:37 AM-6:46 PM), and setting up the NUC projects for the web UI (ui-9hq8, br-xg47) meant hand edits.

Where each project's setup lives today, on each machine:

1. `~/.bridle/config.toml` `[projects]` (machine, port): meant to be identical on every box, but drifted (dalek lacked meta-notes-ui until 6:48 PM; the NUC lacks bridle-ui and track-web).
2. The mail section's projects list.
3. `credentials.toml` `[aide]`, `[orchestrator]`, `[advisor]` per project, plus `[<role>.<machine>]` and `[human.<machine>]` on the other machine.
4. `[peer]` tokens per project pair (gdf3).
5. The workspace's `.bridle/tokens/human`.
6. `~/.bridle/daemons/<project>.json`.
7. A launchd plist or systemd unit per daemon.

## The ask

- **`sync`**: brings all seven into agreement on every machine. It shows each change and asks; `-y` accepts all.
- **`add a project`**: does all seven for a new project on both machines (mail included), asking `[Y/n]` per default (create aide token, add to mail, ...) or taking params; `-y` accepts all defaults.

Related: [[pair-machines-token-setup-over-ssh-sk7p|sk7p]] (`bridle token pair`, planned), [[peer-token-setup-guidance-a-token-per-receiving-project-per-gdf3|gdf3]], [[docs/context/adding-a-project|adding a project]].
