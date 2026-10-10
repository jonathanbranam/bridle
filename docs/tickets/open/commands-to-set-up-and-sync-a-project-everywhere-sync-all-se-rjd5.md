---
id: rjd5
title: One idempotent command to set up a project on a machine, and track which projects are set up where
kind: feature
opened: 2026-10-09
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [sk7p, gdf3]
tasks: [br-rjd5]
---

## The ask

The human, verbatim (2026-10-08 ~7:10 PM ET, relayed by the bridle-ui aide, m-7237):

> we've just got to build commands to do this project setup stuff; wayyy too much to keep handling by hand. How many places list projects and ports and tokens... ? We had a command to copy tokens; basically, ideally, I have a command to sync all of this and then a command to add a project that just handles adding to both machines, mail, etc. It could take params for that or ask for confirmation for all the defaults:
>
> Create aide token: [Y/n]
> Add to mail: [Y/n]
>
> Whatever. And can pass -y to accept all defaults.

Then, ~7:20 PM ET (m-7248), after the bridle-ui aide listed the seven places below:

> yes, sync and set up all of those things; everything; it can confirm for any changes during sync with the user, or same accept with -y

## Context (from the bridle-ui aide)

That evening the orchestrator restarted dalek's gateway with `BRIDLE_AS=orchestrator` in its env (every project unreachable 10:37 AM-6:46 PM), and setting up the NUC projects for the web UI (ui-9hq8, br-xg47) meant hand edits.

Where each project's setup lives today, on each machine:

1. `~/.bridle/config.toml` `[projects]` (machine, port): meant to be identical on every box, but drifted (dalek lacked meta-notes-ui until 6:48 PM; the NUC lacks bridle-ui and track-web).
2. The mail section's projects list.
3. `credentials.toml` `[aide]`, `[orchestrator]`, `[advisor]` per project, plus `[<role>.<machine>]` and `[human.<machine>]` on the other machine.
4. `[peer]` tokens per project pair (gdf3).
5. The workspace's `.bridle/tokens/human`.
6. `~/.bridle/daemons/<project>.json`.
7. A launchd plist or systemd unit per daemon.

## The ask

- **`sync`**: brings all seven into agreement on every machine. It shows each change and asks; `-y` accepts all.
- **`add a project`**: does all seven for a new project on both machines (mail included), asking `[Y/n]` per default (create aide token, add to mail, ...) or taking params; `-y` accepts all defaults.

Related: [[pair-machines-token-setup-over-ssh-sk7p|sk7p]] (`bridle token pair`, planned), [[peer-token-setup-guidance-a-token-per-receiving-project-per-gdf3|gdf3]], [[docs/context/adding-a-project|adding a project]].

## Per machine, not everywhere (the human, 2026-10-10 ~10:40 AM ET, to the advisor)

> An adjustment to get Romeo Juliet Delta 5: every project is not necessarily cloned and kept in
> sync on every machine. That's not what I'm asking for, because that's not strictly a
> requirement here at all. We need to be able to selectively keep track of which projects are
> cloned and set up on which machine.
>
> Again, it should be one bridle command to start working on a project on a machine. It should
> not need to ask an orchestrator or do anything. The orchestrator can run the command, but I
> should be able to run a command and say, "bridle, start working to set up a project." I would
> cover everything. The command would need to know, of course, the target folder (destination
> folder for the parent workspace). It should be idempotent. It should check if that's already
> set up and just make sure everything's set up properly, if there are any migrations or updates
> that need to be done.
>
> I do not want every project kept in sync on every machine. That's not what I'm looking for.
> GitHub is our source of truth here. While Dalek owns bridle, Dalek is in charge, and changes
> flow through our workflow, which is already defined. When changes land, they're pushed to the
> remote.
>
> A couple things: at any time, a machine can either check out an existing project or go up to
> date on an existing project. There's a distinct operation from takeover. If I want to, I can
> keep my copy of bridle up to date, but that's not a requirement at all. Project takeover would
> verify all of that.
>
> If the project doesn't exist:
>
> 1. Set up the workspace.
> 2. Get everything going.
> 3. Enroll.
> 4. Notify all the demons.
> 5. Notify all the machines.
> 6. Mint all the tokens, assuming we have permissions, of course.
> 7. Handle the takeover.
>
> The takeover should be an agreed-upon thing, a coordination between the two machines, unless
> we're forced to override it. That would only happen if a machine is unavailable due to a crash,
> loss of data, or loss of the machine or something. We should always have a way to forcibly take
> that over. If that's restricted to the human, that is totally fine. Seems reasonable.
>
> You made a comment about one record of which machine runs each project. I think we needed one
> record kept in sync, but it's okay. I think for now we don't have a design that would work to
> mandate that, because I think the owner.toml file is part of the project repo. I think that is a
> check on the project. I think which project, which machine owns a project, lives in each
> machine's config as well as in the project repo. Those two things just need to be kept in sync
> when we do a takeover.

What this changes, as the PdM reads it:

- "Everywhere" is out: each machine sets up only the projects chosen for it, and bridle keeps
  track of which projects are set up on which machine.
- One idempotent command sets up a project on a machine, given the workspace parent folder. On a
  machine that already has it, the command checks and repairs the setup and runs any migrations
  or updates. The human or any agent can run it; it needs no orchestrator.
- Checking out a project, or bringing a checkout up to date, is a separate operation from
  takeover. GitHub is the source of truth; the owning machine pushes through the workflow.
- For a new project: set up the workspace, start it, enroll, tell every daemon and every machine,
  mint the tokens (where permitted), then take over.
- Takeover is agreed between the two machines. A forced takeover (the owner crashed, lost its
  data or is gone) always exists, and may be restricted to the human.
- Ownership lives in each machine's config and in the project repo (`owner.toml`). Takeover keeps
  the two in sync; no single record is mandated for now.
