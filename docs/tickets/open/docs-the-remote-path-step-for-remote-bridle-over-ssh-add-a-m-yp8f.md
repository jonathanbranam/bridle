---
id: br-yp8f
title: "Docs: the Remote PATH step for remote bridle over ssh (add-a-machine, WSL2 guide)"
kind: docs
created: 2026-10-14
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-5cru, br-sh8y]
---

Step 2 of br-5cru's design (option B). The human approved B, 2026-10-14 ~7:42 AM ET, via advisor product-manager: "That sounds ok." Can go now, ahead of br-sh8y (the code).

## The ask

Until br-sh8y lands, remote bridle over ssh (token pair, project stop --machine, project start, project preflight --to) needs bridle on the remote non-interactive PATH. Say so where machines are set up.

- `docs/context/add-a-machine.md` and `docs/context/windows-wsl2-host.md`: a short "Remote PATH" step in the ssh section with the two zshenv lines from br-5cru's "Interim" section (macOS `/etc/zshenv`; Ubuntu and WSL2 `/etc/zsh/zshenv`), the check `ssh <host> 'command -v bridle'`, and which commands need it.
- `windows-wsl2-host.md` ~line 242: its `export` snippet fixes only the interactive shell; say so.
- CHANGELOG (Unreleased) if docs entries go there.

Acceptance: the step reads correctly on both docs; `bridle ticket check` clean. No code.

Model: Haiku. Size: XS. Don't run it in parallel with another cli.md editor; br-sh8y edits cli.md and these docs' wording later.
