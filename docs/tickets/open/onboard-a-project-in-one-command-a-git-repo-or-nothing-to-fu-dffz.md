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

## Design options

Focus: interface (the commands and what they print). Written by designer dffz, 2026-10-14.

The problem in a sentence: from "a git remote" to "fully working project on every machine" is
about eight hand steps today, and a failure halfway leaves the human guessing which are done.

### What exists today

| Step | Today |
|---|---|
| clone, `bridle init` | by hand; `init` writes `.bridle/config.toml` + `.gitignore`, never touches `~/.bridle` (`docs/context/adding-a-project.md`) |
| register in `[projects]`, pick a port | by hand, then `bridle projects sync --apply` (29se adds `projects add`) |
| `bridle/state`, owner | `project start` refuses a new project, `take-over` refuses too (kf7t) |
| daemon as a service | `bridle project start` (installs unit, starts, runs `token pair`) |
| tokens | `token pair`; fails on a machine's first project (g9qm); one command for all (n63z) |
| mail | `[mail] projects` by hand on each machine (afzu); `bridle mail install` per project |
| gateway, auto-upgrade | not checked here; the plan must verify what, if anything, a new project needs |

Near-duplicates to respect (rule one-name-per-action): `projects add` (29se, register only),
`project start` (service + tokens), `token pair`, `mail install`. `bridle project` and
`bridle projects` are already one group. None of these should be renamed or copied; the new
command calls them.

### What the user types and sees (all options)

Two inputs cover both scenarios. A **remote URL** the human made (their PAT cannot
`gh repo create`; bridle never creates a repo), or an **existing clone**. "Nothing" and "bare
repo" are the same case: an empty remote. The only fork is whether the repo has commits, and the
human already decided that rule (commits: ask main or `bridle-adopt`, default adopt).

```
$ bridle project setup git@github.com:jonathanbranam/my-proj.git
workspace root [/Users/jbranam/work/my-proj-work]:        <- Enter
  clone ... ok (empty repo: new project on main)
  init ... ok
  register my-proj on dalek, port 7409, on dalek vesta remus ... ok
  bridle/state + owner dalek ... ok
  daemon service ... ok (self-upgrades)
  tokens (token pair) ... minted 9, kept 0, failed 0
  mail ... ok (bridge installed; my-proj in [mail] projects on 3 machines)
Create aide token [Y/n]? y ... ok
Done: my-proj is fully set up. Re-run any time; it only does what is missing.
```

An existing local clone: run `bridle project setup` inside it (or `setup <name>`); the clone is
used where it is, never moved. With commits it asks `work on main or bridle-adopt? [adopt]`;
`--main` / `--adopt` answer up front. `-y` takes every default; it cannot invent a remote, so a
new project with `-y` and no remote exits 2 naming `--remote`/the URL. With no terminal and no
`-y` it never waits: it lists the missing answers and exits 2.

### Option A: one command that does everything (`bridle project setup`)

