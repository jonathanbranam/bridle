+++
id = "br-eavq"
title = "project preflight falsely fails claude, gh and tmux on a macOS target: ssh's bare PATH"
kind = "bug"
state = "claimed"
created_at = "2026-10-18T23:31:21.326Z"
updated_at = "2026-10-19T11:15:27.509902Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:weavq",
]
ticket = "eavq"
+++

docs/tickets/open/project-preflight-falsely-fails-claude-gh-and-tmux-on-a-maco-eavq.md

## Thread

### note · external:advisor/product-manager · 2026-10-18T23:31:36.824Z
Readied by advisor (product-manager): a bug from kx5u trial 2 with one clear fix.

### note · agent:pm-1 · 2026-10-18T23:32:12.356Z
Brief (pm-1). Model: Sonnet. Build the ticket's ask and Proposal as written: one shared PATH line (VERSION_SNIPPET's, extended with /usr/local/bin and /opt/homebrew/bin) prefixed to every probe in crates/bridle/src/preflight_remote.rs; on a Darwin NEW over ssh, a logged-out `claude auth status` is a warning "unverified over ssh" that does not fail the preflight. Tests with the fake Remote for both. Update docs/context/move-a-project.md step 1 if the output changes. Acceptance: just check. Out of scope: other probes' logic, the doctor.rs keychain limit (ticket nrbf). Priority high: it blocks project moves to a macOS machine (kx5u).
