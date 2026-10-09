---
id: kt25
title: Move a project between machines with one command (bridle project move)
kind: feature
opened: 2026-10-09
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [24mj, kuw2, hw6c, rjd5, sk7p, n63z, 3haz]
tasks: []
---

## The ask

The human, 2026-10-09 ~10:15 AM ET, verbatim (to advisor product-manager): "Put tickets in there that would enable basically efficient and pretty direct setup for a new machine. ... I want that as a priority so that I can use it to set up the new Windows machine and add it to the network. So it would include everything for configuring background daemons, launching tokens between machines, tokens between projects, and then as a as a additional feature I would like to see if we can get it in is automated project transfer between machines."

Workstream: machine setup, last phase ("an additional feature"). Today moving a project is the 7-step manual runbook in docs/context/nuc-host.md; the inbox, usage history and session transcripts stay behind, and services, mail, gateway and tokens are redone by hand. Prior design: 24mj Part 2 (resolved ticket, unbuilt) and kuw2 item 5 (machine daemon, waiting on the human's review, br-efs2).

The ask: `bridle project move <P> --to <machine>`: release on the old machine (owner.toml released + SHAs), stop and uninstall its service, take over on the new one (clone if needed, verify SHAs, install the unit, retarget mail and gateway), update `[projects]` everywhere, re-mint tokens, and carry or forward the unread inbox.

Needs a design (the designer role) and the human's decisions first:
- Does the old machine drive the new one over SSH, or only authorise it?
- What happens to uncommitted worktree work and agent sessions that can't resume?
- A CLI over SSH first, or wait for the machine daemon (kuw2)?
Builds on sk7p/n63z (tokens), rjd5 (sync), 3haz slice 2 (br-n7cg, mail follows the move).
