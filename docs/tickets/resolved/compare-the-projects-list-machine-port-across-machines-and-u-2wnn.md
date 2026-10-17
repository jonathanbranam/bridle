---
id: br-2wnn
title: "Compare the [projects] list (machine, port) across machines and update every copy to match: a command of its own"
kind: feature
created: 2026-10-12
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-rjd5, br-eurm, br-evy6]
theme: multi-machine
parent: br-kx5u
resolved: 2026-10-17T11:37:10Z
---

## The ask

The human, 2026-10-11 ~8:19 PM ET (to the aide, while discussing eurm), verbatim:

> Also, the list of projects and ports should be sync-able. Separate command, compares them between machines and can update them all to match

So: a command of its own, separate from `bridle token pair`, that compares the `[projects]` list (project, machine, port) in `~/.bridle/config.toml` across every machine in `[machines]` (over SSH, as pair does), shows the differences, and can update every machine's copy to match.

Context: `token pair` (and eurm's fix) reads `[projects]` on the machine it runs on, so the copies must match for it to pair everything. rjd5 found them drifted (dalek lacked meta-notes-ui until 10-09; the NUC lacked bridle-ui and track-web). rjd5 is the wider "set up and sync a project everywhere"; this is the one narrow piece of it. evy6 (one placement record) would later replace the copies.

The human asked for no big changes (eurm, ~8:15 PM ET): keep this small.
