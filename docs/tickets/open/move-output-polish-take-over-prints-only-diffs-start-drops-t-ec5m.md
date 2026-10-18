---
id: br-ec5m
title: "Move output polish: take-over prints only diffs, start drops the contradictory mail hint, runbook names a real mail status command (from jqf9)"
kind: fix
created: 2026-10-18
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-jqf9]
---

## The ask

Split from br-jqf9 (items 3, 4, 5 of its ask; read that ticket for the trial 2 context).

3. `project take-over` prints the whole `projects sync` table (every machine, the `[mail]` and `[projects]` rows) when one entry changed, and the `[mail] allow` row runs its columns together. Print only the differences.
4. `project start`'s mail bridge output contradicts itself: the install helper says "nothing was started; to load it: launchctl bootstrap ..." and then "mail bridge unit ... enabled and started". Drop the "nothing was started" hint when start goes on to start the unit.
5. docs/context/move-a-project.md step 5.5 names `bridle mail status`, which doesn't exist. Name a command that does (`launchctl list | grep bridle.mail` on macOS, `systemctl --user is-active` on Linux).

Model: Sonnet. Files: crates/bridle (grep take-over, projects sync, the mail unit install helper), docs/context/move-a-project.md, CHANGELOG. Tests: a take-over with one changed entry prints only that entry; start prints no "nothing was started" hint when it starts the unit. Acceptance: just check. Out of scope: the drain and migration fixes (br-jqf9 items 1, 2) and the preflight bug (br-eavq).
