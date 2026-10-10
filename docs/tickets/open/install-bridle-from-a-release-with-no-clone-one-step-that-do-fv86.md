---
id: fv86
title: "Install bridle from a release with no clone: one step that downloads, verifies and installs the binary"
kind: feature
opened: 2026-10-10
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [mrhe, chvf, v7ug]
tasks: []
---

## The ask

The human, 2026-10-10 ~2:00 PM ET, verbatim (to advisor product-manager), on what machine setup
must deliver:

> What I expect from this, if it's not in this epic, we should add it, is that I can install
> bridle without doing a clone, and that can run and self-upgrade. It includes the bridle daemon,
> the bridle gateway, and, of course, the bridle-ui as part of that, all fixed to a version that
> comes from bridle. Tell me if all that's ready, and then project setup is automated with a
> command and syncing. Tell me if that's correct or not. If the UI is not done, that could be a
> separate epic because I understand it's a different piece of work.

Earlier, on mrhe (2026-09-29): "I'm fine with having the binary clone from GitHub in an initial
step."

## Where it stands (PdM, 2026-10-10)

- Releases publish `bridle-<tag>-<target>.tar.gz` for x86_64 Linux and both macOS targets, with
  `SHA256SUMS` (`docs/README.md`, "Releases").
- Once installed, `[daemon] self_upgrade = "release"` (br-88d4) keeps the binary current, and
  br-751e keeps the workflow at the binary's tag. The gateway is the same binary and re-executes
  itself when the file changes (`docs/design/agent-host/daemon.md`).
- **Missing: the first install.** `docs/context/add-a-machine.md` and the WSL2 guide install with
  `just install` in a bridle clone. Nothing documents or scripts installing the release binary.

## The ask

1. One documented step, or a small script, that installs the newest (or a named) release on
   macOS or Linux: download the tarball for this platform, check it against `SHA256SUMS`, put
   `bridle` on the PATH (`~/.local/bin` or `~/.cargo/bin`), and print the next step. No clone,
   no Rust toolchain.
2. The add-a-machine and WSL2 guides use it, and set `self_upgrade = "release"` from the start;
   `bridle doctor` passes on a machine with no bridle clone.
3. macOS: say what happens to signing and the firewall prompt with a release binary (CLAUDE.md,
   "Build configuration"; p88z).

Out of scope: the UI (its own epic), and project setup and sync (rjd5, epic machine-sync).
