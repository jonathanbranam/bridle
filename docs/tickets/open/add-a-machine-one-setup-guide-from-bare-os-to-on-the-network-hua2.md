---
id: hua2
title: "Add a machine: one setup guide from bare OS to on the network (config, tokens, services, moving a project)"
kind: chore
opened: 2026-10-09
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [v7ug, sk7p, gdf3, rjd5, j7r4]
tasks: [br-hua2]
---

## The ask

The human, 2026-10-09 ~10:15 AM ET, verbatim (to advisor product-manager): "Put tickets in there that would enable basically efficient and pretty direct setup for a new machine. ... I want that as a priority so that I can use it to set up the new Windows machine and add it to the network. So it would include everything for configuring background daemons, launching tokens between machines, tokens between projects, and then as a as a additional feature I would like to see if we can get it in is automated project transfer between machines."

Workstream: machine setup (docs/notes/roadmap.md). Found by the PdM's survey: the WSL2 guide (docs/context/windows-wsl2-host.md) stops at `just install`; the config, token and service steps are spread over adding-a-project.md, nuc-host.md and cli.md, and some are missing.

The ask: one "add a machine" procedure, in order, any OS, with the WSL2 and NUC specifics linked:
1. GitHub and git per machine: SSH key, `gh auth login` (for `[ci] github`), a distinct git identity per machine (j7r4 rec 6).
2. `~/.bridle/config.toml` on the new machine (`[machine] name`, `[machines]`, `[projects]`, workflow, budget, gateway, mail) and the matching lines on every other machine.
3. Visitor tokens both ways (by hand until sk7p's `bridle token pair` lands; then that command) and peer tokens for mail (gdf3's rule: one per receiving project per sending machine, minted on the receiver).
4. Services: `bridle systemd install` / launchd, `gateway install`, `mail install`, with the workspace layout (`<P>-workspace/<P>`), linger, and `bridle doctor` to check.
5. Moving a project onto it (nuc-host.md runbook) with the two traps fixed in the text: remove the old machine's launchd plist or systemd unit, and take over by hand before enabling the new unit.
Docs only. Acceptance: a reader can bring the Windows PC onto the network from this page alone; ASCII.
