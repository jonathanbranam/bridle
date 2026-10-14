---
id: br-dffz
title: "Onboard a project in one command: a git repo or nothing, to fully working on every machine"
kind: epic
created: 2026-10-14
created_by: external:advisor/product-manager
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-29se, br-kf7t, br-g9qm, br-afzu, br-rjd5, br-n63z, br-8uxe, br-63rv]
---

## The ask

The human, 2026-10-14 ~2:00 PM ET (to advisor product-manager), verbatim:

> i added a project to remus and have some goals for this in the future.
>
> Scenario:
>
> - I have a git repo, want to onboard to bridle and set up as a new bridle project
> - I don't have anything (maybe a bare repo) and want to start a new bridle project
>
> There are slight differences in these, but for each one, a (preferably single command, it can call other commands if needed, should be safe to call twice, idempotent):
>
> 1. Add the project to bridle config locally
> 2. Sync config between machines
> 3. bridle start the project (create bridle state)
> 4. work on main or optionally bridle-adopt (for existing projects)
> 5. sync, pair, mint all tokens
> 6. set up mail
>
> At the end, everything should be ready to go - agents, the cluster, mail, messaging, gateway, auto upgrade, everything should be working for the project.

Earlier the same day (to advisor wsl2, br-29se): "setting up a new project should be very smooth. I also want a command that adds a new project to the local config and then syncs to other machines." And 2026-10-08 (rjd5): "we've just got to build commands to do this project setup stuff; wayyy too much to keep handling by hand."

## Outcome

One command takes a project from "a git repo" or "nothing" to fully working on bridle, on this
machine and known to every other: safe to run again, and when it finishes, agents, the cluster,
mail, messaging, the gateway and auto-upgrade all work for the project with no hand edits.

## Done when (PdM draft)

For both scenarios (an existing repo; nothing or a bare repo), one command, run twice, leaves the
project: in `[projects]` on every machine (29se), with its owner and `bridle/state` (kf7t), on
`main` or `bridle-adopt` as chosen, with every token minted, failed 0 (g9qm), enrolled in mail
(afzu), its daemon a service that self-upgrades, and reachable from the gateway. Proven by setting
up one real project each way with no hand step. `docs/context/adding-a-project.md` becomes "run
this command".

## Child tickets

29se (projects add: register and sync), kf7t (start a brand-new project), g9qm (token pair on a
machine's first project), afzu (every project in mail by default). Related: rjd5 (one idempotent
setup command and sync-all; the earlier ask, folded into this epic's design), n63z (one command
mints every peer token), 8uxe (home projects, the first users), 63rv (trial adoption on
bridle-adopt).

## Decided (the human, 2026-10-14)

- One command, `bridle project setup` (name open to the design), not separate `new` and `adopt`.
  The human, ~3:15 PM ET: "yes agree; any commits at all, it prompts and asks; also a CLI param
  can specify which".
  - No repo, or a repo with no commits: a new project on `main`, no question.
  - Any commits at all (a README, some documents, real history): it asks "work on `main` or on
    `bridle-adopt`?", default `bridle-adopt` (rule existing-projects). A flag answers it up front
    (`--main` / `--adopt`); `-y` takes the default. Bridle never decides from the contents that
    a repo is throwaway. (The human had asked what "bare" means: no commits, only a README, just
    some documents? This rule makes the question unnecessary.)
- Ports stay as they are. The human, ~3:10 PM ET: "let's not worry about the ports then ...
  keep it as is; i dno't think it will climb too high and I dn't think i'll transfer projects
  enough for it to matter". New projects take the next port unused on any machine (29se);
  take-over keeps its bump-on-clash.

## For the design (the human, 2026-10-14 ~3:20 PM ET)

Verbatim:

> I don't have gh repo create permissions on my PAT tokens RN, so I think we provide the remote; and yes things to specify or somehow decide:
>
> 1. what is the workspace root folder? I prefer slightly shorter so 'my-proj' goes in '~work/my-proj-work/my-proj' is this asked or assumed from the cwd/pwd?
> 2. the remote (if a brand new project)
> 3. picking up after a failed step may be difficult, but a goal; it should inspect the state of the system at each point as much as possible
> 4. I don't mind if there are several commands to run, but they should be sequenced in a meaningful way and should indicate the next command in their output: "to continue setting up, run bridle mail install" or something; IDK; we need a thoughtful design, but in general, if there is an existing bridle setup on the machine, most things should be answered by that setup.

So the design settles:

- **No `gh repo create`**: the human provides the remote for a brand-new project (their PAT
  can't create repos).
- **Workspace root**: the human's preferred layout is `~/work/<p>-work/<p>`. Asked, or taken from
  the current directory, or from this machine's existing projects?
- **Resuming**: each step inspects the system's real state (config, `bridle/state`, daemon, tokens,
  mail) and does only what is missing; a second run picks up where the first stopped.
- **One command or a sequence**: several commands are fine if sequenced meaningfully, each
  ending with the next one to run ("to continue setting up, run `bridle mail install ...`"). This
  softens the earlier "one command": `bridle project setup` may be the entry point that runs or
  names the rest.
- **Defaults from the machine**: when the machine already has a bridle setup, most answers
  (workspace root, machine name, peers, mail, ports) come from it; ask only what it can't infer.

Also open: what else it asks when interactive (aide token, mail; rjd5's "[Y/n]" prompts), `-y`
for all.
