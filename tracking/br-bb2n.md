+++
id = "br-bb2n"
title = "K2: project preflight: check the new machine is ready before a move"
kind = "feature"
state = "planned"
created_at = "2026-10-12T13:38:40.934Z"
updated_at = "2026-10-12T13:39:03.872262Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "bb2n"
+++

docs/tickets/open/k2-project-preflight-check-the-new-machine-is-ready-before-a-bb2n.md

## Thread

### note · agent:pm-1 · 2026-10-12T13:39:02.361Z
Brief (pm-1). Model: Sonnet. Size: M. Blocked by br-3ch7 (K1), which creates `bridle project preflight` and its list-of-checks shape; build on it, don't redo it. Read the ask above and br-kt25 '## Design' section 4.

Goal: add the NEW-machine checks to `project preflight <P> --to M`, run over ssh. Each failing item prints what to run on NEW; unreachable NEW fails with the ssh error; all green prints nothing alarming. Reuse existing `bridle doctor` checks where they exist (run doctor remotely if it already covers an item) rather than reimplementing. Also run the 2wnn comparison (`projects sync` without `--apply`) and the clone check (present, or cloned after a confirm).

Items: bridle binary >= OLD's version, claude logged in, gh auth, git push access to P's origin, P's toolchain command exists, tmux, `[machine] name` equals the name owner.toml will record, Tailscale name resolves, timezone, AWS credentials when `[mail]` names P, and on Linux WSL systemd and linger.

Files likely: the preflight code from K1 in crates/bridle/src, doctor checks, docs/design/cli.md, CHANGELOG.md. Tests: fake the ssh runner (no real ssh in tests); one test per failing-item message plus unreachable.

Acceptance: `just check` passes. Out of scope: the move command, anything about OLD's drain/stop, auto-fixing NEW.
