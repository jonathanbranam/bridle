---
id: br-66qq
title: vesta-home, meta-notes and meta-notes-ui daemons never self-upgraded to 0.7.6 (self_upgrade = release), while notes and dotfiles-local did
kind: bug
created: 2026-10-17
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-fxxz, br-j86p]
---

## The ask

Seen by the aide, 2026-10-17 ~7:45 AM ET, during the 0.7.6 token re-pair (br-fxxz).

On vesta (`~/.bridle/config.toml` has `self_upgrade = "release"`), the notes and dotfiles-local
daemons self-upgraded to 0.7.6 at 11:34Z, but vesta-home, meta-notes and meta-notes-ui stayed on
the old version. No drain holder was in the way, and their `daemon.log` (under
`<workspace>/.bridle/`) showed nothing recent about an upgrade. Consequence: `bridle token pair`
failed two mints (orchestrator -> vesta/meta-notes and vesta/meta-notes-ui, "only a visitor
(--machine) has one") until the human restarted the three daemons by hand at 7:52 AM ET.

Find why those three daemons skipped the release upgrade (did their check run at all? is the
check per daemon, and did it see 0.7.6?), and fix it so every `release` daemon on a machine
upgrades, or logs why it didn't.
