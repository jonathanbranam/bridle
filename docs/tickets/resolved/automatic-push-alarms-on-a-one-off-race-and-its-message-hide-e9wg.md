---
id: br-e9wg
title: Automatic push alarms on a one-off race, and its message hides git's reason
kind: bug
created: 2026-10-11
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-10-17T11:37:14Z
---

## The ask

Twice on 2026-10-11 (13:58Z, 14:28Z) the orchestrator was woken by 'The automatic push of main to origin failed (bridle): To github.com:jonathanbranam/bridle.git. ... find out why.' Both times main was already in sync with origin when checked: the automatic push had collided with another push (origin's reflog shows pushes seconds apart: 14:28:15Z and 14:28:41Z), and a later tick or the other pusher put everything on origin. Nothing was lost; each wake was a false alarm.

Two flaws in crates/bridle-daemon/src/autopush.rs:
1. first_line() takes git's first non-empty stderr line, which for a rejected push is 'To <url>', not the reason ('! [rejected] ... (fetch first)', 'error: ...'). Pick the '! ' or 'error:' line (fall back to the first line). The daemon log doesn't record the reason either.
2. tick() alarms on the first failure. Alarm only when the push still fails on the next tick (the branch is still ahead of origin), so a one-off race between pushers stays quiet.

Acceptance: a test where the first push fails and the next succeeds sends no alarm; a rejection's message names git's reason line; just check passes. Model: Haiku. Not critical: no work is lost, only false wakes.
