---
id: br-sh8y
title: "Remote bridle over ssh: also try ~/.local/bin and ~/.cargo/bin, and say 'bridle not found on <host>'"
kind: fix
created: 2026-10-14
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-5cru]
resolved: 2026-10-15T20:52:32Z
---

Step 1 of br-5cru's design (option B). The human approved B, 2026-10-14 ~7:42 AM ET, via advisor product-manager: "That sounds ok." The full design is the worker's comment on br-5cru's thread (tracking/br-5cru.md on bridle/state; `bridle ticket show` may print it empty, bug br-xcb4).

## The ask

Commands that run bridle on another machine over ssh (token pair, project stop --machine, project start, project preflight --to) find bridle without the user editing the remote non-interactive PATH.

- One helper (for example `remote_bridle_cmd(args) -> String` in `token_pair.rs`, used by `preflight_remote.rs`) builds `PATH="$PATH:$HOME/.local/bin:$HOME/.cargo/bin"; exec bridle <quoted args>`.
- The Runner error tells apart exit 127 (report "bridle not found on <host>"), 255 (ssh failed: "unreachable") and anything else (bridle's own error).
- Preflight's version check runs `command -v bridle; bridle --version` and shows the path in its pass message ("0.9.2 at /home/u/.cargo/bin/bridle"); it separates not found, unparsable and too old.
- `project_stop.rs` and `project_start.rs` only show the new error text.

Files: crates/bridle/src token_pair.rs, preflight_remote.rs, project_stop.rs, project_start.rs and their tests. Docs: `docs/design/cli.md` line ~148 ("bridle on the remote PATH" becomes "on the remote PATH or in ~/.local/bin or ~/.cargo/bin"), and the "Remote PATH" step in add-a-machine / windows-wsl2-host (from the docs ticket) becomes "not needed unless bridle is installed elsewhere"; CHANGELOG.

Acceptance: tests with the fake Runner or fake-ssh scripts cover 127, 255 and other; one test of the snippet string and its quoting; `just check`.

Model: Sonnet. Size: S.

Out of scope (rejected in the design): a per-machine bridle path in `[machines]` (YAGNI until someone installs elsewhere), login shells, sshd config, a sudo symlink.
