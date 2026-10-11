---
id: br-hf8a
title: Rename the NUC's host (vega?)
kind: chore
created: 2026-10-11
created_by: external:advisor/wsl2
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-v7ug]
---

## The ask

The human, 2026-10-11 (to advisor wsl2): "Can you also add instructions for renaming NUC plz"

The NUC's new name is not decided yet. `vega` leads (vesta and luna are the alternatives; see
`docs/context/name-ideas.md` and `docs/context/naming.md`). Below, `vega` stands for whatever
name is chosen.

Then, follow-up messages: "Including for bridle" / "On both machines". So bridle's machine name
changes too, on dalek and on the NUC (step 6), not only the host name.

## Plan

Bridle knows the NUC by its machine name `nuc` in `~/.bridle/config.toml` (`[machine] name`,
`[machines]`, `[projects]`) and in `~/.bridle/credentials.toml`. `[machines]` maps a machine name
to the host name to reach it by. Steps 1-5 rename the host and point bridle at it (bridle keeps
working with `nuc = "vega"`); step 6 then renames bridle's machine name, which touches tokens,
identities and routing, so the orchestrator confirms its exact sequence before it is done.

## Steps

1. **Linux host name** (on the NUC):

   ```
   sudo hostnamectl set-hostname vega
   sudo sed -i 's/^127\.0\.1\.1[[:space:]]\+nuc$/127.0.1.1 vega/' /etc/hosts
   hostnamectl --static; grep 127.0.1.1 /etc/hosts      # vega, and 127.0.1.1 vega
   ```

2. **Tailscale name** (on the NUC), so MagicDNS answers `vega`:

   ```
   sudo tailscale set --hostname=vega
   tailscale status --self                              # the first line shows vega
   ```

   If the admin console shows the machine still as `nuc`, it was renamed there by hand (that
   pins it); rename it in the console too (Machines, the machine's menu, Edit machine name).
   The old name `nuc` stops resolving at once, so do steps 3 and 4 straight after.

3. **Bridle config, both machines** (`~/.bridle/config.toml` on dalek and on the NUC), in
   `[machines]`: `nuc = "vega"` (was `nuc = "nuc"`). On the NUC also `[gateway] public_url =
   "http://vega.tailbc91f5.ts.net:7878"`. Then restart the daemons on both (or let the
   orchestrator do it). Check from dalek: `bridle daemons` lists the NUC's projects as reachable,
   and the gateway opens at the new URL. Add `nuc = "vega"` to remus's config too once bridle is
   installed there (ticket v7ug).

4. **SSH from other machines**: `ssh vega` works through MagicDNS. Anything that says `ssh nuc`
   (scripts, docs, muscle memory) breaks; clear the old host key with `ssh-keygen -R nuc` and
   accept the new one on first `ssh vega` (Tailscale SSH, so no keys to copy).

5. **dotfiles-local**: nothing breaks, since `host-nuc/rcrc` sets `HOSTNAME="nuc"` itself. Renaming
   `host-nuc/` to `host-vega/` (and its `HOSTNAME`) is optional and can come later: after the
   rename, `RCRC=<clone>/host-vega/rcrc rcup -B vega` re-links `~/.rcrc`.

6. **Bridle's machine name `nuc` -> `vega`, on both machines.** What carries the name (found
   2026-10-11 by advisor wsl2; the orchestrator to confirm the list and the order, with daemons
   stopped or restarted around it):

   - `~/.bridle/config.toml` on dalek, the NUC (and remus once it has bridle): `[machines]` key
     `nuc` -> `vega`, and every `[projects]` entry `machine = "nuc"` -> `"vega"` (meta-notes,
     notes, dotfiles-local, meta-notes-ui). On the NUC also `[machine] name = "vega"`.
   - `~/.bridle/credentials.toml` on dalek: the sections keyed by the machine they reach,
     `[advisor.nuc]`, `[human.nuc]`, `[orchestrator.nuc]` -> `.vega` (the tokens inside are the
     NUC daemons' `<name>@dalek` visitors and don't change).
   - Tokens that name the NUC itself, minted on dalek's daemons: the peer token `peer:nuc` (the
     NUC's `[peer]` entries for dalek's projects) and the visitor tokens `<name>@nuc` (the NUC's
     `[advisor.dalek]`, `[aide.dalek]`, `[orchestrator.dalek]`, `[human.dalek]`). Re-mint each
     as `peer:vega` / `<name>@vega` (`bridle token create --peer vega`, `bridle token create
     <name> --machine vega [--home]`, see `docs/design/agent-host/principals.md`), paste them
     into the NUC's credentials, and revoke the old ones.
   - systemd units: `bridle systemd install` picks projects by `[machine] name`, so re-run it on
     the NUC after the rename (check the unit names don't change, or uninstall first).
   - Not migrated: history. Stored messages and events say `@nuc`; leave them, and leave the docs
     and incident history as they are. Unknown: whether a reply to an old `@nuc` message still
     routes after the rename (orchestrator to check).

   Check afterwards: `bridle doctor` and `bridle daemons` on each machine, then a message each
   way (dalek -> a NUC project, and the NUC's orchestrator -> bridle) arrives and its `from`
   says `@vega`.

## Not decided

- The name (`vega` leads).
