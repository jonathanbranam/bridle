# Roadmap

Status: working notes, kept by hand by advisor (product-manager) during the PdM trial
(`docs/notes/product-manager-trial.md`). Treat it as a database: it changes whenever a task is
delivered, planned, held or dropped, or a workstream pauses. Later this becomes a bridle feature.

Terms (the human, 2026-10-09): the **roadmap** orders epics, grouped by theme for reading. An
**epic** is a planned group of work with an outcome and a "Done when", tagged to one theme. A
**theme** is a lasting area (slug, no ID, may never finish); epics and loose tasks are tagged to
one. Until 22ab adds `theme:` and `parent`, the grouping lives only here. Outcome and "Done when"
lines marked "(PdM draft)" wait on the human.

How to read the tables: **State** is the task's state in bridle. **Pri** is its priority. **Next** is what has to happen
before it moves: "waits on the human" is a pending task only the human can approve; "to plan"
is the project manager's; "ready to build" is in the queue; "HELD" is planned in bridle but
stopped by the human.

Last updated: 2026-10-09 19:15 ET.

## Needs the human now

In the order the PdM suggests:

0. **Machine setup** (your priority, 2026-10-09): two decisions unblock phase 2. (a) done 2:20 PM: sk7p
   design approved; 8c25 being re-briefed. (b) j7r4: answered 2:05 PM (one pusher, enforced:
   8z7j to the designer; k6jd, 8ay6, 8umh readied; xccp later). Also your review of kuw2
   (machine daemon, br-efs2) gates project transfer's design.

1. **The designer's first job**: by your ladder it is the attachments design (step 2 below),
   then the scheduler design for the rest of yfv5 (nightly restarts, maintenance windows,
   machine-wide), with the shipped slice (9xze) written up after the fact. ukpm planned fne2 as
   its first job. Attachments first, then yfv5, then fne2?
2. **Attachments**: file the ticket now (several documents and later images per ticket or
   task, the MIME idea), so the designer has something to design?
3. **br-g5y2**: approve role priming for scheduled messages (agents wait at the maximum
   timeout and schedule their own wake-ups). Its mechanism, 9xze, has landed.
4. **95mu** (change specs reviewed before build): review the ticket so it can go to the
   designer. It is the general form of the scheduler ladder below.
5. Held for your review already: **gtzx** (seats, P1-P10, Q1-Q4) and **stx8** (task states,
   needs a proposed design).
6. From the bridle aide: max_workers 2 -> 3 (the aide recommends no until the v6kr baseline);
   close br-a3b9?
7. The new epic `migrations`: its "Done when" is a PdM draft; amend or approve.

## The scheduler ladder (the human's sequence, 2026-10-09)

Goal: ship scheduled messages, with design and specs approved before build. In build order:

| Step | What | Status |
|---|---|---|
| 1 | A design role | Delivered: designer role, br-ukpm (2026-10-09) |
| 2 | The attachments design, written into the ticket body (no attachments yet) | Not started; waits on decisions 1 and 2 above |
| 3 | The human reviews, amends and approves the attachments design | - |
| 4 | Specs for attachments, from the approved design | - |
| 5 | Attachments on tickets and tasks (MIME approach), built | No ticket (decision 2) |
| 6 | The scheduler design (the rest of yfv5), attached to its task | - |
| 7 | The human approves the scheduler design | - |
| 8 | Specs on the task, from the approved design | - |
| 9 | The system scheduler | Slice 1 (per-project scheduled messages) delivered early: br-9xze, 2026-10-09, built from decisions the orchestrator wrote and the human approved 10-08 |

## Epics in order

The roadmap's priority order. Machine setup first (the human, 2026-10-09).

1. **Machine setup** (`machine-setup`, theme `multi-machine`)
2. **Everything is a ticket** (`everything-is-a-ticket`, theme `tickets-and-release`)
3. **Migrations** (`migrations`, theme `tickets-and-release`)
4. **Scheduled messages and timed actions** (`scheduled-messages`, theme `agents-and-cli`)
5. **Documents and attachments on tickets** (`attachments`, theme `product-process`)
6. **Reviews enforced by bridle** (`reviews-enforced`, theme `product-process`)
7. **Products: several product managers and roadmaps** (`products`, theme `product-process`)

## Themes

### Theme `multi-machine`: Many projects, many machines

Daemons, tokens, sync and projects across the laptop, the NUC and the Windows PC.

#### Epic `machine-setup`: Machine setup

