---
id: yhe7
title: Self-upgrade checks a new binary against the clone's newer config, so a config key landing during a build fails the upgrade
kind: bug
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask



Found 2026-10-03 ~13:45 UTC by the orchestrator (an `upgrade_failed` wake). The daemon's
self-upgrade built `1afcb9d` (green CI), then refused to restart into it: the new binary's
self-check failed to parse the clone's `.bridle/config.toml`, line 22
`warm_build = "cargo build --workspace --all-targets"`, "unknown field `warm_build`, expected
`check`". `dd7400b` (br-14cd) landed during the build and added both the `[integration]
warm_build` key and this repo's setting of it. The self-check read the clone's config at the newer
commit, against a binary from the older one.

The refusal was right: the old daemon kept running. But two things follow:

- The upgrade can't succeed until a later commit (one that knows the key) builds; it should
  retry on the next green `main`, which costs one more build.
- Until then the running binary also can't parse the clone's config. If the daemon dies now,
  `bridle daemon serve` with the installed binary fails at start-up. Any land that adds a config
  key together with setting it in `.bridle/config.toml` opens this window, because config
  structs deny unknown fields.

Wanted (the worker picks): the self-check reads the config as of the built commit (e.g. `git
show <commit>:.bridle/config.toml`), or the upgrade builds the clone's HEAD when it has moved, or
a newly added key lands in code first and in `.bridle/config.toml` in a later commit (a rule for
workers). Worst if we don't: an occasional wasted build, and a rare start-up failure after a
crash in that window.
