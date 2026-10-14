+++
id = "br-sh8y"
title = "Remote bridle over ssh: also try ~/.local/bin and ~/.cargo/bin, and say 'bridle not found on <host>'"
kind = "fix"
state = "integrated"
created_at = "2026-10-14T11:42:43.717Z"
updated_at = "2026-10-14T18:09:46.503357Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:sh8y",
]
branch = "bridle/sh8y"
commit = "8ddad90b241847a79454994654add8c50ab8fc4a"
summary = """Remote bridle over ssh now finds bridle in ~/.local/bin and ~/.cargo/bin. token_pair.rs has remote_bridle_cmd (PATH extension, then `exec bridle` with every arg single-quoted) and remote_failure (exit 127 -> "bridle not found on <host>", 255 -> "<host> unreachable: ...", else bridle's own last stderr line); Exec uses both, so token pair, project stop/start --machine get the text with no change to those files. preflight_remote.rs runs `command -v bridle; bridle --version` with the same PATH, shows "0.9.2 at <path>", and separates not found, unparsable and too old. Tests: quoting/snippet string, 127/255/other, preflight cases; token_pair_test.rs argv assertions updated for the quoted form. Docs: cli.md and CHANGELOG. The "Remote PATH" step in add-a-machine/WSL2 docs belongs to the docs ticket (yp8f) and does not exist yet, so not touched. Caveat: just check (1675 passed) ran on the pre-merge commit; merging main afterwards only brought in gateway and ticket files."""
ticket = "sh8y"
+++

docs/tickets/open/remote-bridle-over-ssh-also-try-local-bin-and-cargo-bin-and-sh8y.md

## Thread

### note · agent:sh8y · 2026-10-14T17:28:49.305Z
Code, docs and CHANGELOG are done in the worktree (uncommitted); waiting on the final just check (log /tmp/sh8y-check.log), then commit, merge main, summary, report.

### note · agent:sh8y · 2026-10-14T17:58:14.952Z
done: remote bridle finds ~/.local/bin and ~/.cargo/bin; 'bridle not found on <host>' (127) / 'unreachable' (255); preflight shows path. just check exit 0, 1675 passed, ran on c5f0b262; branch tip 52d7d30dbbf7 is that plus a merge of main (gateway/tickets only, check not re-run). Summary written.

### note · agent:manager-2 · 2026-10-14T17:58:17.664Z
built: 52d7d30dbbf7 (branch bridle/sh8y)

### note · agent:manager-2 · 2026-10-14T18:09:46.503Z
integrated: 8ddad90b241847a79454994654add8c50ab8fc4a (branch bridle/sh8y)
