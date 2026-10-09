---
id: p88z
title: Sign bridle with a stable local certificate on Macs, so the firewall's Allow survives every rebuild
kind: feature
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [sign-binaries-on-intel-macs-cs7x, jmpf]
tasks: [br-p88z]
closed: 2026-10-09T23:11:03Z
---

## The ask


The human, 2026-10-04 (via the advisor), moving dalek's daemons to launchd:

> FYI I get os x popups sometimes "do you want to allow incoming connections"

The advisor explained: the daemons (`100.100.189.100:7401`, `:7405`) and the gateway (`:7878`)
listen on dalek's Tailscale address (k7mw), and the macOS application firewall is on, so macOS asks.
dalek is x86_64, so bridle is ad-hoc signed at link time
([[sign-binaries-on-intel-macs-cs7x|cs7x]], `.cargo/config.toml`); an ad-hoc signature has no
stable identity, so each rebuild (every self-upgrade) is a new program to the firewall, which
forgets the Allow and asks again. A missed or denied prompt (likely when launchd starts a daemon
while the human is away) blocks other machines (the NUC) from that daemon. Proposed: "sign bridle
with a fixed certificate created on this Mac, so macOS sees every rebuild as the same program and
remembers your answer." The human, verbatim:

> yes definitely.

## Wanted

- A one-time setup step for the human: create a self-signed code-signing certificate in the login
  keychain (e.g. "bridle local signing"), trusted for code signing on this Mac only. A
  `bridle` or `just` command that creates it, or written steps, whichever is simpler and doesn't
  need the human's keychain password more than once.
- Every build on that Mac (`just install`, self-upgrade, worker builds) signs with that identity
  (`codesign --force -s "<name>"`) when it exists, and falls back to today's ad-hoc signature when
  it doesn't (CI, other machines). Check it works on arm64 too.
- After the first Allow, a rebuild doesn't prompt again (verify on dalek across a self-upgrade).
- Agents must be able to sign unattended (launchd-started daemons, no GUI prompt): check that the
  key's access control allows `codesign` without a keychain prompt.
- Document it in `docs/context/adding-a-project.md` or the machine setup doc, and in CLAUDE.md's
  "Build configuration".

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
