+++
id = "br-eavq"
title = "project preflight falsely fails claude, gh and tmux on a macOS target: ssh's bare PATH"
kind = "bug"
state = "integrated"
created_at = "2026-10-18T23:31:21.326Z"
updated_at = "2026-10-19T11:51:34.122439Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:weavq",
]
branch = "bridle/weavq"
commit = "ec1cec0a18231746bc054bc8d4a185f438effccd"
summary = "Every NEW probe script in preflight_remote.rs now starts with one shared PATH_LINE (~/.local/bin, ~/.cargo/bin, /usr/local/bin, /opt/homebrew/bin; VERSION_SNIPPET extended the same). The claude probe exits 3 when logged out on Darwin; check_new turns that into a pass with 'unverified over ssh: check with claude auth status in a terminal on NEW'. Non-Darwin still FAILs. Two fake-Remote tests; move-a-project.md step 1 and CHANGELOG updated."
ticket = "eavq"
+++

docs/tickets/open/project-preflight-falsely-fails-claude-gh-and-tmux-on-a-maco-eavq.md

## Thread

### note · external:advisor/product-manager · 2026-10-18T23:31:36.824Z
Readied by advisor (product-manager): a bug from kx5u trial 2 with one clear fix.

### note · agent:pm-1 · 2026-10-18T23:32:12.356Z
Brief (pm-1). Model: Sonnet. Build the ticket's ask and Proposal as written: one shared PATH line (VERSION_SNIPPET's, extended with /usr/local/bin and /opt/homebrew/bin) prefixed to every probe in crates/bridle/src/preflight_remote.rs; on a Darwin NEW over ssh, a logged-out `claude auth status` is a warning "unverified over ssh" that does not fail the preflight. Tests with the fake Remote for both. Update docs/context/move-a-project.md step 1 if the output changes. Acceptance: just check. Out of scope: other probes' logic, the doctor.rs keychain limit (ticket nrbf). Priority high: it blocks project moves to a macOS machine (kx5u).

### note · agent:weavq · 2026-10-19T11:33:31.821Z
built: 83d7a543 (branch bridle/weavq)

### note · agent:weavq · 2026-10-19T11:33:31.904Z
done: probes get a full PATH; macOS logged-out claude is 'unverified over ssh' (warning pass); just check exit 0, 1890 tests passed; 83d7a543

### note · agent:manager-2 · 2026-10-19T11:38:11.389Z
integrated: ec1cec0a18231746bc054bc8d4a185f438effccd (branch bridle/weavq)

### note · agent:manager-2 · 2026-10-19T11:38:26.547Z
cleanup: removed agent weavq, branch bridle/weavq

### note · agent:manager-2 · 2026-10-19T11:51:05.738Z
CI is red on main ec1cec0a (this landing), check (ubuntu-latest): https://github.com/jonathanbranam/bridle/actions/runs/39246103379 . weavq: find the failing test (gh run view 39246103379 --log-failed, or reason from the ubuntu environment: no Homebrew dirs, no ~/.local/bin, uname Linux, HOME differences), fix it on a new commit on your branch bridle/weavq after merging main, run just check once, and message me the tip. Likely your new PATH/Darwin tests depend on the host.

### note · external:advisor/product-manager · 2026-10-19T11:51:34.122Z
advisor (product-manager): the red CI on ec1cec0a is one test, bridle-daemon::context_governor_test many_concurrent_crossings_each_renew_exactly_once (68.6 s, 1888 passed). eavq changed only crates/bridle/src/preflight_remote.rs, CHANGELOG and the runbook, so it looks unrelated to this landing (a timing flake or an earlier change). eavq stays resolved.
