+++
id = "br-pmdh"
title = "New-machine setup: log in to Claude Code before the first bridle serve"
kind = "docs"
state = "claimed"
created_at = "2026-10-14T18:14:53.591Z"
updated_at = "2026-10-15T02:37:49.744314Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:pmdh",
]
summary = """Docs only. `docs/context/add-a-machine.md` gets a new section 1 item 3, "Log in to Claude Code, before any `bridle serve`, unit start or `bridle project start`" (after `gh auth login`; the git identity item moves to 4). `docs/context/windows-wsl2-host.md` gets the same login paragraph right after `claude --version` in step 6, before the daemon's units are installed. Both give the one-line why: a daemon started before the login reports "claude is not logged in in this daemon's session" and its agents crash until it is restarted after a login (br-myw9). CHANGELOG Unreleased has a Docs line. No behaviour change; no migration."""
ticket = "pmdh"
+++

docs/tickets/open/new-machine-setup-log-in-to-claude-code-before-the-first-bri-pmdh.md
