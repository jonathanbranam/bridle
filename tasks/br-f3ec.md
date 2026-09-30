+++
id = "br-f3ec"
title = "hw6c 1: state branch records its owner host; serve refuses another host's project; serve --take-over"
kind = "feature"
state = "planned"
created_at = "2026-09-30T03:06:13.247Z"
updated_at = "2026-09-30T03:06:15.840683Z"
+++

Ticket: docs/questions/open/one-machine-owns-a-project-hw6c.md (Shape 1; read it, plus docs/design/storage.md state branch and crates/bridle-daemon/src/state_branch.rs, we2r push). Do: the state branch carries one file (e.g. owner.toml: host name, daemon start time) written by the daemon that serves the project. 'bridle serve' fetches origin/bridle/state first; if another host owns it, the daemon refuses to start, saying which host and since when. 'bridle serve --take-over' on the new machine claims it (writes itself as owner, pushed with the normal we2r push) after the old daemon stopped and pushed. Also make the fetch-before-first-flush ordering safe: serve's initial fetch must make the NUC runbook's manual 'git fetch origin bridle/state:bridle/state' unnecessary (state_branch.rs try_fetch only accepts an empty seed; fix so a fresh clone adopts origin's state instead of reporting Diverged). No origin configured / same host / first ever serve must keep working unchanged. Tests with local bare-repo remotes: owned by other host refuses; --take-over succeeds; same host starts; fresh clone adopts origin state. Docs (storage.md, cli.md, daemon.md), CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: tools-only clones (next task), GitHub branch protection.
