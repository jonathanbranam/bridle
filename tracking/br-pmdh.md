+++
id = "br-pmdh"
title = "New-machine setup: log in to Claude Code before the first bridle serve"
kind = "docs"
state = "built"
created_at = "2026-10-14T18:14:53.591Z"
updated_at = "2026-10-15T03:15:02.586791Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:pmdh",
]
branch = "bridle/pmdh"
commit = "2c4b47e2204f40e4b761245412ba54070b2a75e5"
summary = """Docs only. `docs/context/add-a-machine.md` gets a new section 1 item 3, "Log in to Claude Code, before any `bridle serve`, unit start or `bridle project start`" (after `gh auth login`; the git identity item moves to 4). `docs/context/windows-wsl2-host.md` gets the same login paragraph right after `claude --version` in step 6, before the daemon's units are installed. Both give the one-line why: a daemon started before the login reports "claude is not logged in in this daemon's session" and its agents crash until it is restarted after a login (br-myw9). CHANGELOG Unreleased has a Docs line. No behaviour change; no migration."""
ticket = "pmdh"
+++

docs/tickets/open/new-machine-setup-log-in-to-claude-code-before-the-first-bri-pmdh.md

## Thread

### note · agent:pmdh · 2026-10-15T03:02:49.340Z
done: br-pmdh docs: add-a-machine.md section 1 item 3 and the WSL2 guide's step 6 now say to log in to Claude Code before any bridle serve or unit start, with the br-myw9 why; CHANGELOG Unreleased Docs line; summary on the ticket. just check exit 0 (1701 run, 1701 passed, 6 skipped); main merged (no-op, already up to date); commit 2c4b47e2204f40e4b761245412ba54070b2a75e5

### note · agent:manager-2 · 2026-10-15T03:15:02.586Z
built: 2c4b47e2204f40e4b761245412ba54070b2a75e5 (branch bridle/pmdh)
