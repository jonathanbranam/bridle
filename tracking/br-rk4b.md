+++
id = "br-rk4b"
title = "Agents must not search the whole disk or home folder: a worker's find / set off macOS privacy prompts in bridle's name"
kind = "bug"
state = "open"
created_at = "2026-11-21T19:06:42.154Z"
updated_at = "2026-11-21T19:09:37.149132Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "rk4b"
+++

docs/tickets/open/agents-must-not-search-rk4b.md

## Thread

### note · external:aide · 2026-11-21T19:06:51.674Z
Readied by aide (bug, self-evident) on the human's report.

### note · external:aide · 2026-11-21T19:07:00.114Z
readied by external:aide

### note · agent:pm-1 · 2026-11-21T19:07:43.285Z
pm-1 brief: model Sonnet.
- Follow the human's 2026-11-17 rule: edit an existing rule, add no new one. Preferred: broaden `workflow/base/rules/no-kill-by-name.md`? No: it is about kills. Pick the closest existing base rule about what shell commands agents may run (look in `workflow/base/rules/`); if none fits, a new base rule is acceptable here ONLY because the ticket asks for it, and say so in one line.
- The guard: copy the shape of `bridle kill-guard` (`crates/bridle/src/commands/hook.rs`, wiring in `crates/bridle-daemon/src/config.rs` and `crates/bridle/src/project.rs`, listed in `docs/design/workflow-layers.md` and `docs/design/cli.md`). Refuse `find` and `mdfind` (and `fd`, `locate`, `du`, `ls -R`, `grep -r`/`rg` with a root path that is `/`, `~`, `$HOME` or `/Users/<name>`), also inside compound commands, with a message that says where to look instead. Allow a root under the workspace, /tmp, /Volumes/Data, and any path under the project's worktrees. Keep the pattern list small (kiss); the common case is `find /` and `find ~`.
- Tell agents where Playwright is (`~/Library/Caches/ms-playwright`, project `node_modules`, `npx playwright --version`) in the existing UI verification text (`verify-ui-in-a-browser` rule or `docs/design/dev-site.md`); br-4bhp is the ticket that gives workers the browser, do not wait for it.
- Migration: the guard is a hook in the generated agent settings; existing projects get it on the next spawn with no edit. Say so in the done message.
- Acceptance: `just check`; tests for the guard (refuses `find / -name x`, `find ~ ...`, `cd /tmp && find / ...`; allows `find . -name x`, `find /tmp/x`). Out of scope: macOS TCC/privacy settings, per-agent sandboxes.

### note · external:aide · 2026-11-21T19:08:22.815Z
aide: the human, 2026-11-21, after confirming 'playwright-cli --version' -> 0.1.13: "yeah, so WTF". Context for the fix: playwright-cli 0.1.13 is installed globally on dalek with Chromium in ~/Library/Caches/ms-playwright, and bridle's CLAUDE.md names it, but bridle-ui's CLAUDE.md, .bridle and docs never mention playwright-cli. picker-a went looking for the playwright-core library (to script it from Node) instead. The fix must put 'use playwright-cli; it is installed' where every project's workers see it (base rule verify-ui-in-a-browser / worker prime), not only in bridle's CLAUDE.md.

### note · external:aide · 2026-11-21T19:09:37.149Z
aide: the human upgraded the global @playwright/cli to 0.1.25 (2026-11-21). Name that version, not 0.1.13, in whatever the fix tells workers; it may need a newer Chromium in ~/Library/Caches/ms-playwright on first use.
