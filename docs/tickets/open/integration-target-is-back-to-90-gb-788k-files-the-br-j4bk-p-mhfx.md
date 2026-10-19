---
id: br-mhfx
title: "integration/target is back to 90 GB (788K files): the br-j4bk prune isn't keeping it down, and dead worktrees keep their target/"
kind: bug
created: 2026-10-19
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

[[integration-target-grows-without-limit-833k-files-152-gb-pru-j4bk|j4bk]] landed a prune for
`integration/target`, but on 2026-10-18 ~8:40 PM ET it was 90 GB again, with ~788K files in `debug/deps`
(j4bk found 833K when it was filed). Find out why the prune isn't running or isn't keeping up, and fix
it so the folder stays near a clean build's size (the clone's `target/debug/deps` has ~12K files).

Also handle worktrees whose agent is gone. On dalek, five were left behind with their `target/`:
`dev-site-design` (78 GB, 547K files in deps), `w79c3` (11 GB), `wait-cmd` (3.4 GB), `rci` and `w8swr`.
All were merged or superseded and none had an agent, so `bridle agent rm` couldn't remove them. A
worktree whose branch has landed and whose agent is gone should be removed, or at least its `target/`.
The worker can pick which, with a log line or event for each removal.

## Why it matters

Each new executable in these folders gets the macOS syspolicyd scan on its first exec
([[the-macos-verifying-popup-still-flashes-during-bridle-builds-p29s|p29s]]). On 2026-10-18 evening:
syspolicyd at 40-250% CPU; 32 test binaries in the br-65qh landing check each waited ~45 s on their
first `--list`; every `just check` took ~2 hours (orchestrator m-17040), one br-65qh land was killed
after 2 hours; the self-upgrade to ac004852 was refused 3x on its self-check (br-mp9d); the human's
track-web tests crawled; the popup flashed in bursts and ate the human's keystrokes.

## The human

2026-10-18 ~9:00 PM ET (to aide), approving aide's plan (cleanup, prune, fewer checks at once, this
ticket; not the Developer Tools change), verbatim:

> Yes I agree with your plan; iTerm2 has been listed in developer permissions for a long time and through a restart; it won't fix anything.
