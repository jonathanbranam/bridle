---
id: br-p2ad
title: "add-a-machine: WSL2 bullet installs from the release script; state the release-install check (fv86 ends)"
kind: docs
created: 2026-10-17
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-fv86]
---

## The ask

Follow-up to br-fv86 (install bridle from a release with no clone; landed 2026-10-10 as
scripts/install-release.sh). Two small ends found when verifying it (advisor product-manager,
2026-10-17); readied under rule ready-without-asking (docs chore, self-evident).

**Problem.** (1) `docs/context/add-a-machine.md:19-20` tells the WSL2 machine to follow the WSL2
guide "up to and including `just install`", but that guide no longer has a `just install` step:
WSL2 installs from the release script like Linux and macOS. (2) fv86's done-condition "`bridle
doctor` passes on a machine with no bridle clone" can't pass as written: doctor checks a repo.

**Approach.** (1) Reword the WSL2 bullet to: follow the WSL2 guide for the OS side (Tailscale,
linger, SSH), then install bridle with the release script below, as the Linux and macOS bullets say.
Check `docs/context/windows-wsl2-host.md` agrees. (2) No change to doctor. In add-a-machine.md,
state the check for a release install as: `bridle --version` prints the release, and
`bridle doctor` run inside a project clone passes (outside a clone it is not expected to).

**Scope.** Docs only: add-a-machine.md (and windows-wsl2-host.md if it disagrees). No code.
Acceptance: `just check`.
