---
id: 4r3k
title: The NUC recovers everything on boot
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: [projects-on-other-machines-by-config-k7mw, dotfiles-local-as-a-bridle-project-35mw]
see: [orchestrator-identity-and-recovery-7d62, hold-the-orchestrator-relaunch-8fsx, pause-before-a-planned-reboot-3nyk, unattended-while-the-human-travels-tv8r]
---

## The ask

The human (2026-09-30, via the advisor): after a reboot the NUC should come back with no login:
tmux with their windows and panes, bridle's daemons for the projects it owns, and an orchestrator.
Advisors start on request. Their plan for tmux, verbatim:

> I prefer tmux-resurrect as the first option; on boot, run tmux, tmux-resurrect, and then if (a)
> there is no resurrect file or (b) there is no tagged orchestrator pane then: create a new tmux
> session (or window) and tag a pane with orchestrator

## Checked on the NUC (2026-09-30)

- `Linger=no`: the systemd user manager doesn't start without a login (`docs/context/nuc-host.md`
  says linger is on; it isn't). tmux itself needs no login, only that.
- tmux 3.6 with tpm, tmux-resurrect, no tmux-continuum: no automatic save (last save 02:03 UTC) and
  no automatic restore. The laptop's resurrect fix isn't there (35mw).
- Git pushes use an SSH key without a passphrase: fine at boot. Tailscale is a system service.
- No user systemd units. The `claude-rc` unit in `nuc-host.md` was never set up and won't be (the
  human).

## Plan (approved in outline)

1. **tmux comes back** (dotfiles-local, 35mw): linger on; tmux-continuum starts tmux at boot and
   restores with resurrect; saves every 15 min.
2. **Daemons without tmux**: `bridle serve` needs no terminal. Each project this machine owns
   (k7mw's machine config) runs as a systemd user service with `Restart=on-failure`: a `bridle
   systemd install`, the Linux sibling of `bridle launchd install`.
3. **Orchestrator in tmux**: after the restore, a boot step (e.g. `bridle up`) looks for a pane
   tagged `@bridle orchestrator`. If there's no resurrect file or no tagged pane, it creates a
   session (or window) and tags a pane. Then it runs `bridle session orchestrator` there; the
   daemon's supervision relaunches it from then on. One orchestrator for the whole machine (the
   human's working rule, ma8e), whatever the number of projects.
   Catch: resurrect doesn't save pane options, so a restored pane loses its tag and (b) would
   always hold, leaving the old orchestrator pane as a stray shell. Save the tags with
   `@resurrect-hook-post-save-all` and put them back with `@resurrect-hook-post-restore-all`.
4. **Advisors on request**: `bridle advisor start <name> --brief` (built, br-284f).
5. **Detached, attach later**: a detached tmux session is that; `ssh nuc`, `tmux attach`.
6. **Check in the drill**: `gh auth` survives a reboot; the orchestrator pid file (7d62).

## Drill

Reboot the NUC with a project running. Pass: tmux restored, the daemon up, the orchestrator up in
its tagged pane with its wake loop, the manager resumed. The human may run a first reboot on
2026-09-30 evening, before any of this is built, to see the baseline.
