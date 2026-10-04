---
id: bek3
title: The gateway runs detached and keeps itself current, like the daemon, on dalek and on client machines
kind: feature
opened: 2026-10-04
repos: [bridle-ui]
changes: []
specs: []
needs: []
see: [tc7t, chvf, jmpf, essy, mnzh]
tasks: []
---

## The ask


The human, verbatim (2026-10-04, to the bridle-ui aide, right after choosing full automation in
[[a-landing-that-needs-a-ui-install-or-gateway-restart-to-show-tc7t|tc7t]]; "Bridal" is bridle,
dictated):

> And let's get the work spec'd out and running also for running the gateway detached in the
> background the same way the daemon runs. We've already solved these problems for the daemon,
> so we should just copy and use basically the same solution. And this should be the same
> solution that happens when on a client machine. So, you know, that I don't think has been
> released yet, but that's an important thing, right? If we, we're going to need an automated way
> for Bridal to keep itself updated and updating, restarting, updating all the daemons includes
> updating a running gateway. You know, where every project or machine should have some config to
> control, you know, where the gateway runs and I assume what port it, what port it listens on,
> things like that.

## Context

- The daemon: `bridle serve --detach` re-executes itself in a new process group, logs to
  `.bridle/daemon.log` and waits for `/v1/health` (`docs/design/agent-host/daemon.md`, "Running
  it"). With `[daemon] self_upgrade = true` it builds the newest green `main` at a quiet point and
  restarts in place, resuming every agent (same doc, "Automatic upgrade").
- The gateway today: `bridle gateway` runs in the foreground. `bridle gateway install` writes a
  launchd plist / systemd user unit (`dev.bridle.gateway`, restart on failure, log
  `~/.bridle/gateway.log`) and only prints the commands to load it
  (`docs/design/human-web-ui.md`, build task 10). On dalek on 2026-10-04 it was a foreground
  process started at 8:55 AM ET, still running pre-upgrade code after the bridle binary was
  rebuilt at 5:09 PM (incident ui-wdp3 in bridle-ui).
- Gateway config is the optional `[gateway]` section in the machine's `~/.bridle/config.toml`
  (login, bind address); see [[a-gateway-section-in-bridle-config-toml-stops-every-daemon-f-jmpf|jmpf]].
- Client machines staying current with bridle by release:
  [[client-machines-stay-current-with-bridle-workflow-and-daemon-chvf|chvf]].
