---
id: br-kt25
title: Move a project between machines with one command (bridle project move)
kind: feature
created: 2026-10-09
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-24mj, br-kuw2, br-hw6c, br-rjd5, br-sk7p, br-n63z, br-3haz]
parent: br-kx5u
theme: multi-machine
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

## Now: a design pass that splits the move into build tickets

The human, 2026-10-12 ~12:00 AM ET (to advisor product-manager): "This is actually a massive
ticket that is likely an epic itself. I asked for a rewview and design I see related tickets but
not sure where the design it."

PdM decision: kt25 is the design for the rest of the move, not one build. The epic stays kx5u
(its done-when is this move); the design ends in build tickets with `parent: br-kx5u`, each
small enough for one worker. The designer (Sonnet) writes `## Design` in this file:

1. **Inputs.** The human's decisions on this thread (2026-10-11: the DB is NOT copied; list what
   is lost and ask; move scheduled messages; recreate tokens and config). The 3pc2 inventory
   (`inventory-everything-a-project-has-on-a-machine-and-what-mov-3pc2.md`), Sections 3, 4 and 5.
   What ga6s, n4ug and 86qp (53gy A, B, C) already build.
2. **A coverage table**: every item in 3pc2 Sections 4 and 5, one row each: covered by
   ga6s/n4ug/86qp, a new child here, named in the pre-move listing only, moot (because the DB
   is not copied), or for jw75 (moving bridle itself: 3pc2 Section 2). Nothing silently dropped.
3. **Answers proposed** for the three open questions above (SSH-driven or authorise-only; dirty
   worktrees and sessions; CLI over SSH now or wait for kuw2), each with a recommendation. The
   human's rule: one command, nothing by hand, nothing left behind.
4. **The children**, in build order, each with its ask, acceptance and size. Highest risks from
   3pc2 Section 5 first (committed absolute `system_prompt` paths; unpushed branches and dirty
   worktrees; the launchd plist left loaded on OLD).

Then the human reviews the design; only after that are the children readied.
