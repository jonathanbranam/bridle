---
id: br-4r3k
title: A machine recovers on boot (vesta, remus, any Linux/WSL host): orchestrator first, then the sessions each project asks for
created: 2026-09-30
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [projects-on-other-machines-by-config-k7mw, dotfiles-local-as-a-bridle-project-35mw, make-the-orchestrator-non-interactive-headless-z485, orchestrator-identity-and-recovery-7d62, hold-the-orchestrator-relaunch-8fsx, pause-before-a-planned-reboot-3nyk, unattended-while-the-human-travels-tv8r]
kind: feature
---

## The ask

The human (2026-09-30, via the advisor): after a reboot the NUC should come back with no login:
tmux with their windows and panes, bridle's daemons for the projects it owns, and an orchestrator.
Advisors start on request. Their plan for tmux, verbatim:

> I prefer tmux-resurrect as the first option; on boot, run tmux, tmux-resurrect, and then if (a)
> there is no resurrect file or (b) there is no tagged orchestrator pane then: create a new tmux
> session (or window) and tag a pane with orchestrator

## Update 2026-10-14 (advisor wsl2): every machine, orchestrator first

**What happened.** At 8:32 PM ET Windows Update rebooted romulus, the Windows PC that hosts remus
(WSL2). Event 1074: `MoUsoCoreWorker.exe` ("Service pack (Planned)"), then `TrustedInstaller.exe`
("Operating System: Upgrade (Planned)"). Windows Home can't turn this off; active hours (at most
18 h) only push it to the night. With no login, WSL came back (the `bridle WSL start` task) and so
did everything that is a systemd user service: Tailscale, both daemons (remus-home, data-contracts),
both mail bridges, and the daemons resumed their managers. Nothing else came back. Once the human
logged in and ran `tmux`, tmux-resurrect restored the window (three panes), but each pane was a
plain `zsh`: no orchestrator, no aide, and no `@bridle` pane tags, so the supervisor had no pane to
relaunch into (the catch under step 3 below, now seen).

**The human, verbatim (2026-10-14, to advisor wsl2):**

> The main thing is the orchestrator, I guess, or one per project. Those would be the big things
> that I would want to come back. It's a lot of auto-start stuff that we need to think about. We
> don't want managers coming up on most projects unless there's work to do. I think, as a first
> step, it'd be good to have the orchestrator back up, and they could help with other things.

> The bigger question for me is: do we want an aide per project, and how is that configured? I
> think there needs to be a per-project configuration that says, "After reboot, restore these
> roles." Probably just aide in most cases, but maybe there's an advisor role that I always want
> running for some reason.

> You really have to think through carefully what you want to have happen at startup, because it
> always happens at startup, right? If you say start an aide in every project, and then, for some
> reason, you have 30 projects and 30 aides, that destroys the machine. If you set it to run at
> startup, you have to boot into safe mode or something to turn it off. Ideally, there's some
> mechanism for throttling this or controlling this.

> I would say to have an orchestrator come up per machine. What I think, ideally, I would want is
> the configuration for which aides come up, and then a good solution for orchestrators starting
> aides on my behalf. [...] if the aides don't come back, or if I don't turn on auto-resurrect on
> the aides, then I want the orchestrator to be able to start them in a reasonable way. [...] we
> need to make sure the [pane] tagging persists.

> We don't need to over-architect the solution right now [the orchestrator may become a managed,
> background agent, z485].

> It's pretty urgent because I want to move the bridle work. I want to move a lot of work over to
> this machine. If this continues to be a problem or is something we can't overcome, I'm going to
> end up dual-booting to Linux so I have more control over this. Let's try a solution first, though.

**Scope now (replaces "the NUC"; applies to vesta, remus and any Linux or WSL host; macOS where it
costs nothing extra).** Agreed with the human in outline:

1. **One orchestrator per machine at boot**, the minimum and the first slice. After tmux is up (the
   machine's own tmux unit or resurrect), a boot step finds the pane tagged `@bridle orchestrator`,
   or creates a session/window and tags one, and runs `bridle session orchestrator` there. The
   daemon's supervisor keeps it from then on. Pane tags must survive a resurrect save/restore
   (`@resurrect-hook-post-save-all` / `@resurrect-hook-post-restore-all`, step 3 below). Written so
   it carries over if the orchestrator becomes a background agent (z485, now dropped).
2. **Per-project "restore at boot" list, off by default**, e.g. `[boot] sessions = ["aide"]` in the
   project's config: which interactive roles (aide, a named advisor) come back after a reboot.
   Managers stay as today (`autostart`, and only when there is work); this list doesn't touch them.
3. **Startup safety, since it always runs:**
   - a machine-wide cap on sessions started at boot; any over it are listed for the orchestrator,
     not started;
   - starts staggered, not all at once;
   - a kill switch reachable over ssh, checked before anything starts (e.g. a file
     `~/.bridle/no-autostart` or a command), so a bad config is one command away from off, never
     safe mode.
4. **The orchestrator starts sessions for the human.** For anything not restored at boot, it opens
   a tagged pane and runs `bridle session aide --project P` (or a named advisor) on request, and
   after a reboot it says what didn't come back and offers to start it.
5. **Not now:** the headless orchestrator (z485).

**Blockers dropped.** k7mw (`[machines]` / `[projects]` config) is in use on every machine, and the
daemons are systemd/launchd units installed by `bridle project start`, so plan step 2 is built.
35mw (dotfiles-local) is still pending, but tmux and resurrect already restore on remus, and vesta
has a `tmux.service` unit (`tmux new-session -d`, resurrect save on stop); kept as related.

**Drill (updated):** reboot remus (or let Windows Update do it) with remus-home and data-contracts
running. Pass: with no login, tmux is up, the orchestrator is up in its tagged pane with its wake
loop, each project's managers resumed, and each project's listed sessions started (within the cap).

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
