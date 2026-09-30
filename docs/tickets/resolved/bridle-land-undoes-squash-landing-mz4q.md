---
id: mz4q
title: bridle land merges with --no-ff, undoing squash landing
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [sq4m, tr7k]
---

## What happened

sq4m (3d8f986) made the manager land each task as **one squash commit** on the
integration branch (subject `<task id>: <title>`, the summary as body, `Task:`/`Branch:`
trailers), the human's traceability decision (tr7k, 2026-09-29). P5's `bridle land`
(br-6dd6) came later and merges with `--no-ff`, and `workflow/base/roles/manager.md`
now says so, while `workflow/base/skills/manager/SKILL.md` still says
`git merge --squash`. The first landings after the reboot (00af029 br-d99e, 8e41ef6
br-f671) are plain merge commits.

## The ask

`bridle land <task-id>` lands the task as one squash commit in sq4m's shape, and the
manager role and skill agree. `is_merged`'s `Branch:` trailer check (sq4m) keeps
`task done --branch` and `rm --delete-branch` working. Don't rewrite the two merges
already on `main`.

## Resolution

Already fixed by br-1d3d (8d078f4): `bridle land` squashes to one commit (subject `<id>: <title>`, summary body, `Task:`/`Branch:` trailers, single parent); `land_test` asserts the shape and `is_merged` via the trailer; `manager.md`, the manager skill, `cli.md` and the CHANGELOG agree. Verified 2026-09-30 by br-739a (dropped, no change needed).
