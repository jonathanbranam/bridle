---
id: mtdg
title: daemon restart --upgrade builds the project's repo instead of bridle's
kind: bug
opened: 2026-10-06
repos: [meta-notes]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask


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
