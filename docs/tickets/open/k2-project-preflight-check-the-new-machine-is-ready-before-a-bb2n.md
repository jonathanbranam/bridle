---
id: br-bb2n
title: "K2: project preflight: check the new machine is ready before a move"
kind: feature
created: 2026-10-12
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: [br-3ch7]
related: [br-kt25]
parent: br-kx5u
theme: multi-machine
---

## The ask

Child K2 of the kx5u design (br-kt25, '## Design', section 4), written by the designer; the human, 2026-10-12: "order - whatever order you think is best" (PdM: K1 and K2 first, independent of the open grilling questions).

**K2. Preflight: NEW is ready (M).** Ask: `project preflight` also checks NEW over ssh (H):
`bridle` binary at least OLD's version, `claude` logged in, `gh auth`, git push access to P's
origin, P's toolchain command exists, tmux, `[machine] name` equals the hostname-or-name
`owner.toml` will record, Tailscale name resolves, timezone, AWS credentials when `[mail]` names P,
and on Linux: WSL systemd and linger. Also runs the 2wnn comparison (`projects sync` without
`--apply`) and the clone check (present, or cloned with a confirm). Acceptance: each failing item
prints what to run on NEW; all-green prints nothing alarming; unreachable NEW fails with the ssh
error. Reuses `bridle doctor` checks where they exist.
