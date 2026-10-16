---
id: br-mtdg
title: Self-upgrade defaults to release; commit mode only for bridle's own repo
kind: bug
created: 2026-10-06
repos: [bridle]
changes: []
specs: []
blocked_by: [br-tvrp]
related: []
---

## The ask

Found by the meta-notes orchestrator on the NUC, 2026-10-06 00:25 UTC.
`bridle daemon restart --upgrade --project meta-notes` reported "building
331ae333d (green CI)", a meta-notes commit, then failed:

```
upgrade_failed: build of 331ae333d failed; the daemon is unchanged:
cargo install --path crates/bridle failed (exit status: 101):
error: `/srv/shared/work/meta-notes-work/.bridle/upgrade-src/crates/bridle`
is not a directory.
```

The upgrade picks "the newest commit on main with green CI" from the
project's own repo and checks it out into `<workspace>/.bridle/upgrade-src`.
For any project but bridle it should use bridle's repo. Harmless (nothing
changed), but it leaves a non-bridle project's daemon with no way to upgrade
itself.

### Scope, agreed with the human 2026-10-16 17:05 ET

The original fix ("for any project but bridle, use bridle's repo") is wrong: a machine that
isn't building bridle shouldn't build it at all. Release upgrades exist now (br-9a7b,
`self_upgrade = "release"`). The human, on why a machine would build bridle: "Why would a machine
without bridle build bridle? If it isn't using releases?" And on the default: "Is self upgrade
release the default? It should be. Then it's true everywhere and we turn it off on the machine
that hosts bridle." Then: "Defaults should be to self upgrade always and through release."

1. **Release is the default.** `[daemon] self_upgrade` defaults to `"release"` (today `Off`,
   `config.rs`). Every daemon on every machine follows bridle's releases unless told otherwise.
2. **The machine that hosts bridle opts out.** dalek sets the machine value in
   `~/.bridle/config.toml` so its other daemons don't install releases over the binary bridle's
   daemon builds from `main`, while bridle's own project keeps commit mode (`self_upgrade = true`
   in `.bridle/config.toml`). Today the machine value wins over a project's (daemon.md, 'Release
   upgrade'), so a machine `false` would also stop bridle's commit mode: the planner settles how
   bridle's commit mode survives (for example, a project's explicit `true` keeps it), and says so
   in daemon.md.
3. **Commit mode outside bridle's repo refuses clearly.** `restart --upgrade` (and the automatic
   tick) in commit mode on a project whose repo has no `crates/bridle` doesn't build: one line,
   "this project's repo isn't bridle; set `[daemon] self_upgrade = \"release\"`", instead of a
   cargo error.

Done when: a fresh daemon with no `self_upgrade` set upgrades from releases; dalek's non-bridle
daemons don't install releases while bridle's daemon still builds `main`; commit mode on a
non-bridle repo refuses with that line; daemon.md and roles-and-config.md show the new default
and precedence.
