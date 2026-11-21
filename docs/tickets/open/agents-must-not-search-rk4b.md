---
id: br-rk4b
title: "Agents must not search the whole disk or home folder: a worker's find / set off macOS privacy prompts in bridle's name"
kind: bug
created: 2026-11-21
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## What happened

2026-11-21 14:03 ET, bridle-ui worker picker-a (a-m9beg, worktree wt/picker-a) ran, looking for Playwright:

    find / -name "playwright-core" -maxdepth 6 -type d 2>/dev/null | head -3

macOS attributes an agent's commands to the daemon that spawned it (responsible process /Users/jbranam/.cargo/bin/bridle, the bridle-ui daemon pid 17407), so walking / touched protected folders and the human got a burst of privacy prompts naming bridle: Desktop, Documents, Downloads, Network Volumes, Contacts, Calendar, Reminders, Photos, iCloud Drive (tccd log, 14:03:33-14:03:53 ET). The human denied most of them.

The human, 2026-11-21, to aide (verbatim): "Bridle just asked for permissions to read from about 5-6 locations on my computer as well as my contacts. Why did this happen?" and "I denied most of the requests. I think it was Desktop, Documents and a few others".

## The ask

Stop agents from scanning the disk or the home folder:

- A rule (base) telling agents never to run `find /`, `find ~` (or `$HOME`, `/Users/...`), `mdfind` over home, or similar whole-disk searches; look in known locations instead (for Playwright: `~/Library/Caches/ms-playwright`, the project's `node_modules`, `npx playwright --version`).
- Backed by a PreToolUse check (like `kill-guard`) that refuses `find` with a root of `/`, `~`, `$HOME` or `/Users/<user>` from agents.
- Where agents need Playwright (br-4bhp, the UI check), give them its path so nobody goes looking.

Denials are harmless to bridle: nothing it runs needs those folders (workspaces are on /Volumes/Data, a local disk). Because the decision is recorded against the bridle binary's path, every agent on this machine is now denied those folders, which is the right default.