- Outcome: A new machine (first the Windows PC) is set up directly: background daemons, tokens between machines and projects, sync. (The human, 2026-10-09.)
- Done when: Phase 1: the PC runs bridle unattended. Phase 2: one command sets up and syncs a machine. Phase 3: a project moves between machines.

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-jgdb](http://dalek.tailbc91f5.ts.net:7878/task?id=br-jgdb) | Windows PC: follow the WSL2 setup guide (install WSL2 + Ubuntu, wslconfig, Tailscale, s... | low | claimed | the human's to-do |
| [br-h7mu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-h7mu) | Pick a name for the Windows PC (docs/context/naming.md) (v7ug) | low | claimed | the human's to-do |
| [br-hua2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-hua2) | Add a machine: one setup guide from bare OS to on the network (config, tokens, services... | high | integrated | delivered |
| [br-gdf3](http://dalek.tailbc91f5.ts.net:7878/task?id=br-gdf3) | Peer-token setup guidance: a token per receiving project per sending machine, minted on... | high | integrated | delivered |
| [br-8c25](http://dalek.tailbc91f5.ts.net:7878/task?id=br-8c25) | bridle token pair, part 1: the token-role list, the spec, and role tokens across machin... | high | planned | approved to land (the human, 6:05 PM ET); lands after br-ngya turns main green |
| [br-jw9e](http://dalek.tailbc91f5.ts.net:7878/task?id=br-jw9e) | bridle token pair, part 2: peer tokens, the [mail] peers opt-out, and pairing on projec... |  | planned | ready to build |
| [br-88d4](http://dalek.tailbc91f5.ts.net:7878/task?id=br-88d4) | self_upgrade = "release": fetch, verify and swap the release binary (chvf 2) | high | planned | ready to build |
| [br-751e](http://dalek.tailbc91f5.ts.net:7878/task?id=br-751e) | Daemon keeps its own workflow checkout at the binary's tag (chvf 3) | high | planned | ready to build |
| [br-57nt](http://dalek.tailbc91f5.ts.net:7878/task?id=br-57nt) | bridle gateway restart takes a launchd-managed gateway out of launchd and inherits the ... | high | integrated | delivered |
| [br-xrkh](http://dalek.tailbc91f5.ts.net:7878/task?id=br-xrkh) | systemd uninstall, and an owner refusal never crash-loops a launchd or systemd unit aft... | high | integrated | delivered |
| [br-rjd5](http://dalek.tailbc91f5.ts.net:7878/task?id=br-rjd5) | Commands to set up and sync a project everywhere: sync all seven places per machine, an... |  | pending | phase 2: needs a design (source of truth for the project list), then the human's approval |
| [br-f8f9](http://dalek.tailbc91f5.ts.net:7878/task?id=br-f8f9) | The NUC recovers everything on boot (4r3k) |  | pending | phase 2: 'bridle up' after boot needs a small design |
| [br-v7ug](http://dalek.tailbc91f5.ts.net:7878/task?id=br-v7ug) | Run bridle's heavy work on the Windows PC under WSL2 |  | pending | umbrella; audit and guide delivered |
| [br-kt25](http://dalek.tailbc91f5.ts.net:7878/task?id=br-kt25) | Move a project between machines with one command (bridle project move) |  | pending | phase 3: needs a design and the human's decisions (see ticket) |
| [br-8z7j](http://dalek.tailbc91f5.ts.net:7878/task?id=br-8z7j) | Only the owner's clone can push the integration branch: enforced, not a rule |  | reopened | Option A approved; blocked until dotfiles-local shared git hooks are removed on both machines (dotfiles-local aide, NUC) |
| [br-k6jd](http://dalek.tailbc91f5.ts.net:7878/task?id=br-k6jd) | Managers and the orchestrator fetch origin; divergence from origin is warned (N ahead, ... | high | planned | ready to build |
| [br-xccp](http://dalek.tailbc91f5.ts.net:7878/task?id=br-xccp) | A git identity per machine, so commits show which clone made them | low | pending | later (the human: non-urgent); needs the human's keys or tokens |

#### Not in an epic

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-1ddd](http://dalek.tailbc91f5.ts.net:7878/task?id=br-1ddd) | One watcher for every project: 'bridle agent wake --all-projects' |  | planned | ready to build |
| [br-kuvh](http://dalek.tailbc91f5.ts.net:7878/task?id=br-kuvh) | Remove 'bridle agent wake --all-projects' once 3haz lands and rolls out: warn first, th... |  | pending | waits on the human (approve to ready) |
| [br-n7cg](http://dalek.tailbc91f5.ts.net:7878/task?id=br-n7cg) | Mail between daemons, slice 2: mail for a visitor is forwarded to its home daemon (3haz... |  | pending | waits on the human (approve to ready) |
| [br-cufw](http://dalek.tailbc91f5.ts.net:7878/task?id=br-cufw) | Mail between daemons, slice 4: visible state: outbox status, message show (queued/arriv... |  | pending | waits on the human (approve to ready) |
| [br-3932](http://dalek.tailbc91f5.ts.net:7878/task?id=br-3932) | Run bridle on a project without a local bridle clone (mrhe) |  | pending | waits on the human (approve to ready) |
| [br-u6w9](http://dalek.tailbc91f5.ts.net:7878/task?id=br-u6w9) | Human interaction time: daemon serves the prompt log; gateway collects across machines ... |  | pending | waits on the human (approve to ready) |
| [ui-9hq8](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-9hq8) | Documents of projects on another machine (the NUC): read and comment in bridle-ui | high | planned | ready to build |

### Theme `tickets-and-release`: Tickets, changelog and release

How work is recorded (tickets, fields, migrations) and shipped (changelog, releases, self-upgrade).

#### Epic `everything-is-a-ticket`: Everything is a ticket

- Outcome: One record per piece of work: tickets, with a row for every ticket and the thread on the state branch; standard field names; project-qualified IDs. Ticket 22ab holds the decisions, design and plan.
- Done when: The code rename is finished too (the human: "must be completed as part of this workstream before it is considered complete").

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-syqn](http://dalek.tailbc91f5.ts.net:7878/task?id=br-syqn) | Ticket fields get standard names (blocked_by, related, parent, created, resolved), a th... |  | open | to plan (project manager) |
| [br-bpku](http://dalek.tailbc91f5.ts.net:7878/task?id=br-bpku) | Every ticket gets a row; types fix and epic; readiness moves to the ticket (ticket ready) |  | pending | after br-syqn lands (blocked_by) |
| [br-3v75](http://dalek.tailbc91f5.ts.net:7878/task?id=br-3v75) | Ticket links live in the frontmatter: the daemon indexes blocked_by, parent and related... |  | pending | after br-bpku lands |
| [br-72t9](http://dalek.tailbc91f5.ts.net:7878/task?id=br-72t9) | A ticket's thread moves to the ticket (tickets/<id>.md on the state branch); ticket com... |  | pending | after br-3v75 lands |
| [br-d9wq](http://dalek.tailbc91f5.ts.net:7878/task?id=br-d9wq) | Research: product manager roles, human and agentic (BMAD and others), and what to call ... |  | pending | research done; the human reads it and picks the term |
| [br-95mu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-95mu) | A change spec (proposal and design) reviewed for risk and impact before any worker buil... |  | pending | to the designer once the human has reviewed the ticket |

#### Epic `migrations`: Migrations

- Outcome: Every bridle project's tickets and tasks are brought to the current format by migrations that run on their own at start-up (xebc, br-2718 landed). The human, 2026-10-09: "this would be a great new epic to have! migrations!"
- Done when: (PdM draft) Every open ticket in every project has a kind and two-way task links; `bridle ticket check` is clean in each project; later format changes (22ab) ship as migrations here.

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-e7e2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-e7e2) | Migration: backfill ticket kind and two-way task links in every bridle project (v3dk sl... |  | planned | ready to build |

#### Not in an epic

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-g3az](http://dalek.tailbc91f5.ts.net:7878/task?id=br-g3az) | Status line token setup in the docs writes an empty file: token create needs --print now |  | integrated | delivered |
| [br-01ff](http://dalek.tailbc91f5.ts.net:7878/task?id=br-01ff) | Tickets through the bridle binary in every project: new, frontmatter, check, resolve; a... |  | pending | waits on the human (approve to ready) |
| [br-ubjd](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ubjd) | A CHANGELOG line for every landed task, written on the branch, and one section per kind... |  | pending | waits on the human (approve to ready) |

### Theme `agents-and-cli`: Agents and the CLI

Agents use bridle correctly: commands, help, roles, the workflow reaching every agent, timed actions.

#### Epic `scheduled-messages`: Scheduled messages and timed actions

- Outcome: Agents and the human schedule messages and timed actions through one scheduler (yfv5).
- Done when: Nightly restarts, maintenance windows and machine-wide schedules run from the one scheduler, designed and specced before build.

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-9xze](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9xze) | Scheduled messages, first slice: an agent schedules a message to itself (one-time or re... |  | integrated | delivered |
| [br-g5y2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-g5y2) | Scheduled messages slice 2: role priming: wait at the maximum timeout and schedule a me... |  | pending | waits on the human; 9xze has landed |
| [br-yfv5](http://dalek.tailbc91f5.ts.net:7878/task?id=br-yfv5) | One scheduler for timed actions: scheduled messages (hrcn), nightly session restarts (c... |  | pending | needs a design (designer) for the rest: cbbn, 3nyk, cy2v; then the human's review |
| [br-cbbn](http://dalek.tailbc91f5.ts.net:7878/task?id=br-cbbn) | Scheduled nightly restart of an interactive session at a clock time (e.g. 3 AM) |  | pending | waits on the human (approve to ready) |
| [br-ft3b](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ft3b) | Per-role handover instructions in the workflow, with project overrides |  | pending | waits on the human (approve to ready) |

#### Not in an epic

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-3mz4](http://dalek.tailbc91f5.ts.net:7878/task?id=br-3mz4) | One orchestrator per machine, not per project: say so in the advisor role and wherever ... |  | open | to plan (project manager) |
| [br-4cgx](http://dalek.tailbc91f5.ts.net:7878/task?id=br-4cgx) | @-mention a role in a ticket reply or a document comment and that role gets a message | low | pending | waits on the human (approve to ready) |
| [br-163f](http://dalek.tailbc91f5.ts.net:7878/task?id=br-163f) | Group the CLI's 54 top-level commands; split commands.rs/cli.rs by group (a67t) |  | reopened | reopened |
| [br-ts6b](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ts6b) | One command tree for interactive sessions: bridle session <verb> <seat>, retiring bridl... |  | pending | waits on the human (approve to ready) |
| [br-fne2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-fne2) | bridle session advisor and bridle advisor start: near-identical commands that do differ... |  | pending | the designer's first job (ukpm) |
| [br-abnq](http://dalek.tailbc91f5.ts.net:7878/task?id=br-abnq) | One command for any agent's status and context |  | pending | waits on the human (approve to ready) |
| [br-m9sd](http://dalek.tailbc91f5.ts.net:7878/task?id=br-m9sd) | Agents don't know how to use the bridle CLI correctly: help, skills, shorter primes or ... |  | pending | waits on the human (approve to ready) |
| [br-a9g2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-a9g2) | agent spawn --allow-tool silently does nothing for a tool outside the role's --tools se... |  | pending | waits on the human (approve to ready) |
| [br-79c3](http://dalek.tailbc91f5.ts.net:7878/task?id=br-79c3) | Reserve role names, by prefix, for agent names (2vja) |  | planned | ready to build |
| [br-9z2d](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9z2d) | Incident: the orchestrator didn't use bridle advisor start or per-advisor addresses, an... |  | pending | waits on the human (approve to ready) |
| [br-86c6](http://dalek.tailbc91f5.ts.net:7878/task?id=br-86c6) | The workflow doesn't reach agents: resolved rules, hooks and overrides stop at the CLI |  | pending | waits on the human (approve to ready) |
| [br-h3ar](http://dalek.tailbc91f5.ts.net:7878/task?id=br-h3ar) | Interactive sessions kill each other's wake waiters with pkill -f (exit 144) |  | pending | waits on the human (approve to ready) |
| [br-mvtz](http://dalek.tailbc91f5.ts.net:7878/task?id=br-mvtz) | Postmortem: sessions killed each other's wake waiters with pkill -f (h3ar) |  | pending | waits on the human (approve to ready) |
| [br-qdw8](http://dalek.tailbc91f5.ts.net:7878/task?id=br-qdw8) | A haiku worker reported done before its check finished, then lost its task on renewal a... |  | pending | waits on the human (approve to ready) |
| [br-ytqu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ytqu) | Postmortem: a haiku worker waited on a check it couldn't see, reported done early, and ... |  | pending | waits on the human (approve to ready) |

### Theme `product-process`: Product process and gates

How work gets from an idea to ready: the PdM, the designer, change specs, reviews, task states.

#### Epic `attachments`: Documents and attachments on tickets

- Outcome: Several documents (a design, specs) and later images on one ticket; the human's MIME parts idea.
- Done when: A ticket carries a design and specs as parts, and the designer writes into them.

No tasks yet.

#### Epic `reviews-enforced`: Reviews enforced by bridle

- Outcome: Reviews are required, signed on the work and enforced before landing (v2va).
- Done when: No landing until every required review is signed (slice D).

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-91b3](http://dalek.tailbc91f5.ts.net:7878/task?id=br-91b3) | Reviews enforced by bridle, signed on the task (v2va): umbrella, slices A-D | low | planned | ready to build |
| [br-6ba3](http://dalek.tailbc91f5.ts.net:7878/task?id=br-6ba3) | Reviews A: in_review state, ready-for-review, review requirements on tasks (v2va slice A) | low | planned | ready to build |
| [br-46fe](http://dalek.tailbc91f5.ts.net:7878/task?id=br-46fe) | Reviews B: signed review records on the task (v2va slice B) | low | planned | ready to build |
| [br-cf00](http://dalek.tailbc91f5.ts.net:7878/task?id=br-cf00) | Reviews C: the daemon spawns the required reviewers; comment-only talk; cost recorded (... | low | planned | ready to build |
| [br-f610](http://dalek.tailbc91f5.ts.net:7878/task?id=br-f610) | Reviews D: the gate: no landing until every required review is signed (v2va slice D) | low | planned | ready to build |

#### Epic `products`: Products: several product managers and roadmaps

- Outcome: Each product (a set of projects, 1:1 with GitHub repos, sharing one roadmap and one product manager; the human, 2026-10-09) has its own PdM and roadmap, so neither the agent's context nor the human's mixes unrelated products.
- Done when: There are multiple product managers, each responsible for a product that contains many projects; a product roadmap lives in a single place (probably a project), but its planning spans multiple projects. (The human.)

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-g5dm](http://dalek.tailbc91f5.ts.net:7878/task?id=br-g5dm) | Products: a product is a set of projects with one roadmap and one product manager; seve... | low | pending | needs a design (designer), then the human's review; keep it flexible while the trial runs |
| [br-6h65](http://dalek.tailbc91f5.ts.net:7878/task?id=br-6h65) | A product manager that relates every ticket to open and planned work: links, merges, an... |  | pending | this trial is the stand-in (docs/notes/product-manager-trial.md) |

#### Not in an epic

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [ui-vhrb](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-vhrb) | A theme page (everything about one theme) and a roadmap page (epics in order) in the UI | low | pending | waits on the human (approve to ready) |
| [br-gtzx](http://dalek.tailbc91f5.ts.net:7878/task?id=br-gtzx) | Seats: every role is a named, tracked seat that outlives its sessions, with its own inb... |  | open | HELD: waits on the human's review of P1-P10, Q1-Q4 |
| [br-stx8](http://dalek.tailbc91f5.ts.net:7878/task?id=br-stx8) | A task's state says what's really happening: held and built-awaiting-landing are states... |  | planned | HELD: needs a proposed design and the human's approval |
| [br-519b](http://dalek.tailbc91f5.ts.net:7878/task?id=br-519b) | Task watchers: a creator field, a watchers list, and wakes that say what changed and ar... |  | pending | waits on the human (approve to ready) |
| [br-cr7t](http://dalek.tailbc91f5.ts.net:7878/task?id=br-cr7t) | Add a postmortem ticket kind: the full write-up after an incident |  | pending | waits on the human (approve to ready) |
| [br-avu7](http://dalek.tailbc91f5.ts.net:7878/task?id=br-avu7) | vk3y slice 2: 'bridle task new' requires --ticket (no-ticket sentinel) and a rule for e... |  | pending | waits on the human (approve to ready) |

### Theme `reliability`: Reliability

The orchestrator and daemons stay up, relaunch once, upgrade cleanly and recover after sleep or reboot.

#### Not in an epic

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-8a53](http://dalek.tailbc91f5.ts.net:7878/task?id=br-8a53) | main red (ubuntu): gateway_test a_replaced_binary_is_re_executed; INVOCATION_ID misdete... | critical | integrated | delivered |
| [br-ngya](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ngya) | Flaky on Linux CI: upgrade_test a_drain_starting_during_a_spawn_restarts_promptly reads... | critical | integrated | delivered |
| [br-rztb](http://dalek.tailbc91f5.ts.net:7878/task?id=br-rztb) | Incident: something keeps restarting dalek's gateway outside launchd from a Claude sess... | high | integrated | delivered |
| [br-4zfa](http://dalek.tailbc91f5.ts.net:7878/task?id=br-4zfa) | Incident: the bridle orchestrator was killed (SIGTERM) at 11:18 PM ET and nothing relau... | high | pending | waits on the human (approve to ready) |
| [br-f4xu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-f4xu) | Flaky on macOS CI: process_test sigterm_via_signal_group_exits_143 exits 1, not 143 |  | integrated | delivered |
| [br-h7gt](http://dalek.tailbc91f5.ts.net:7878/task?id=br-h7gt) | Flaky on Linux CI: upgrade_test self_upgrade_restarts_only_after_the_mid_turn_agent_fin... | critical | integrated | delivered |
| [br-8ay6](http://dalek.tailbc91f5.ts.net:7878/task?id=br-8ay6) | Direct-to-main docs commits are pushed straight after, on the owner's clone |  | planned | ready to build |
| [br-8umh](http://dalek.tailbc91f5.ts.net:7878/task?id=br-8umh) | A failed push is an event and an alarm; an agent that can't send puts the blocker on th... |  | planned | ready to build |
| [br-vabu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-vabu) | Flaky on Linux CI: store cancelled_blocking_task_returns_shutting_down_not_a_panic | critical | integrated | delivered |
| [br-b6mu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-b6mu) | Self-upgrade drain may never restart when it starts during a spawn |  | integrated | delivered |
| [br-6b8a](http://dalek.tailbc91f5.ts.net:7878/task?id=br-6b8a) | Orchestrator relaunch liveness: one clock, never a second orchestrator (jf9u) | high | planned | ready to build |
| [br-96a6](http://dalek.tailbc91f5.ts.net:7878/task?id=br-96a6) | Hold the orchestrator relaunch without restarting the daemon (8fsx) |  | pending | waits on the human (approve to ready) |
| [br-9966](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9966) | bridle orchestrator hold / release: runtime switch for the relaunch (8fsx) |  | planned | ready to build |
| [br-10c0](http://dalek.tailbc91f5.ts.net:7878/task?id=br-10c0) | Orchestrator identity and disaster recovery (7d62) |  | pending | waits on the human (approve to ready) |
| [br-btdn](http://dalek.tailbc91f5.ts.net:7878/task?id=br-btdn) | Daemon restart and self-upgrade blocked forever: manager.spawning() stays true with no ... |  | pending | waits on the human (approve to ready) |
| [br-y455](http://dalek.tailbc91f5.ts.net:7878/task?id=br-y455) | Incident: bridle's daemon couldn't restart or self-upgrade for ~23 h: a stuck 'spawning... |  | planned | ready to build |
| [br-aqa7](http://dalek.tailbc91f5.ts.net:7878/task?id=br-aqa7) | Self-upgrade refused good builds 3 times overnight: the new binary's self-check timed o... |  | pending | waits on the human (approve to ready) |
| [br-5j35](http://dalek.tailbc91f5.ts.net:7878/task?id=br-5j35) | bridle session restart run from inside the session stops it and never relaunches: the d... |  | pending | waits on the human (approve to ready) |
| [br-s5ah](http://dalek.tailbc91f5.ts.net:7878/task?id=br-s5ah) | A daemon without an aide leaves its orchestrator no route to the human |  | pending | waits on the human (approve to ready) |
| [br-2ax5](http://dalek.tailbc91f5.ts.net:7878/task?id=br-2ax5) | Incident: br-3haz broke every cross-project message for ~22 h: the CLI needed an outbox... |  | planned | ready to build |
| [br-zcqv](http://dalek.tailbc91f5.ts.net:7878/task?id=br-zcqv) | dalek slept in a bag 8:42 AM-1:38 PM ET on 10-06: bridle froze, but the workforce had a... |  | pending | waits on the human (approve to ready) |
| [br-vn54](http://dalek.tailbc91f5.ts.net:7878/task?id=br-vn54) | Incident: remote control dropped for the dalek aide session after a tmux detach and lap... |  | pending | waits on the human (approve to ready) |
| [br-ysmu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ysmu) | CI watch missed 13 red runs on main; first ci_failed wake came 40 minutes late |  | pending | waits on the human (approve to ready) |

### Theme `performance`: Performance and resource cost

Bridle's own cost on the machine: measure against a baseline, then cut it.

#### Not in an epic

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-n4w4](http://dalek.tailbc91f5.ts.net:7878/task?id=br-n4w4) | Postmortem: bridle's own 'ps' polling (every daemon, test daemons at 200 ms) drove dale... |  | pending | waits on the human (approve to ready) |
| [br-v6kr](http://dalek.tailbc91f5.ts.net:7878/task?id=br-v6kr) | A system architect role, and measuring bridle's own resource cost against a baseline |  | planned | ready; baseline must use the macOS memory measures (ticket) |
| [br-jxwr](http://dalek.tailbc91f5.ts.net:7878/task?id=br-jxwr) | Track and report how long a task takes from pickup to merge, split into agent work, bui... | low | pending | the human's ask (via aide); would show where the 30-40 min per task goes; low (the human): on a workstream, not queued now |
| [br-6nzj](http://dalek.tailbc91f5.ts.net:7878/task?id=br-6nzj) | Test daemons stop polling at 200 ms; a resource-budget test; log the incident in docs/c... |  | integrated | delivered |
| [br-g76s](http://dalek.tailbc91f5.ts.net:7878/task?id=br-g76s) | Load-hold notes: one per machine, name bridle-owned top consumers, honest text, load.ho... |  | planned | ready to build |
| [br-fzwa](http://dalek.tailbc91f5.ts.net:7878/task?id=br-fzwa) | Audit every periodic daemon loop for what it forks or reads per tick; list them with co... |  | planned | ready to build |
| [br-ks55](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ks55) | Only one full test run at a time per machine: just check takes a machine-wide lock (n4w... |  | planned | ready to build |
| [br-yw8b](http://dalek.tailbc91f5.ts.net:7878/task?id=br-yw8b) | fake-claude spawns skip the pyenv shim: resolve the interpreter once (n4w4 rec 7) |  | planned | ready to build |
| [br-z7y5](http://dalek.tailbc91f5.ts.net:7878/task?id=br-z7y5) | Incident: syspolicyd and Spotlight pegged, builds and app launches stalled: each spawn ... |  | pending | waits on the human (approve to ready) |

### Theme `human-ui`: The human's interface

What the human sees and touches: web UI, documents and comments, to-dos, links, quiet hours.

#### Not in an epic

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-1665](http://dalek.tailbc91f5.ts.net:7878/task?id=br-1665) | A web UI for the human: my to-dos and decisions, to run through and check off |  | pending | waits on the human (approve to ready) |
| [br-pa8h](http://dalek.tailbc91f5.ts.net:7878/task?id=br-pa8h) | Document review: keep the review list in the database, and scan for unresolved comments... | low | pending | waits on the human (approve to ready) |
| [br-yydm](http://dalek.tailbc91f5.ts.net:7878/task?id=br-yydm) | bridle-ui: a usage page with week-to-week charts of the five-hour and seven-day limits ... |  | pending | waits on the human (approve to ready) |
| [br-enx3](http://dalek.tailbc91f5.ts.net:7878/task?id=br-enx3) | bridle link: the unified URL scheme (/p/{project}/tasks/{id}, ...), document and spec l... |  | planned | ready to build |
| [br-4ge4](http://dalek.tailbc91f5.ts.net:7878/task?id=br-4ge4) | Show the current budget and focus settings from the CLI, consistently |  | pending | waits on the human (approve to ready) |
| [br-44ms](http://dalek.tailbc91f5.ts.net:7878/task?id=br-44ms) | Start an unplanned focus period now, through an agent (for example quiet for sleep) |  | pending | waits on the human (approve to ready) |
| [br-yy88](http://dalek.tailbc91f5.ts.net:7878/task?id=br-yy88) | Interactive sessions reply in quiet hours: the focus gate never reaches background wakes |  | pending | waits on the human (approve to ready) |
| [br-v3b7](http://dalek.tailbc91f5.ts.net:7878/task?id=br-v3b7) | Postmortem: quiet hours didn't reach replies to background wakes |  | pending | waits on the human (approve to ready) |
| [br-9s8u](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9s8u) | Background wakes carry the session's prompt context: quiet hours, and every hook that s... |  | pending | waits on the human (approve to ready) |
| [br-sdrw](http://dalek.tailbc91f5.ts.net:7878/task?id=br-sdrw) | Turn off Claude Code prompt suggestions in every bridle session |  | pending | waits on the human (approve to ready) |
| [br-767c](http://dalek.tailbc91f5.ts.net:7878/task?id=br-767c) | Drop the Write(path) focus-file deny rules: Claude Code warns on every session start |  | pending | waits on the human (approve to ready) |
| [ui-hu3k](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-hu3k) | Review the document picker prototypes and choose A, B or C (ui-m2pz) |  | claimed | the human's to-do |
| [ui-7jg4](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-7jg4) | [at restart] Test the new comment selection (ui-bpsd) on laptop and phone |  | claimed | the human's to-do |
| [ui-ha6m](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-ha6m) | The human can delete a resolved comment thread (kept in git history) |  | integrated | delivered |
| [ui-wtr3](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-wtr3) | An expanded comment thread can be collapsed again (a [-] button) |  | integrated | delivered |
| [ui-vnuu](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-vnuu) | Comment IDs never repeat after deletes: a counter in the document's front matter | low | integrated | delivered |
| [br-gd43](http://dalek.tailbc91f5.ts.net:7878/task?id=br-gd43) | Comment IDs never repeat after deletes: assign_ids reads and bumps a front-matter count... | low | planned | ready to build |
| [ui-qfur](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-qfur) | The human edits or deletes their own document comments until another role replies (unre... | low | pending | waits on the human (approve to ready) |

### Theme `specs`: Executable specs

The spec tooling (built, not wired in) and its rough edges.

#### Not in an epic

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-pakx](http://dalek.tailbc91f5.ts.net:7878/task?id=br-pakx) | Specs: a way to run executable specs in CI without a bridle checkout |  | pending | waits on the human (approve to ready) |
| [br-2d6x](http://dalek.tailbc91f5.ts.net:7878/task?id=br-2d6x) | Specs: vitest-bridle setup leaves step files untypechecked |  | pending | waits on the human (approve to ready) |
| [br-m5kf](http://dalek.tailbc91f5.ts.net:7878/task?id=br-m5kf) | Specs: say what happens to unit tests a scenario now covers |  | pending | waits on the human (approve to ready) |
| [br-awh4](http://dalek.tailbc91f5.ts.net:7878/task?id=br-awh4) | Specs: step text can't quote code or markup |  | pending | waits on the human (approve to ready) |
| [br-dbvd](http://dalek.tailbc91f5.ts.net:7878/task?id=br-dbvd) | Specs: bridle spec id duplicates ledger entries for hand-written IDs |  | pending | waits on the human (approve to ready) |
| [br-s4ve](http://dalek.tailbc91f5.ts.net:7878/task?id=br-s4ve) | vitest-bridle: a step can't skip a scenario at runtime |  | pending | waits on the human (approve to ready) |

### Theme `onboarding`: Onboarding

New projects under bridle and the packs they need.

#### Not in an epic

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-a16f](http://dalek.tailbc91f5.ts.net:7878/task?id=br-a16f) | Onboarding survey: file-db |  | pending | waits on the human (approve to ready) |
| [br-c265](http://dalek.tailbc91f5.ts.net:7878/task?id=br-c265) | Onboarding survey: track-web and harness (pi) |  | pending | waits on the human (approve to ready) |
| [br-5299](http://dalek.tailbc91f5.ts.net:7878/task?id=br-5299) | Onboarding survey: otters (otter-life and otters-back) |  | pending | waits on the human (approve to ready) |
| [br-f070](http://dalek.tailbc91f5.ts.net:7878/task?id=br-f070) | dotfiles-local as a bridle project, directly on main (35mw) |  | pending | waits on the human (approve to ready) |
| [br-22n2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-22n2) | Web/mobile pack rule: non-prose inputs turn off autocapitalize, autocorrect and spellcheck |  | pending | waits on the human (approve to ready) |
| [br-9xbk](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9xbk) | Research: web design rule sources to steal from, proposed web/mobile pack rules, attrib... |  | pending | waits on the human (approve to ready) |
| [tw-5915](http://dalek.tailbc91f5.ts.net:7878/task?id=tw-5915) | Push bridle-adopt to dev and main (deploys production) | high | claimed | the human's to-do |
| [tw-57dv](http://dalek.tailbc91f5.ts.net:7878/task?id=tw-57dv) | Decide: delete the bridle-adopt branch (track-web now works on dev) |  | claimed | the human's to-do |
| [tw-mbzn](http://dalek.tailbc91f5.ts.net:7878/task?id=tw-mbzn) | Review the engagement-tracking + feedback design (openspec change games-engagement-and-... |  | claimed | the human's to-do |

### The human's to-dos

Chores assigned to the human: expand on an idea, review a ticket or branch.

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-a3b9](http://dalek.tailbc91f5.ts.net:7878/task?id=br-a3b9) | Sat 10-03: review and land the parked branches (br-6b8a, br-2718, br-2672, br-8b98, br-... |  | claimed | the human's to-do; aide asks whether to close |
| [br-7575](http://dalek.tailbc91f5.ts.net:7878/task?id=br-7575) | Expand on 8r5x: keep your interactive session logs (which sessions, how long, where) |  | claimed | the human's to-do |
| [br-3a42](http://dalek.tailbc91f5.ts.net:7878/task?id=br-3a42) | Expand on z485: make the orchestrator non-interactive (what it gives you, who you talk ... |  | claimed | the human's to-do |
| [br-da2b](http://dalek.tailbc91f5.ts.net:7878/task?id=br-da2b) | Expand on 2tpm: evaluate Go instead of Rust (why, what would decide it) |  | claimed | the human's to-do |
| [br-9667](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9667) | Expand on 67qw: improve the base system's architecture (which parts, what's wrong) |  | claimed | the human's to-do |
| [br-tkph](http://dalek.tailbc91f5.ts.net:7878/task?id=br-tkph) | Follow up on the workflow review: 6 decisions, 2 waiting on others (34bw, vp9e, sk52, 7... |  | claimed | the human's to-do |
| [br-efs2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-efs2) | Review and comment on kuw2, the machine daemon design (docs/tickets/open/a-machine-daem... |  | claimed | the human's to-do |
| [br-cfb2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-cfb2) | Review ticket v8uu: seeing what background agents do (findings + 5 possible features) |  | claimed | the human's to-do |
| [br-twg8](http://dalek.tailbc91f5.ts.net:7878/task?id=br-twg8) | Try document review (x8jt) on gtzx: install the UI, review add, start the gateway, comment |  | claimed | the human's to-do |

## Delivered (since the trial began)

| When (ET) | Task | Workstream |
|---|---|---|
| 2026-10-09 03:05 | br-9xze scheduled messages, slice 1 | scheduler |
| 2026-10-09 03:13 | br-ukpm designer role | product-process |
| 2026-10-09 morning | br-7m99, br-r9h7 macOS test flake | reliability |
| 2026-10-09 morning | br-2uje bridle doctor Linux/WSL checks | multi-machine |
| 2026-10-09 morning | ui-kqsp, ui-5zrr Document page markdown rendering and highlights | human-ui |
| 2026-10-09 ~10:00 | br-g3az status line token docs (first item routed through the PdM) | tickets-release |
| 2026-10-09 ~12:20 | br-6nzj test daemons poll at 200 ms, resource-budget test | performance |
| 2026-10-09 ~12:00 | br-f4xu macOS CI flake (sigterm exit 143) | reliability |
| 2026-10-09 ~12:50 | br-h7gt Linux CI flake (mid-turn upgrade test made deterministic) | reliability |
| 2026-10-09 ~13:50 | br-vabu Linux CI store flake (main green again) | reliability |
| 2026-10-09 ~14:05 | br-hua2 add-a-machine guide (docs/context/add-a-machine.md) | machine setup |
| 2026-10-09 afternoon | ui-ha6m delete a resolved comment thread | human-ui |
| 2026-10-09 ~14:15 | br-b6mu self-upgrade drain during a spawn | reliability |
| 2026-10-09 ~14:34 | br-gdf3 peer-token direction rule in errors, help and docs | machine setup |
| 2026-10-09 ~15:25 | br-57nt gateway restart stays under launchd, Claude env stripped | machine setup |
| 2026-10-09 ~16:10 | br-xrkh systemd uninstall; owner refusal never crash-loops a unit | machine setup |

## Changes to this roadmap

- 2026-10-09 09:20 ET: first version. 12 workstreams; every open task in bridle, bridle-ui and
  track-web placed in one.
- 2026-10-09 09:45 ET: added br-d9wq (PdM research) to product process; decision 7 (the term).
- 2026-10-09 10:05 ET: br-g3az delivered.
- 2026-10-09 10:45 ET: new workstream Machine setup (PRIORITY), first. Filed hua2, xrkh, kt25; readied hua2, xrkh, gdf3, 57nt; priority high on those and 8c25, 88d4, 751e. Moved 8c25, gdf3, rjd5, v7ug, jgdb, h7mu, f8f9, 57nt, 88d4, 751e into it. Placed br-f4xu (CI flake) in reliability.
- 2026-10-09 10:25 ET: machine setup phase 1 all planned by pm-1 (hua2, gdf3, 57nt, xrkh now ready to build, with 8c25, 88d4, 751e).
- 2026-10-09 12:22 ET: br-6nzj and br-f4xu delivered. Placed br-h7gt (Linux CI flake, critical) in reliability.
- 2026-10-09 12:54 ET: br-h7gt delivered.
- 2026-10-09 13:25 ET: placed br-vabu (Linux CI flake, critical; main red, blocks hua2 merge) and br-b6mu (self-upgrade drain bug from h7gt) in reliability. Asked the orchestrator to put machine setup ahead of normal work; agreed: vabu, then hua2, then phase 1.
- 2026-10-09 14:06 ET: br-vabu, br-hua2 delivered. br-8c25 HELD: the human widened token pair (one command, role and peer tokens, all machines/projects/roles by default); design written into ticket sk7p (n63z folded in), waits on the human's approval.
- 2026-10-09 14:20 ET: the human answered postmortem j7r4 (c1-c5). Filed 8z7j (one pusher enforced; needs design) and k6jd (fetch, divergence warning; high) in machine setup; 8ay6 (push docs commits) and 8umh (push failed alarm) in reliability, readied; xccp (git identity per machine) low, pending.
- 2026-10-09 14:28 ET: ui-ha6m delivered. The human asked for comment IDs that never repeat (ui-vnuu, via the bridle-ui aide): readied high with its bridle half br-gd43, in human-ui; no design review (the human gave the shape).
- 2026-10-09 14:15 ET: br-b6mu delivered.
- 2026-10-09 14:23 ET: the human approved the sk7p design (token pair: role and peer tokens, all by default, opt-out [mail] peers = false). br-8c25 hold released; pm-1 re-briefs.
- 2026-10-09 14:34 ET: br-gdf3 delivered.
- 2026-10-09: pm-1 split token pair: br-8c25 part 1 (role tokens, token-role list), br-jw9e part 2 (peer tokens, opt-out, pairing on project creation); placed in machine setup.
- 2026-10-09 14:55 ET: new workstream Everything is a ticket (22ab, from advisor (tickets)); moved d9wq, 95mu, stx8 into it; plan written into 22ab for approval. 8z7j: Option A approved, blocked on removing dotfiles-local shared git hooks on both machines (asked the dotfiles-local aide on the NUC).
- 2026-10-09 15:25 ET: br-57nt delivered. 22ab Q1-Q5 answered (stx8 out of the workstream; project-qualified IDs into step 1); plan still waits on approval. br-8z7j wrongly closed by its design landing, reopened HELD. br-gd43 brief: next_comment_id: c<n> and the human's doc_watch addition.
- 2026-10-09 15:40 ET: the human: all comment work low, end of queue (br-gd43, ui-vnuu, br-pa8h set low); order by priority, never size (each task ~30-40 min of builds). Placed br-jxwr (task timing) in performance, low (the human: don't jump the queue).
- 2026-10-09 15:50 ET: incident br-rztb (gateway restarted outside launchd, recurring; the human asked for an investigation; critical, straight to the orchestrator) placed in reliability.
- 2026-10-09 16:10 ET: br-xrkh delivered.
- 2026-10-09 16:30 ET: restructured from workstreams into themes (slugs) and epics, the human's model (d9wq): roadmap orders epics, grouped by theme. 5 epics, 9 themes, the human's to-dos apart.
- 2026-10-09 16:50 ET: the human approved the epic Done-when drafts and the 22ab plan. Filed 22ab steps 1-4 (br-syqn readied; bpku, 3v75, 72t9 chained by blocked_by). New epic `migrations` (the human's ask), third in order; br-e7e2 moved into it.
- 2026-10-09 17:10 ET: filed ui-vhrb (theme page and roadmap page; the human's future work), low, theme product-process. Open question on br-syqn: themes and epics cross projects.
- 2026-10-09 17:30 ET: new epic `products` (g5dm; the human: a product is a set of projects with one roadmap and one PdM), fourth in order; br-6h65 (the PdM role) moved into it. br-syqn's question answered (theme: accepts any slug for now).
- 2026-10-09 17:40 ET: `products` moved last and br-g5dm low (the human: "the right direction BUT NOT URGENT"). No enforced theme list yet; one later (the human).
- 2026-10-09 18:15 ET: br-8c25 approved to land (the human); waits for main green (br-ngya, critical CI flake, placed in reliability).
- 2026-10-09 18:40 ET: filed br-4cgx (@-mention a role in a ticket reply or document comment; the human's ask), low, theme agents-and-cli, no epic.
- 2026-10-09 18:45 ET: br-8a53 (main red on ubuntu, gateway_test) placed in reliability.
- 2026-10-09 18:55 ET: filed ui-qfur (the human edits or deletes own document comments until another role replies; via the bridle-ui aide), low, theme human-ui.
- 2026-10-09 19:15 ET: 57nt resolved (gateway seen restarting under launchd at the 5:31 PM upgrade). Filed and readied br-3mz4 (one orchestrator per machine: advisor role and sends), theme agents-and-cli.
