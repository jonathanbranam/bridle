+++
id = "br-rk4b"
title = "Agents must not search the whole disk or home folder: a worker's find / set off macOS privacy prompts in bridle's name"
kind = "bug"
state = "integrated"
created_at = "2026-11-21T19:06:42.154Z"
updated_at = "2026-11-21T19:42:13.851362Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:wrk4b",
]
priority = "high"
priority_at = "2026-11-21T19:16:24.743844Z"
branch = "bridle/wrk4b"
commit = "152a464715dfdecb89c50acbd2d9a577703685de"
summary = "Added 'bridle search-guard' (crates/bridle/src/search_guard.rs, wired like kill-guard, in base PreToolUse.json so layer hooks pass it at every spawn): refuses find/fd/du/rg, ls -R, grep -r rooted at /, ~, $HOME, /Users, /Users/<name>, and locate/mdfind (mdfind allowed with a narrow -onlyin), also in compound commands. New base rule no-whole-disk-search (new only because the ticket asks); UI verification rule and dev-site.md say use global playwright-cli 0.1.25 outside Node projects. Docs: cli.md, workflow-layers, coordination. Existing projects get the guard on next spawn."
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

### note · external:aide · 2026-11-21T19:10:50.883Z
aide: the human, 2026-11-21 (verbatim): "if you aren't using node, the CLI is playwright-cli; only if you are in a node project and install it there is the command playwright." So the instruction for workers: outside a Node project use the global playwright-cli (no install, no library hunting); 'playwright' exists only as a Node project's own dev dependency (npx playwright).

### note · external:aide · 2026-11-21T19:11:40.353Z
aide: the human asked aide to read playwright.dev/docs/intro and playwright.dev/agent-cli/introduction (2026-11-21). Findings: (1) docs/intro is Playwright Test, a per-project Node test runner (npm init playwright@latest, then npx playwright test); that is the only place a 'playwright' command exists. (2) The agent CLI is a separate global package, npm install -g \@playwright/cli, command playwright-cli, 'designed for coding agents': a persistent browser daemon, headless by default, each command prints URL, title and an accessibility snapshot with element refs (e21) for the next command; open/goto/click/fill/type/press/snapshot/screenshot/console/requests; named sessions with -s=<name>; installable agent skills; 'install-browser' for browsers. Its own comparison: the CLI is best for coding agents like Claude Code (lower token cost) and MCP for specialised agentic loops. So: workers should drive the UI with playwright-cli (installed on dalek, 0.1.25), not write Node scripts against playwright-core and not add an MCP.

### note · external:aide · 2026-11-21T19:15:50.964Z
aide: second occurrence 14:13 ET: bridle-ui worker tix-click (a-bynm0) ran find / -name headless_shell -maxdepth 8 after its hand-made Node script lost Chromium (the human's upgrade to 0.1.25 moved it to chromium_headless_shell-1255). Prompt 'access data from other apps' (kTCCServiceSystemPolicyAppData); the human denied it. Asked the orchestrator to brief managers now as a stopgap.

### note · agent:pm-1 · 2026-11-21T19:16:24.743Z
priority: normal -> high

### note · agent:wrk4b · 2026-11-21T19:41:36.960Z
built: 63f5baa1 (branch bridle/wrk4b)

### note · agent:wrk4b · 2026-11-21T19:41:37.045Z
done: search-guard hook + rule no-whole-disk-search (new base rule only because ticket asks) + playwright-cli 0.1.25 text in verify rule and dev-site.md; just check exit 0, 2028 tests run, 2028 passed (last full 2012); main merged (up to date); commit 63f5baa1. Existing projects get the guard on next spawn (layer hooks, no edit).

### note · agent:manager-2 · 2026-11-21T19:41:49.743Z
integrated: 152a464715dfdecb89c50acbd2d9a577703685de (branch bridle/wrk4b)

### note · agent:manager-2 · 2026-11-21T19:41:50.091Z
ticket file moved to docs/tickets/resolved/agents-must-not-search-rk4b.md

### note · agent:manager-2 · 2026-11-21T19:42:13.851Z
cleanup: removed agent wrk4b, origin branch bridle/wrk4b, branch bridle/wrk4b
