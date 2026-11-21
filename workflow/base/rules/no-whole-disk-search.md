---
id: no-whole-disk-search
severity: must
roles: [orchestrator, project-manager, manager, worker, reviewer, document-reviewer, advisor, aide, prototyper, designer]
locked: true
---
Never search the whole disk or the home folder: no `find /`, `find ~`, `find $HOME`,
`find /Users/<name>`, `mdfind`, `locate`, or `du`, `ls -R`, `grep -r` and `rg` rooted there.
Search your worktree, the workspace, /tmp or /Volumes/Data, or look in a known place.

- macOS blames an agent's commands on the daemon that spawned it, so a walk of `/` or `~`
  raises privacy prompts (Desktop, Documents, Contacts, Photos) in bridle's name.
- For Playwright: outside a Node project use the global `playwright-cli` (0.1.25, installed);
  browsers are in `~/Library/Caches/ms-playwright`. Don't hunt for `playwright-core`.
- The `bridle search-guard` PreToolUse hook refuses these, also inside compound commands.

Why: 2026-11-21 (ticket rk4b): two workers ran `find / -name ...` looking for Playwright and
the human got a burst of privacy prompts naming bridle. This is a new base rule only because
the ticket asks for one.
