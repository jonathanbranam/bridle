---
id: br-eavq
title: "project preflight falsely fails claude, gh and tmux on a macOS target: ssh's bare PATH"
kind: bug
created: 2026-10-18
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kx5u]
---

Found in kx5u trial 2 (data-contracts remus -> dalek, 2026-10-18), the first move with a macOS NEW.
`bridle --project data-contracts project preflight --to dalek --path ...` printed:

    FAIL  claude login: on dalek run: claude auth login (from a local terminal or tmux, not over ssh, on macOS)
    FAIL  gh auth: on dalek run: gh auth login
    FAIL  tmux: on dalek run: install tmux (brew install tmux / apt install tmux)

All three are present and logged in on dalek: tmux and gh at /usr/local/bin, claude at
~/.local/bin, `gh auth status` logged in. Trial 1 moved to remus (Linux), so this path was never run.

Cause (read from `crates/bridle/src/preflight_remote.rs`): `probes()` scripts run as a bare
`ssh host '<script>'`. On macOS that shell's PATH is /usr/bin:/bin:/usr/sbin:/sbin, without
/usr/local/bin, /opt/homebrew/bin or ~/.local/bin. Only `VERSION_SNIPPET` extends PATH (with
~/.local/bin and ~/.cargo/bin). Even with a fixed PATH, `claude auth status` over ssh on macOS may
not read the keychain (the same limit doctor.rs and ticket nrbf name), so it would still report
logged out.

## The ask

1. Run every NEW probe with the same PATH the daemon gets on that machine (at least
   ~/.local/bin, ~/.cargo/bin, /usr/local/bin, /opt/homebrew/bin), not ssh's bare one.
2. On a macOS NEW, don't FAIL the claude login check when ssh can't see the keychain. Report it
   as "unverified over ssh: check with `claude auth status` in a terminal on NEW" (a warning that
   doesn't fail the preflight), or check it some way that works over ssh.
3. A test for each fix with a fake Remote.
4. Update docs/context/move-a-project.md step 1 if the output changes.

## Proposal

PATH: prefix one shared PATH line (the VERSION_SNIPPET one, extended with the Homebrew dirs) to
every probe script. Claude: on Darwin, when `claude auth status` reports logged out and ssh is
the transport, pass with a warning saying it was not verified. A false FAIL is worse than an
unverified pass because the start step (`project start`, then doctor) checks the login again.