Setup is a driver over the steps above. Each step has a `check` (read real state) and an
`apply`; setup runs `check` on all, shows the checklist, applies the unmet ones in order, and
stops at the first failure with the exact fix and "run the same command again".
- Cost: one new module with the step table; steps stay the existing code. State is read, never
  stored, so a second run picks up where the first stopped (the human's point 3).
- Short: if a step is wrong the human has no reason to learn the step commands; fine, they exist
  anyway. A long run is hard to test as a whole (test each step's check/apply).

### Option B: a chain of small commands, each naming the next

No driver. `projects add` ends "next: bridle project start", `start` ends "next: bridle mail
install ...", and so on. The "next" line is computed from real state, not hardcoded.
- Matches the human's point 4 literally; no big command to build.
- Falls short: the human types six commands for every project (the thing they called "wayyy too
  much"); the chain order lives in six places and drifts (violates modularity: each command now
  knows its successor). Resuming means remembering where you were.

### Option C: A plus B (recommended)

`bridle project setup` is the entry (Option A). The same state inspector prints the "next:" line
at the end of each step command when run alone (`projects add`, `project start`, `token pair`,
`mail install`): "next: bridle project token pair --projects my-proj". One inspector, one step
order, two ways in. Also `bridle project setup --check`: the checklist only, changes nothing
(replaces a separate doctor-for-new-projects).
- Cost: A's module plus a one-line hook in four commands.
- Meets all four design points. Short: the "next" hook is a small coupling of those commands to
  the inspector; keep it a function call, not a framework.

### Option D: a declarative project file (`.bridle/project.toml` + `bridle apply`)

Rejected. A second source of truth beside `[projects]`, `bridle/state` and `config.toml`, for a
need one person with a few projects does not have (yagni). Reality inspection (C) gives
resumability without a file.

### Option E: docs only (do nothing)

Honest but rejected: the human said no more hand setup; the three bugs (kf7t, g9qm, afzu) and
the hand-edited `[projects]` are what keep costing hours.

### The four open design points, answered (applies to C)

**1. Workspace root (`~/work/<p>-work/<p>`).** Asked, with a default shown, never silently taken
from the cwd. Order of answers: `--root DIR`; else, if the cwd is inside a clone, that clone's
own place (nothing moves); else a proposed default, confirmed with Enter (`-y` takes it). The
default is `<base>/<p>-work/<p>`, where `<base>` is the parent of the workspaces of this
machine's existing projects (the daemon registry in `~/.bridle/daemons/*.json` records each
`workspace` and `repo`; take the most common parent) or `~/work` on a machine with none. The
`-work` suffix is fixed: this machine today mixes `-workspace` (bridle-ui, track-web) and plain
`/Volumes/Data/work/bridle`, so "copy the majority" would copy an accident. Rejected: cwd as
the parent (running from `~` or `/tmp` puts a project there silently); a new `[machine]
work_dir` key (yagni until the inference is wrong on a real machine).

**2. The remote.** Given, as the positional URL or `--remote`, or read from the clone's
`origin`. A new project without one is an error that says "make an empty repo and pass its
URL". No `gh repo create`, ever (human's decision). The remote is checked reachable before
anything is changed. A non-empty remote takes the main/adopt question.

**3. Resuming.** No progress file. Each step reads the truth: clone present with matching
origin; `.bridle/config.toml`; `[projects]` entry on each machine (`projects sync` compare);
`bridle/state` on origin and `owner.toml` names this machine; unit installed and daemon
answering; `token pair` reports failed 0; mail unit and `[mail] projects`; aide token. Main
versus adopt is also read, not remembered: being on `bridle-adopt` means adopt. A step that
cannot tell (machine unreachable) is reported "unknown", not "done", and the run exits non-zero.
This is a goal, not a promise for every half-failure; destructive repair is never automatic.

**4. Sequence and defaults from the machine.** The order is fixed: clone, init, register (port
and mail enrollment, synced), state and owner, service, tokens, mail, aide token. Register is
before start because the daemon needs its port; tokens after start because minting needs the
daemon. Everything inferable is inferred and printed, not asked: machine name (`[machine]
name`), peers (`[machines]`), port (next unused in `[projects]` on any machine, 29se), mail
(on unless `[machine] mail = false`, afzu), integration branch (the branch HEAD is on). Asked
only: remote (if none), workspace root (confirm), main/adopt (if commits), aide token `[Y/n]`.
There is no per-project mail question: afzu says every project is enrolled.

### Against the principles

- KISS/YAGNI: C adds one module and no file or key; D and a `work_dir` key fail yagni.
- One name per action: `setup` is the only new name; `projects add` stays as "register only"
  (29se), which is a different action. Say so in its help to avoid reading as a duplicate.
- Modularity: steps stay in their commands; setup couples to them only through check/apply.
- User's side: the transcript above is the design; the human types one line.

### Recommendation

Option C. Accepts: a driver that is only as good as its step checks (a wrong "done" hides a
gap, so `--check` is the tool to trust-but-verify); the "next:" hook in four commands.

### Not yet known (decide at planning, from the next real project)

- Whether the gateway and auto-upgrade need any per-project step, or follow from `[projects]` and
  the service unit. The ticket says both must work; no step is designed until checked.
- Whether setup may run for a project owned by another machine (`--machine`, over ssh as
  `project start` does) or only on the owning one. Recommend only on the owner at first.
- Whether `bridle projects` or `bridle project` is the spelling to document. Both work today.

## Design approved (the human, 2026-10-14 ~6:15 PM ET)

Verbatim: "that looks really good. And yes, my standard is `$WORK_ROOT/<proj-name>-work/<proj-name>`"

- Option C approved as the direction. Still backlog: no planning or build until the human picks
  it up ("we'll learn more as I create more new projects and refine it as we go").
- Workspace layout is the human's standard: `$WORK_ROOT/<proj-name>-work/<proj-name>`. Where
  `$WORK_ROOT` comes from on each machine (an environment variable, inferred from this machine's
  projects as the design proposes, or a setting) is for planning.
- The human, ~6:20 PM ET, verbatim: "But the system should work with any folder name in there,
  but that can be the default; it should prompt for confirmation and override before writing the
  path anyway". So `$WORK_ROOT/<proj-name>-work/<proj-name>` is only the default: any path works
  (bridle never assumes the layout elsewhere), and setup always shows the path and asks to
  confirm or override it before writing anything (`--root` / `-y` answer it up front).
