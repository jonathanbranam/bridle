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

Last updated: 2026-10-17 7:30 AM ET. "Epic status" below is current (checked against live
ticket states); the theme tables further down were last regenerated 2026-10-12 and are stale.

## Epic status (2026-10-16), in the human's order

What each epic needs from the human to finish. "Hands-on" is the human's own doing; "decision"
needs real thought; cheap approvals were cleared 2026-10-16 (p4f2 ef3u re79 4cgx 9dcz ui-sxs3
ui-qfur readied, d9wq dropped).

| # | Epic | Status | Needs from the human |
|---|---|---|---|
| 0 | Usage you can trust (rdkn) | Done (UI ui-h79b, ui-evy7 queued) | Nothing |
| 1 | Three machines on bridle (du7e) | Nearly done: all on 0.7.5 (confirmed 10-16) | Migrations complete everywhere (waits on p4f2 for track-web on dalek); then 22ab step 10 (8zts) opens |
| 1b | Transfer a project (kx5u) | In progress: code built, trial 1 done (data-contracts on remus) | Hands-on: trial 2 (back), trial 3 (work in flight). Decision: is K7, the one `bridle project move` (kt25), part of V1? |
| 1c | Bridle to remus (jw75) | Not started, after 1b | Decision: when the design pass starts. Hands-on: dalek-home |
| - | One-command project setup (dffz) | Nearly done: all built | Hands-on: one real `bridle project setup` |
| 2 | UI ships with releases | Done (2026-10-17: 785a folded into 1c) | - |
| 3 | One orchestrator per machine | In progress (homes run orchestrators; 9966 queued) | Decision: confirm done-when; "second orchestrator can't start" and "coordinate a takeover" undefined. Hands-on: reboot test (mru4) |
| 4 | Machine admin role (j6d3) | Not started | Approve gb97's design pass |
| 5 | Products | Blocked, low | Decision: g5dm's 5 questions |
| 6 | Stages MVP: tickets prepare themselves (br-tj7q): NEXT PRIORITY (the human, 2026-10-17) | Grilled and scoped 2026-10-17 (advisor tj7q): MVP tj7q (7 children, 9mqh first), V1 ej8y, V2 ckbt; all pending | Ready the MVP children (the human wants it done today, to use tomorrow); step 6 (ewk8) is the human's hands-on try; step 7 (u2r8) waits for their go |
| 7 | Gates (reviews-enforced) | Design-before-build half moves into tj7q (95mu); review-before-landing (v2va slices A-D) held | Re-review v2va once 22ab ends |
| 8 | Machine sync | In progress | Reboot test (as 3); fold or drop rjd5 |
| 9-11 | Machine-transparent, background agents, security | Not started by choice | Nothing now |

Finishing by themselves: everything-is-a-ticket (8zts, gated on 1), migrations (p4f2). Done:
messaging, machine setup, ui-releases' t3dr. Benchmarking: held (v6kr); the human wants another
idle run on a quiet midweek night (9d95 dropped 10-16).

Most "(PdM draft)" done-when lines were never confirmed by the human.

## The scheduler ladder (the human's sequence, 2026-10-09)

Goal: ship scheduled messages, with design and specs approved before build. In build order:

2026-10-17: steps 2-8 are replaced by epic br-tj7q (change stages on every ticket); step 9's rest (yfv5) becomes a ticket run through it. 2026-10-16: stalled since 10-09. Steps 2-8 never started (step 2 waited on two decisions); work moved to machines and transfers. Restart, fold into gates (epic 7), or drop as a sequence: the human's call.

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

The human's order, 2026-10-11 ~6:35 PM ET (to advisor product-manager): "In terms of the bridle
system, the most impactful changes after that will be: 1. Per-machine role support. Both current
orch and an admin role in a following epic. ... 2. Formalize the PdM role and functionality ...
3. Multi-part tickets (to support design work) 4. Proper per task workflow (mostly 10 I think)";
11 and 15 "could go anytime if there is room in the queue waiting for me"; security "is low
priority, comes after many other epics"; background agents "can wait for now".

Done or finishing: machine setup (code done), messaging, scheduled messages (what's left waits on
kuw2), benchmarking (report due), everything is a ticket (step 10 gated on three machines current),
migrations.

0. **Usage you can trust** (`usage`, br-rdkn; the human, 2026-10-12 ~6:35 PM ET: "Let's do it"; front of the queue)
1. **Human work: bring the three machines onto bridle** (du7e; the human's to-dos h7mu, jgdb, r2nr)
   - The builds it waits on are epic kx5u (below).
1b. **Machines: set up a new machine and transfer a project to it, nothing left behind** (kx5u; the human, 2026-10-11 ~8:45 PM ET: transfer bridle and bridle-ui to remus, a smaller project first). Order: 3pc2 inventory, eurm, 2wnn, 53gy, kt25 (carry bridle.db, mail bridge, gateway; overrides kt25's wait for orchestrator-per-machine). Then rjd5, n63z folded in or dropped.
1c. **Move bridle and bridle-ui to remus: remus the dev machine, dalek a release client** (jw75; the human, 2026-10-11 ~9:05 PM ET). After kx5u, proven first on a small project (data-contracts). First step a design (orchestrator home, workflow source, upgrades, UI). Absorbs br-785a: the dev machine (remus) installs bridle-ui landings itself (tc7t option C); dalek, a release client, gets the UI from releases (t3dr). The human, 2026-10-17 ~6:20 PM ET: "Ok sure".
2. **bridle-ui ships with bridle releases** (`ui-releases`; t3dr landed 2026-10-11, in v0.6.2: done)
3. **One orchestrator per machine** (`orchestrator-per-machine`)
4. **Machine admin role** (j6d3; gb97)
5. **Products: formalize the PdM role** (`products`)
6. **Stages MVP: tickets prepare themselves** (br-tj7q; the human, 2026-10-17 ~7:25 AM ET: "tj7q should be the next priority"; grilled and split into MVP, V1 ej8y, V2 ckbt ~5:00 PM ET). Absorbs `attachments` and scheduler ladder steps 2-8, and 95mu from gates.
7. **Gates: design before build, review before landing: the per-ticket workflow** (`reviews-enforced`; stx8 first)
8. **Machine sync and recovery** (`machine-sync`)
9. **Projects become machine-transparent** (`machine-transparent`)
10. **Interactive agents move to the background** (`background-agents`; waits)
11. **Security and hardening** (3827; low)

Anytime, when the queue is waiting on the human: **Comments and mentions reach the right agent**
(`comment-routing`), **bridle-ui enhancements and fixes** (`ui-cleanups`).

## Themes

### Theme `multi-machine`: Many projects, many machines

Daemons, tokens, sync and projects across the laptop, the NUC and the Windows PC.

**Phases** (the human, 2026-10-12 ~9:25 AM ET, verbatim): "I see several phases (epics) in this
overall project work theme. we make steps towards the eventual goal of a project not necessarily
assigned to a particular machine at all. How far this makes sense to pursue is something we will
determine. One phase will be that a project has a home machine, but worker agents and interactive
agents can run anywhere."

1. Today: a project runs on one machine; transferring it is a lot of manual work.
2. MVP: bridle commands pause and land work on one machine and transfer it; a few manual steps by
   agents or the human may remain. (Epic kx5u; kt25 is its design.)
3. Workers run on any machine: they push git branches; needs coordination between the owning
   manager and both machines' orchestrators. Depends on one orchestrator per machine.
4. Project advisors and aide run on any machine: they talk by messages; how they change files like
   tickets is TBD; needs much design and iteration; more branch and merge conflicts. "may not be
   worth the investment!"
5. Projects are fully machine-free: all work and agents run anywhere and interact the same way.
   "also, may not be worth the investment!"

Phases 3-5 are epic `machine-transparent` (below) until split; 4 and 5 are decided later.

#### Epic `machine-setup`: Machine setup: the PC runs bridle from a release

- Outcome: A second machine (first the Windows PC) runs bridle unattended from a release, with no bridle source clone: background daemons, tokens between machines and projects, owner-only pushes, and the binary and workflow upgrading together by release (chvf, mrhe). The human, 2026-10-10: "Split the epics and write a proper 'when done' statement for the epic".
- Done when: (1) v0.6.0 is released with br-751e in it and CI on main is green; (2) bridle installs from a release in one step with no clone (fv86), and the Windows PC is named and set up under WSL2 that way, its daemon and gateway running from the release binary with no bridle clone, its workflow at the binary's tag; (3) the PC's and the NUC's daemons run self_upgrade = "release" and take the next release on their own; (4) tickets chvf and mrhe are resolved.

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-ygkc](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ygkc) | release::tests::installs_and_keeps_the_previous_binary fails with checksum mismatch (br... |  | integrated | delivered |
| [br-751e](http://dalek.tailbc91f5.ts.net:7878/task?id=br-751e) | Daemon keeps its own workflow checkout at the binary's tag (chvf 3) | high | integrated | delivered |
| [br-36dp](http://dalek.tailbc91f5.ts.net:7878/task?id=br-36dp) | Machine-wide self_upgrade: let ~/.bridle/config.toml set [daemon] self_upgrade for ever... | high | integrated | delivered |
| [br-fv86](http://dalek.tailbc91f5.ts.net:7878/task?id=br-fv86) | Install bridle from a release with no clone: one step that downloads, verifies and inst... | high | integrated | delivered |
| [br-jgdb](http://dalek.tailbc91f5.ts.net:7878/task?id=br-jgdb) | Windows PC: follow the WSL2 setup guide (install WSL2 + Ubuntu, wslconfig, Tailscale, s... | low | claimed | the human's to-do |
| [br-h7mu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-h7mu) | Pick a name for the Windows PC (docs/context/naming.md) (v7ug) | low | integrated | done (the human's to-do); ticket still open |
| [br-3932](http://dalek.tailbc91f5.ts.net:7878/task?id=br-3932) | Run bridle on a project without a local bridle clone (mrhe) |  | integrated | umbrella: both parts landed (br-85bc, br-8411); closes with tickets mrhe and chvf once br-751e lands |
| [br-hua2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-hua2) | Add a machine: one setup guide from bare OS to on the network (config, tokens, services... | high | integrated | delivered |
| [br-gdf3](http://dalek.tailbc91f5.ts.net:7878/task?id=br-gdf3) | Peer-token setup guidance: a token per receiving project per sending machine, minted on... | high | integrated | delivered |
| [br-8c25](http://dalek.tailbc91f5.ts.net:7878/task?id=br-8c25) | bridle token pair, part 1: the token-role list, the spec, and role tokens across machin... | high | integrated | delivered |
| [br-jw9e](http://dalek.tailbc91f5.ts.net:7878/task?id=br-jw9e) | bridle token pair, part 2: peer tokens, the [mail] peers opt-out, and pairing on projec... |  | integrated | delivered |
| [br-88d4](http://dalek.tailbc91f5.ts.net:7878/task?id=br-88d4) | self_upgrade = "release": fetch, verify and swap the release binary (chvf 2) | high | integrated | delivered |
| [br-57nt](http://dalek.tailbc91f5.ts.net:7878/task?id=br-57nt) | bridle gateway restart takes a launchd-managed gateway out of launchd and inherits the ... | high | integrated | delivered |
| [br-xrkh](http://dalek.tailbc91f5.ts.net:7878/task?id=br-xrkh) | systemd uninstall, and an owner refusal never crash-loops a launchd or systemd unit aft... | high | integrated | delivered |
| [br-8z7j](http://dalek.tailbc91f5.ts.net:7878/task?id=br-8z7j) | Only the owner's clone can push the integration branch: enforced, not a rule |  | dropped | dropped: built as br-hdbj |
| [br-hdbj](http://dalek.tailbc91f5.ts.net:7878/task?id=br-hdbj) | Only the owner's clone can push the integration branch: enforced, not a rule (build) |  | integrated | delivered |
| [br-k6jd](http://dalek.tailbc91f5.ts.net:7878/task?id=br-k6jd) | Managers and the orchestrator fetch origin; divergence from origin is warned (N ahead, ... | high | integrated | delivered |

#### Epic `machine-sync`: Machine sync and recovery

- Outcome: Phases 2 and 3 of the old machine-setup epic, split out 2026-10-10 at the human's ask: each machine is set up and kept in sync by one idempotent command (rjd5), recovers everything on boot (f8f9), and the PC takes bridle's heavy work (v7ug). Sequenced after one orchestrator per machine (the human, 2026-10-10).
- Done when: (PdM draft) One idempotent command sets up or re-syncs a project on a machine (rjd5); the NUC and the PC bring every daemon, gateway and session back after a reboot with no human step (f8f9); heavy builds and checks can run on the PC (v7ug).

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-rjd5](http://dalek.tailbc91f5.ts.net:7878/task?id=br-rjd5) | Commands to set up and sync a project everywhere: sync all seven places per machine, an... |  | pending | needs a design (source of truth for the project list), then the human's approval |
| [br-f8f9](http://dalek.tailbc91f5.ts.net:7878/task?id=br-f8f9) | The NUC recovers everything on boot (4r3k) |  | pending | 'bridle up' after boot needs a small design |
| [br-v7ug](http://dalek.tailbc91f5.ts.net:7878/task?id=br-v7ug) | Run bridle's heavy work on the Windows PC under WSL2 |  | pending | umbrella; audit and guide delivered |

#### Epic `machine-transparent`: Projects become machine-transparent

- Outcome: FUTURE (unscheduled). A project is set up on every machine and moves between them on command; which machine runs it becomes transparent to bridle; later, one project's workers on several machines (evy6). The human, 2026-10-10: "It should essentially start to become transparent to bridle which machine a project is running on."
- Done when: (PdM draft) One placement record; projects cloned and synced everywhere (rjd5); move on command with a drain (kt25); agents reach a project without knowing its machine. One project, one machine stays the rule until the human changes it.

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-evy6](http://dalek.tailbc91f5.ts.net:7878/task?id=br-evy6) | Projects become machine-transparent: one placement record, move on command, later worke... |  | pending | waits on the human (approve to ready) |
| [br-kt25](http://dalek.tailbc91f5.ts.net:7878/task?id=br-kt25) | Move a project between machines with one command (bridle project move) |  | integrated | design pass landed; ticket kt25 still open |

#### Not in an epic

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-eadm](http://dalek.tailbc91f5.ts.net:7878/task?id=br-eadm) | Keep dalek awake while it's open: a toggled check every ~30 min that tells the orchestr... |  | pending | waits on the human (approve to ready) |
| [br-xccp](http://dalek.tailbc91f5.ts.net:7878/task?id=br-xccp) | A git identity per machine, so commits show which clone made them | low | pending | low, unscheduled; no new credentials (option A: machine in the committer name, aide 2026-10-10); for the human to approve |
| [br-u6w9](http://dalek.tailbc91f5.ts.net:7878/task?id=br-u6w9) | Human interaction time: daemon serves the prompt log; gateway collects across machines ... |  | pending | waits on the human (approve to ready) |
| [ui-9hq8](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-9hq8) | Documents of projects on another machine (the NUC): read and comment in bridle-ui | high | integrated | delivered |
| [br-xxq8](http://dalek.tailbc91f5.ts.net:7878/task?id=br-xxq8) | List the projects an identity holds tokens for, without printing the tokens | low | pending | waits on the human (approve to ready) |
| [br-m5n2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-m5n2) | Onboard data-contracts: start the trial that was prepared on 2026-09-28 |  | dropped | dropped |

### Theme `tickets-and-release`: Tickets, changelog and release

How work is recorded (tickets, fields, migrations) and shipped (changelog, releases, self-upgrade).

#### Epic `everything-is-a-ticket`: Everything is a ticket

- Outcome: One record per piece of work: tickets, with a row for every ticket and the thread on the state branch; standard field names; project-qualified IDs. Ticket 22ab holds the decisions, design and plan.
- Done when: The code rename is finished too (the human: "must be completed as part of this workstream before it is considered complete").

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-syqn](http://dalek.tailbc91f5.ts.net:7878/task?id=br-syqn) | Ticket fields get standard names (blocked_by, related, parent, created, created_by, res... |  | integrated | delivered |
| [br-epvy](http://dalek.tailbc91f5.ts.net:7878/task?id=br-epvy) | Project-qualified ticket and task IDs (syqn part 2): br-k7tm in frontmatter, links, thr... |  | integrated | delivered |
| [br-wrmu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-wrmu) | Ticket migration (syqn part 3): rename ticket frontmatter fields and qualify IDs in eve... |  | integrated | delivered |
| [br-bpku](http://dalek.tailbc91f5.ts.net:7878/task?id=br-bpku) | Every ticket gets a row; types fix and epic; readiness moves to the ticket (ticket ready) |  | integrated | delivered |
| [br-3v75](http://dalek.tailbc91f5.ts.net:7878/task?id=br-3v75) | Ticket links live in the frontmatter: the daemon indexes blocked_by, parent and related... |  | integrated | delivered |
| [br-72t9](http://dalek.tailbc91f5.ts.net:7878/task?id=br-72t9) | A ticket's thread moves to the ticket (tickets/<id>.md on the state branch); ticket com... |  | integrated | first slice landed; ticket 72t9 still open |
| [br-stx8](http://dalek.tailbc91f5.ts.net:7878/task?id=br-stx8) | A task's state says what's really happening: held and built-awaiting-landing are states... |  | integrated | delivered |

#### Epic `migrations`: Migrations

- Outcome: Every bridle project's tickets and tasks are brought to the current format by migrations that run on their own at start-up (xebc, br-2718 landed). The human, 2026-10-09: "this would be a great new epic to have! migrations!"
- Done when: (PdM draft) Every open ticket in every project has a kind and two-way task links; `bridle ticket check` is clean in each project; later format changes (22ab) ship as migrations here.

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-e7e2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-e7e2) | Migration: backfill ticket kind and two-way task links in every bridle project (v3dk sl... |  | integrated | delivered |

#### Not in an epic

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-g3az](http://dalek.tailbc91f5.ts.net:7878/task?id=br-g3az) | Status line token setup in the docs writes an empty file: token create needs --print now |  | integrated | delivered |
| [br-01ff](http://dalek.tailbc91f5.ts.net:7878/task?id=br-01ff) | Tickets through the bridle binary in every project: new, frontmatter, check, resolve; a... |  | pending | waits on the human (approve to ready) |
| [br-ubjd](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ubjd) | A CHANGELOG line for every landed task, written on the branch, and one section per kind... |  | pending | waits on the human (approve to ready) |

### Theme `agents-and-cli`: Agents and the CLI

Agents use bridle correctly: commands, help, roles, the workflow reaching every agent, timed actions.

#### Epic `messaging`: Messages are reliable, one waiter per agent

- Outcome: Sending and receiving messages works reliably across projects and machines, and each principal waits on one waiter on its home daemon (3haz). The human, 2026-10-09: "let's prioritize work that makes sending and receiving messages work better and more reliable, reducing waiter counts like the orc has 5 waiters".
- Done when: (PdM draft) The orchestrator runs one waiter; every wake reason arrives as a message; send works to any project on the machine without a peer token; CI failures wake on time; outbox state is visible.

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-3haz](http://dalek.tailbc91f5.ts.net:7878/task?id=br-3haz) | Mail between daemons, slice 1: outbox, peer tokens, forwarding with acknowledgement and... |  | integrated | delivered |
| [br-fvkq](http://dalek.tailbc91f5.ts.net:7878/task?id=br-fvkq) | Mail between daemons, slice 3: outbox retry with backoff and the start-up ping (3haz P3) |  | integrated | delivered |
| [br-hwek](http://dalek.tailbc91f5.ts.net:7878/task?id=br-hwek) | A named advisor's unread mail stays in its own inbox when its session ends or restarts ... |  | integrated | delivered |
| [br-9aj2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9aj2) | Message delivery you can check, part 1: bridle messages command, delivery audit events ... |  | integrated | delivered |
| [br-3zhx](http://dalek.tailbc91f5.ts.net:7878/task?id=br-3zhx) | Message delivery you can check, part 2: a message stays unread until the session has se... |  | integrated | delivered |
| [br-djru](http://dalek.tailbc91f5.ts.net:7878/task?id=br-djru) | Message delivery you can check, part 2: a message stays unread until the session has se... |  | dropped | dropped |
| [br-2msq](http://dalek.tailbc91f5.ts.net:7878/task?id=br-2msq) | bridle send --project to a local daemon needs a peer token, and --url can't be combined... |  | integrated | delivered |
| [br-n7cg](http://dalek.tailbc91f5.ts.net:7878/task?id=br-n7cg) | Mail between daemons, slice 2: mail for a visitor is forwarded to its home daemon (3haz... |  | integrated | delivered |
| [br-rhba](http://dalek.tailbc91f5.ts.net:7878/task?id=br-rhba) | Mail between daemons, slice 5: every orchestrator wake is a message, sent home; one wai... |  | integrated | delivered |
| [br-ysmu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ysmu) | CI watch missed 13 red runs on main; first ci_failed wake came 40 minutes late |  | integrated | delivered |
| [br-cufw](http://dalek.tailbc91f5.ts.net:7878/task?id=br-cufw) | Mail between daemons, slice 4: visible state: outbox status, message show (queued/arriv... |  | integrated | delivered |
| [br-qac3](http://dalek.tailbc91f5.ts.net:7878/task?id=br-qac3) | Mail between daemons, slice 6: an agent of another project or machine can watch a task ... |  | integrated | 3haz slice 6 (the human's Q3: watching a task from another project must work); resolves 3haz |

#### Epic `scheduled-messages`: Scheduled messages and timed actions

- Outcome: Agents and the human schedule messages and timed actions through one scheduler (yfv5).
- Done when: Externals schedule for themselves (3k7d); nightly session restarts from a plain [[restart]] config (cbbn, the human 2026-10-11); maintenance windows and machine-wide schedules after kuw2/cy2v.

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-9xze](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9xze) | Scheduled messages, first slice: an agent schedules a message to itself (one-time or re... |  | integrated | delivered |
| [br-g5y2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-g5y2) | Scheduled messages slice 2: role priming: wait at the maximum timeout and schedule a me... |  | integrated | waits on the human; 9xze has landed |
| [br-yfv5](http://dalek.tailbc91f5.ts.net:7878/task?id=br-yfv5) | One scheduler for timed actions: scheduled messages (hrcn), nightly session restarts (c... |  | integrated | needs a design (designer) for the rest: cbbn, 3nyk, cy2v; then the human's review |
| [br-3k7d](http://dalek.tailbc91f5.ts.net:7878/task?id=br-3k7d) | Externals (orchestrator, advisors, aides) can schedule messages for themselves | high | integrated | delivered |
| [br-cbbn](http://dalek.tailbc91f5.ts.net:7878/task?id=br-cbbn) | Scheduled nightly restart of an interactive session at a clock time (e.g. 3 AM) |  | integrated | delivered |
| [br-ft3b](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ft3b) | Per-role handover instructions in the workflow, with project overrides |  | integrated | delivered |

#### Epic `background-agents`: Interactive agents move to the background

- Outcome: More of the interactive roles (orchestrator, aide, advisors) run under the daemon, so the daemon controls their message delivery instead of each agent's waiter discipline (ae9f). The human, 2026-10-10: "we need an epic around getting more of these interactive agents moved to the background".
- Done when: (PdM draft) A designer pass says which roles move and in what order; the first one runs headless and the human reaches it without a waiter.

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-ae9f](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ae9f) | Move interactive agents to the background, so the daemon controls their message delivery |  | pending | waits on the human (approve to ready) |

#### Epic `orchestrator-per-machine`: One orchestrator per machine

- Outcome: One orchestrator per machine, not per project, runs every project on that machine; its identity and recovery are clear (3mz4, ma8e, 7d62, pdmd; kuw2 related). The human, 2026-10-10: "let's make sure we have an epic for one orchestrator per machine, and let's sequence that epic before this one [project transfer]. I think that will simplify things. We already have problems with the orchestrator role".
- Done when: (PdM draft) Each machine runs one orchestrator for all its projects; roles and agents address it by machine; a second orchestrator can't start; it recovers cleanly; it is ready to coordinate a takeover with another machine's orchestrator.

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-3mz4](http://dalek.tailbc91f5.ts.net:7878/task?id=br-3mz4) | One orchestrator per machine, not per project: say so in the advisor role and wherever ... |  | integrated | delivered |

#### Not in an epic

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-jstr](http://dalek.tailbc91f5.ts.net:7878/task?id=br-jstr) | Recipient hint: suggest external:advisor/<name> for a bare advisor/<name> | low | integrated | delivered |
| [br-p896](http://dalek.tailbc91f5.ts.net:7878/task?id=br-p896) | Agent evaluations: run agents with a clean, specified context, capture everything, and ... | low | pending | future (the human, 2026-10-10): long-term, low; not now |
| [br-py93](http://dalek.tailbc91f5.ts.net:7878/task?id=br-py93) | Idea: bridle as a tool (MCP) for interactive sessions instead of the CLI, so waiting an... |  | pending | future idea (the human, 2026-10-10): unscheduled; goes with the background-agents epic (ae9f) |
| [br-7zd4](http://dalek.tailbc91f5.ts.net:7878/task?id=br-7zd4) | Remove the stale 'Until bridle workflow sync renders rules' lines from workflow/base/ru... | low | integrated | landed; ticket 98xt still open |
| [br-sfpg](http://dalek.tailbc91f5.ts.net:7878/task?id=br-sfpg) | A researcher role with web tools, and managers check a task's needs against the role's ... | low | pending | needs a designer pass (low; later) |
| [br-d48r](http://dalek.tailbc91f5.ts.net:7878/task?id=br-d48r) | Machine load notes go to aide, not the orchestrator |  | dropped | dropped |
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

#### Epic `change-stages` (br-tj7q): Stages MVP: tickets prepare themselves

- Outcome: A version 2 ticket prepares itself: bridle starts a designer for each stage (proposal in
  the body; design, specs, tasks as files beside the ticket) as it unblocks, giving it the stage's
  instructions and template at the moment of writing; it stops only at the gates the ticket lists
  for the human, who signs (traced) or sends back; prepared means ready; a worker builds from
  tasks.md and the specs land in bridle specs. Version 1 tickets keep today's workflow. Grilled
  with the human 2026-10-17 (advisor tj7q); replaces epic `attachments` and scheduler ladder steps
  2-8; takes 95mu from gates.
- Done when: version 1 and version 2 tickets can both be created and each follows its own
  workflow; one real version 2 ticket filed to landed with the human signing a gate (ewk8);
  version 2 is the default for new tickets (u2r8).
- Next: the human readies the children; 9mqh first, then quvh and ussh side by side, ewhy, 6c8x,
  ewk8 (the human's try), u2r8 (on their go).

| Step | Ticket | What | After |
|---|---|---|---|
| 1 | br-9mqh | version, `stages:` schema, proposal section, stage folder, check, stages.md | - |
| 2 | br-quvh | stage rules and templates; `bridle ticket next` | 1 |
| 3 | br-ussh | stage-done, approve, changes, restage; ready means prepared; watchers | 1 |
| 4 | br-ewhy | bridle spawns a designer per unblocked stage; timer to the PdM | 2, 3 |
| 5 | br-6c8x | a worker builds from tasks.md | 2 |
| 6 | br-ewk8 | one real version 2 ticket end to end, with the human | 4, 5 |
| 7 | br-u2r8 | version 2 becomes the default | 6 |

#### Epic `stages-v1` (br-ej8y): Stages you can see and steer

- Outcome: the bridle-ui ticket page with stage tabs and a picker (ui-fwsx folded in); review like
  a PR (highlight and overall comments; approve or request changes, sent to a fresh writer); a
  template-heading check; a designer can send a stage back as blocked; the interactive path.
- Next: after the MVP lands.

#### Epic `stages-v2` (br-ckbt): Gates beyond the human

- Outcome (to be grilled): approve with feedback; agent-signed gates and consulted reviewers;
  standard schemas and other stage types; attachments of any type; spec re-review on drift; rules
  injected into the apply step; mhjn reworked around "ready means prepared".
- Next: after V1.

#### Epic `reviews-enforced`: Gates: design before build, review before landing

- Outcome: A change's proposal and design are approved before it is planned (95mu, 22ab sections 5-6), and reviews are required, signed on the work and enforced before landing (v2va). Moved here from everything-is-a-ticket (the human, 2026-10-11 ~12:05 AM ET: "yes, agreed move it").
- Done when: The daemon refuses to plan a change ticket without its approved design (95mu), and no landing until every required review is signed (v2va slice D).

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-95mu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-95mu) | A change spec (proposal and design) reviewed for risk and impact before any worker buil... |  | pending | to the designer once the human has reviewed the ticket |
| [br-91b3](http://dalek.tailbc91f5.ts.net:7878/task?id=br-91b3) | Reviews enforced by bridle, signed on the task (v2va): umbrella, slices A-D | low | held | HELD |
| [br-6ba3](http://dalek.tailbc91f5.ts.net:7878/task?id=br-6ba3) | Reviews A: in_review state, ready-for-review, review requirements on tasks (v2va slice A) | low | held | HELD |
| [br-46fe](http://dalek.tailbc91f5.ts.net:7878/task?id=br-46fe) | Reviews B: signed review records on the task (v2va slice B) | low | held | HELD |
| [br-cf00](http://dalek.tailbc91f5.ts.net:7878/task?id=br-cf00) | Reviews C: the daemon spawns the required reviewers; comment-only talk; cost recorded (... | low | held | HELD |
| [br-f610](http://dalek.tailbc91f5.ts.net:7878/task?id=br-f610) | Reviews D: the gate: no landing until every required review is signed (v2va slice D) | low | held | HELD |

#### Epic `products`: Products: several product managers and roadmaps

- Outcome: Each product (a set of projects, 1:1 with GitHub repos, sharing one roadmap and one product manager; the human, 2026-10-09) has its own PdM and roadmap, so neither the agent's context nor the human's mixes unrelated products.
- Done when: There are multiple product managers, each responsible for a product that contains many projects; a product roadmap lives in a single place (probably a project), but its planning spans multiple projects. (The human.)

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-g5dm](http://dalek.tailbc91f5.ts.net:7878/task?id=br-g5dm) | Products: a product is a set of projects with one roadmap and one product manager; seve... | low | integrated | design written into the ticket; ticket still open, the human's review |
| [br-6h65](http://dalek.tailbc91f5.ts.net:7878/task?id=br-6h65) | A product manager that relates every ticket to open and planned work: links, merges, an... |  | pending | this trial is the stand-in (docs/notes/product-manager-trial.md) |

#### Not in an epic

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [ui-vhrb](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-vhrb) | A theme page (everything about one theme) and a roadmap page (epics in order) in the UI | low | pending | waits on the human (approve to ready) |
| [br-gtzx](http://dalek.tailbc91f5.ts.net:7878/task?id=br-gtzx) | Seats: every role is a named, tracked seat that outlives its sessions, with its own inb... |  | open | HELD: waits on the human's review of P1-P10, Q1-Q4 |
| [br-d9wq](http://dalek.tailbc91f5.ts.net:7878/task?id=br-d9wq) | Research: product manager roles, human and agentic (BMAD and others), and what to call ... |  | pending | research done; the human reads it and picks the term |
| [br-519b](http://dalek.tailbc91f5.ts.net:7878/task?id=br-519b) | Task watchers: a creator field, a watchers list, and wakes that say what changed and ar... |  | pending | waits on the human (approve to ready) |
| [br-cr7t](http://dalek.tailbc91f5.ts.net:7878/task?id=br-cr7t) | Add a postmortem ticket kind: the full write-up after an incident |  | pending | waits on the human (approve to ready) |
| [br-avu7](http://dalek.tailbc91f5.ts.net:7878/task?id=br-avu7) | vk3y slice 2: 'bridle task new' requires --ticket (no-ticket sentinel) and a rule for e... |  | pending | waits on the human (approve to ready) |

### Theme `reliability`: Reliability

The orchestrator and daemons stay up, relaunch once, upgrade cleanly and recover after sleep or reboot.

#### Not in an epic

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-vt9k](http://dalek.tailbc91f5.ts.net:7878/task?id=br-vt9k) | Incident: the benchmark sampler's script and ~2 h of data were lost when landing br-4vm... | high | pending | waits on the human (approve to ready) |
| [br-fpde](http://dalek.tailbc91f5.ts.net:7878/task?id=br-fpde) | Restart on Linux execs '<path> (deleted)' after the binary is replaced |  | integrated | delivered |
| [br-4vmc](http://dalek.tailbc91f5.ts.net:7878/task?id=br-4vmc) | Flaky lifecycle_test spawn_child_orphan_is_swept_on_stop times out waiting for agent.or... |  | integrated | delivered |
| [br-n96z](http://dalek.tailbc91f5.ts.net:7878/task?id=br-n96z) | Flaky under load: governor_test working_agent_is_notified_then_stopped_when_its_turn_en... |  | integrated | delivered |
| [br-37r9](http://dalek.tailbc91f5.ts.net:7878/task?id=br-37r9) | Landing a task removes its worker even when the worker holds another task's uncommitted... |  | integrated | delivered |
| [br-bcw6](http://dalek.tailbc91f5.ts.net:7878/task?id=br-bcw6) | cli_e2e sigint and events_stream shutdown tests flake under load |  | integrated | delivered |
| [br-p29s](http://dalek.tailbc91f5.ts.net:7878/task?id=br-p29s) | The macOS Verifying popup still flashes during bridle builds, though the daemons run un... |  | pending | waits on the human (approve to ready) |
| [br-8ff8](http://dalek.tailbc91f5.ts.net:7878/task?id=br-8ff8) | Flaky on CI: settle_wake_test blocked-task note races its edge setup | critical | integrated | delivered |
| [br-7ufd](http://dalek.tailbc91f5.ts.net:7878/task?id=br-7ufd) | Self-upgrade at most every few hours, batching the landings; critical fixes go through ... | high | integrated | delivered |
| [br-zpc7](http://dalek.tailbc91f5.ts.net:7878/task?id=br-zpc7) | Mail bridge failures as events or messages (gdyy part 2) | low | pending | waits on the human (approve to ready) |
| [br-8a53](http://dalek.tailbc91f5.ts.net:7878/task?id=br-8a53) | main red (ubuntu): gateway_test a_replaced_binary_is_re_executed; INVOCATION_ID misdete... | critical | integrated | delivered |
| [br-ngya](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ngya) | Flaky on Linux CI: upgrade_test a_drain_starting_during_a_spawn_restarts_promptly reads... | critical | integrated | delivered |
| [br-rztb](http://dalek.tailbc91f5.ts.net:7878/task?id=br-rztb) | Incident: something keeps restarting dalek's gateway outside launchd from a Claude sess... | high | integrated | delivered |
| [br-4zfa](http://dalek.tailbc91f5.ts.net:7878/task?id=br-4zfa) | Incident: the bridle orchestrator was killed (SIGTERM) at 11:18 PM ET and nothing relau... | high | pending | waits on the human (approve to ready) |
| [br-f4xu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-f4xu) | Flaky on macOS CI: process_test sigterm_via_signal_group_exits_143 exits 1, not 143 |  | integrated | delivered |
| [br-h7gt](http://dalek.tailbc91f5.ts.net:7878/task?id=br-h7gt) | Flaky on Linux CI: upgrade_test self_upgrade_restarts_only_after_the_mid_turn_agent_fin... | critical | integrated | delivered |
| [br-8ay6](http://dalek.tailbc91f5.ts.net:7878/task?id=br-8ay6) | Direct-to-main docs commits are pushed straight after, on the owner's clone |  | integrated | delivered |
| [br-8umh](http://dalek.tailbc91f5.ts.net:7878/task?id=br-8umh) | A failed push is an event and an alarm; an agent that can't send puts the blocker on th... |  | integrated | delivered |
| [br-vabu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-vabu) | Flaky on Linux CI: store cancelled_blocking_task_returns_shutting_down_not_a_panic | critical | integrated | delivered |
| [br-b6mu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-b6mu) | Self-upgrade drain may never restart when it starts during a spawn |  | integrated | delivered |
| [br-6b8a](http://dalek.tailbc91f5.ts.net:7878/task?id=br-6b8a) | Orchestrator relaunch liveness: one clock, never a second orchestrator (jf9u) | high | integrated | delivered |
| [br-96a6](http://dalek.tailbc91f5.ts.net:7878/task?id=br-96a6) | Hold the orchestrator relaunch without restarting the daemon (8fsx) |  | pending | waits on the human (approve to ready) |
| [br-9966](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9966) | bridle orchestrator hold / release: runtime switch for the relaunch (8fsx) |  | planned | ready to build |
| [br-10c0](http://dalek.tailbc91f5.ts.net:7878/task?id=br-10c0) | Orchestrator identity and disaster recovery (7d62) |  | integrated | delivered |
| [br-btdn](http://dalek.tailbc91f5.ts.net:7878/task?id=br-btdn) | Daemon restart and self-upgrade blocked forever: manager.spawning() stays true with no ... |  | integrated | delivered |
| [br-y455](http://dalek.tailbc91f5.ts.net:7878/task?id=br-y455) | Incident: bridle's daemon couldn't restart or self-upgrade for ~23 h: a stuck 'spawning... |  | planned | ready to build |
| [br-aqa7](http://dalek.tailbc91f5.ts.net:7878/task?id=br-aqa7) | Self-upgrade refused good builds 3 times overnight: the new binary's self-check timed o... |  | pending | waits on the human (approve to ready) |
| [br-5j35](http://dalek.tailbc91f5.ts.net:7878/task?id=br-5j35) | bridle session restart run from inside the session stops it and never relaunches: the d... |  | pending | waits on the human (approve to ready) |
| [br-ve74](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ve74) | A handover request reaches the new session after the old one already handed over | low | integrated | delivered |
| [br-s5ah](http://dalek.tailbc91f5.ts.net:7878/task?id=br-s5ah) | A daemon without an aide leaves its orchestrator no route to the human |  | pending | waits on the human (approve to ready) |
| [br-2ax5](http://dalek.tailbc91f5.ts.net:7878/task?id=br-2ax5) | Incident: br-3haz broke every cross-project message for ~22 h: the CLI needed an outbox... |  | planned | ready to build |
| [br-zcqv](http://dalek.tailbc91f5.ts.net:7878/task?id=br-zcqv) | dalek slept in a bag 8:42 AM-1:38 PM ET on 10-06: bridle froze, but the workforce had a... |  | pending | waits on the human (approve to ready) |
| [br-vn54](http://dalek.tailbc91f5.ts.net:7878/task?id=br-vn54) | Incident: remote control dropped for the dalek aide session after a tmux detach and lap... |  | pending | waits on the human (approve to ready) |

### Theme `performance`: Performance and resource cost

Bridle's own cost on the machine: measure against a baseline, then cut it.

#### Epic `usage` (br-rdkn): Usage you can trust: every session counted, costs right, the weekly limits charted

- Outcome: The human can see, and trust, where the account's token budget goes. The human, 2026-10-12 ~6:00 PM ET: "we need tracing and reporting to know for sure"; ~6:35 PM: "Agree on 1-3. Let's do it" and "Put this in a proper theme and epic of course".
- Done when: (PdM draft) no negative costs, a renew doesn't skew totals (9y7v); external and interactive sessions counted in `bridle usage`, or reported as not measured (hd28); week-to-week limit charts in bridle-ui (yydm, ui-u6uz); any agent instance's usage charted (xfy7, ui-mxkj).

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| br-9y7v | Usage shows negative costs for long-running agents | | open | to plan (first) |
| br-hd28 | Count the external and interactive sessions in bridle usage | | open | to plan |
| br-yydm | Gateway route for the limit history (UI: ui-u6uz) | | integrated | delivered; ui-u6uz delivered too (5690c8b) |
| br-xfy7 | Usage per agent instance over time (UI: ui-mxkj) | | open | to plan; after 9y7v |

#### Epic `benchmarking`: Benchmarking bridle's own cost

- Outcome: A repeatable live benchmark of bridle's own resource cost: the same committed script every time, idle and busy stretches, a log of what happened, results kept permanently (v6kr). The human, 2026-10-10: "The benchmarking is important but should not interrupt other work during the day"; "don't schedule or run the benchmark until I sign off".
- Done when: The design (ticket v6kr) signed off 2026-10-10; the script merged; a live and an idle run published on bridle/benchmarks; an agent's report on both (Sunday 2026-10-11). Runs alongside the ranked epics: time-bound and small.

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-re57](http://dalek.tailbc91f5.ts.net:7878/task?id=br-re57) | Benchmark design pass (designer): options and a recommendation in ticket v6kr |  | integrated | delivered |
| [br-57ec](http://dalek.tailbc91f5.ts.net:7878/task?id=br-57ec) | Benchmark script: write, commit and merge it per the v6kr design |  | integrated | delivered |
| [br-g9xe](http://dalek.tailbc91f5.ts.net:7878/task?id=br-g9xe) | Benchmark: first live run (GATED on the human's sign-off of the v6kr design and plan) |  | integrated | delivered |
| [br-9d95](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9d95) | Benchmark: idle run tonight, 2026-10-11 ~4:00 AM ET (after the live run checks out) |  | planned | ready to build |
| [br-ju5q](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ju5q) | Benchmark report: analyse the live and idle runs, ready Sunday 2026-10-11 morning |  | integrated | delivered |

#### Not in an epic

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-srj7](http://dalek.tailbc91f5.ts.net:7878/task?id=br-srj7) | Idea: Python helpers to load and chart benchmark runs from a Jupyter notebook |  | pending | tentative idea (the human, 2026-10-10): unscheduled; for the next PdM review of themes and epics |
| [br-tnyt](http://dalek.tailbc91f5.ts.net:7878/task?id=br-tnyt) | Machine load notes repeat every few minutes: add a quiet period, and send one note per ... | high | integrated | delivered |
| [br-n4w4](http://dalek.tailbc91f5.ts.net:7878/task?id=br-n4w4) | Postmortem: bridle's own 'ps' polling (every daemon, test daemons at 200 ms) drove dale... |  | dropped | dropped |
| [br-jxwr](http://dalek.tailbc91f5.ts.net:7878/task?id=br-jxwr) | Track and report how long a task takes from pickup to merge, split into agent work, bui... | low | pending | the human's ask (via aide); would show where the 30-40 min per task goes; low (the human): on a workstream, not queued now |
| [br-6nzj](http://dalek.tailbc91f5.ts.net:7878/task?id=br-6nzj) | Test daemons stop polling at 200 ms; a resource-budget test; log the incident in docs/c... |  | integrated | delivered |
| [br-g76s](http://dalek.tailbc91f5.ts.net:7878/task?id=br-g76s) | Load-hold notes: one per machine, name bridle-owned top consumers, honest text, load.ho... |  | integrated | delivered |
| [br-fzwa](http://dalek.tailbc91f5.ts.net:7878/task?id=br-fzwa) | Audit every periodic daemon loop for what it forks or reads per tick; list them with co... |  | integrated | delivered |
| [br-ks55](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ks55) | Only one full test run at a time per machine: just check takes a machine-wide lock (n4w... |  | integrated | delivered |
| [br-yw8b](http://dalek.tailbc91f5.ts.net:7878/task?id=br-yw8b) | fake-claude spawns skip the pyenv shim: resolve the interpreter once (n4w4 rec 7) |  | integrated | delivered |
| [br-z7y5](http://dalek.tailbc91f5.ts.net:7878/task?id=br-z7y5) | Incident: syspolicyd and Spotlight pegged, builds and app launches stalled: each spawn ... |  | pending | waits on the human (approve to ready) |

### Theme `human-ui`: The human's interface

What the human sees and touches: web UI, documents and comments, to-dos, links, quiet hours.

#### Epic `ui-releases`: bridle-ui ships with bridle releases

- Outcome: The UI is installed and upgraded with bridle, fixed to the bridle version, so a machine installed from a release has the matching UI (t3dr). The human, 2026-10-10: "the bridle-ui as part of that, all fixed to a version that comes from bridle ... If the UI is not done, that could be a separate epic".
- Done when: (PdM draft) A machine installed from a release (fv86) serves the UI that matches its bridle version, and a self-upgrade moves both. (The dev machine following landings, tc7t/785a, moved to 1c jw75 on 2026-10-17, the human's OK.) Done 2026-10-17.

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-k3jk](http://dalek.tailbc91f5.ts.net:7878/task?id=br-k3jk) | bridle-ui ships with bridle releases: design pass (designer): options and a recommendat... |  | integrated | delivered |
| [br-t3dr](http://dalek.tailbc91f5.ts.net:7878/task?id=br-t3dr) | bridle-ui ships with each bridle release, fixed to that version, and self-upgrade moves... |  | integrated | delivered |
| [br-785a](http://dalek.tailbc91f5.ts.net:7878/task?id=br-785a) | A landing that needs a UI install shows without a manual step (tc7t, option C) |  | pending | design first (designer on tc7t, then the human's review); build split after br-7ufd lands |

#### Epic `comment-routing`: Comments and mentions reach the right agent

- Outcome: The human's comments and replies reach the agent that should act: a document's reviewer front matter (re79) and @-mentions in replies and comments (4cgx). The human, 2026-10-09: "they are related".
- Done when: (PdM draft) A comment on a document with `reviewer:` reaches that agent, and an @-mention in a ticket reply or document comment reaches the named role, external roles included.

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-fd8e](http://dalek.tailbc91f5.ts.net:7878/task?id=br-fd8e) | Commit every edit bridle makes to a reviewed document or ticket (status markers, thread... |  | integrated | delivered |
| [br-re79](http://dalek.tailbc91f5.ts.net:7878/task?id=br-re79) | Comments on a ticket an agent asked the human to approve go back to that agent; a pendi... | low | pending | waits on the human (approve to ready) |
| [br-4cgx](http://dalek.tailbc91f5.ts.net:7878/task?id=br-4cgx) | @-mention a role in a ticket reply or a document comment and that role gets a message | low | pending | waits on the human (approve to ready) |
| [br-g8pe](http://dalek.tailbc91f5.ts.net:7878/task?id=br-g8pe) | Link-reminder hook: design pass (designer): how hooks work, options and a recommendatio... |  | integrated | delivered |
| [br-9dcz](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9dcz) | Agents are reminded once to link the ticket and task IDs they show the human (a hook, n... |  | pending | waits on the human (approve to ready) |

#### Epic `ui-cleanups`: bridle-ui enhancements and fixes

- Outcome: Small cleanups and fixes in the bridle UI, low priority, picked up between larger work. The human, 2026-10-10 ~11:10 AM ET (via the bridle-ui aide): "we should have a theme on bridle-ui for enhancements and fixes if we don't have one, and it's just cleanups."
- Done when: (PdM draft) Ongoing: no end state; the list is worked down between epics.

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [ui-sxs3](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-sxs3) | Picking a document from the Document page's dropdown opens it (no Open click); a bridle... | low | pending | waits on the human (approve to ready) |
| [ui-ha6m](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-ha6m) | The human can delete a resolved comment thread (kept in git history) |  | integrated | delivered |
| [ui-wtr3](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-wtr3) | An expanded comment thread can be collapsed again (a [-] button) |  | integrated | delivered |

#### Not in an epic

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-bnfs](http://dalek.tailbc91f5.ts.net:7878/task?id=br-bnfs) | Deep links to a human to-do open the task page and fail; open the to-do page instead, a... | low | pending | waits on the human (approve to ready) |
| [ui-jafg](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-jafg) | Send-to-agent button on a task (UI half of rk7k) | low | integrated | delivered |
| [br-kaez](http://dalek.tailbc91f5.ts.net:7878/task?id=br-kaez) | The bridle report in the daily mail digest (rcvb slice 2) | low | pending | waits on the human (approve to ready) |
| [br-1665](http://dalek.tailbc91f5.ts.net:7878/task?id=br-1665) | A web UI for the human: my to-dos and decisions, to run through and check off |  | pending | waits on the human (approve to ready) |
| [br-pa8h](http://dalek.tailbc91f5.ts.net:7878/task?id=br-pa8h) | Document review: keep the review list in the database, and scan for unresolved comments... | low | pending | waits on the human (approve to ready) |
| [br-yydm](http://dalek.tailbc91f5.ts.net:7878/task?id=br-yydm) | bridle-ui: a usage page with week-to-week charts of the five-hour and seven-day limits ... |  | pending | waits on the human (approve to ready) |
| [br-enx3](http://dalek.tailbc91f5.ts.net:7878/task?id=br-enx3) | bridle link: the unified URL scheme (/p/{project}/tasks/{id}, ...), document and spec l... |  | dropped | dropped |
| [br-4ge4](http://dalek.tailbc91f5.ts.net:7878/task?id=br-4ge4) | Show the current budget and focus settings from the CLI, consistently |  | pending | waits on the human (approve to ready) |
| [br-44ms](http://dalek.tailbc91f5.ts.net:7878/task?id=br-44ms) | Start an unplanned focus period now, through an agent (for example quiet for sleep) |  | pending | waits on the human (approve to ready) |
| [br-yy88](http://dalek.tailbc91f5.ts.net:7878/task?id=br-yy88) | Interactive sessions reply in quiet hours: the focus gate never reaches background wakes |  | pending | waits on the human (approve to ready) |
| [br-v3b7](http://dalek.tailbc91f5.ts.net:7878/task?id=br-v3b7) | Postmortem: quiet hours didn't reach replies to background wakes |  | pending | waits on the human (approve to ready) |
| [br-9s8u](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9s8u) | Background wakes carry the session's prompt context: quiet hours, and every hook that s... |  | pending | waits on the human (approve to ready) |
| [br-sdrw](http://dalek.tailbc91f5.ts.net:7878/task?id=br-sdrw) | Turn off Claude Code prompt suggestions in every bridle session |  | pending | waits on the human (approve to ready) |
| [br-767c](http://dalek.tailbc91f5.ts.net:7878/task?id=br-767c) | Drop the Write(path) focus-file deny rules: Claude Code warns on every session start |  | pending | waits on the human (approve to ready) |
| [ui-hu3k](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-hu3k) | Review the document picker prototypes and choose A, B or C (ui-m2pz) |  | claimed | the human's to-do |
| [ui-7jg4](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-7jg4) | [at restart] Test the new comment selection (ui-bpsd) on laptop and phone |  | claimed | the human's to-do |
| [ui-vnuu](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-vnuu) | Comment IDs never repeat after deletes: a counter in the document's front matter | low | integrated | delivered |
| [br-gd43](http://dalek.tailbc91f5.ts.net:7878/task?id=br-gd43) | Comment IDs never repeat after deletes: assign_ids reads and bumps a front-matter count... | low | integrated | landed (bridle half of ui-vnuu) |
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

### Theme `email`: Email (future)

Future, not current work: no tickets yet. The human, 2026-10-16 (via aide): "ask PdM to have a
future theme on email including that and images", after "Don't need email to multiple projects
yet" earlier that day.

- **One email to several projects.** Today it reaches only the project whose bridge polls first:
  the bridges share one S3 object, deleted after delivery.
- **Images in mail**, both ways.

### The human's to-dos

Chores assigned to the human: expand on an idea, review a ticket or branch.

| Task | Title | Pri | State | Next |
|---|---|---|---|---|
| [br-x7fx](http://dalek.tailbc91f5.ts.net:7878/task?id=br-x7fx) | Turn self_upgrade back on ([daemon] self_upgrade = true in bridle's .bridle/config.toml... |  | integrated | delivered |
| [br-a3b9](http://dalek.tailbc91f5.ts.net:7878/task?id=br-a3b9) | Sat 10-03: review and land the parked branches (br-6b8a, br-2718, br-2672, br-8b98, br-... |  | integrated | done (the human's to-do); ticket still open |
| [br-7575](http://dalek.tailbc91f5.ts.net:7878/task?id=br-7575) | Expand on 8r5x: keep your interactive session logs (which sessions, how long, where) |  | claimed | the human's to-do |
| [br-3a42](http://dalek.tailbc91f5.ts.net:7878/task?id=br-3a42) | Expand on z485: make the orchestrator non-interactive (what it gives you, who you talk ... |  | claimed | the human's to-do |
| [br-da2b](http://dalek.tailbc91f5.ts.net:7878/task?id=br-da2b) | Expand on 2tpm: evaluate Go instead of Rust (why, what would decide it) |  | claimed | the human's to-do |
| [br-9667](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9667) | Expand on 67qw: improve the base system's architecture (which parts, what's wrong) |  | claimed | the human's to-do |
| [br-tkph](http://dalek.tailbc91f5.ts.net:7878/task?id=br-tkph) | Follow up on the workflow review: 6 decisions, 2 waiting on others (34bw, vp9e, sk52, 7... |  | claimed | the human's to-do |
| [br-efs2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-efs2) | Review and comment on kuw2, the machine daemon design (docs/tickets/open/a-machine-daem... |  | claimed | the human's to-do |
| [br-cfb2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-cfb2) | Review ticket v8uu: seeing what background agents do (findings + 5 possible features) |  | claimed | the human's to-do |
| [br-twg8](http://dalek.tailbc91f5.ts.net:7878/task?id=br-twg8) | Try document review (x8jt) on gtzx: install the UI, review add, start the gateway, comment |  | integrated | done (the human's to-do); ticket still open |

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
| 2026-10-09 ~17:40 | br-8c25 bridle token pair, part 1 (role tokens across machines over SSH) | machine setup |
| 2026-10-09 ~18:45 | br-k6jd managers and the orchestrator fetch origin; divergence warned (N ahead, M behind) | machine setup |
| 2026-10-09 ~18:56 | br-jw9e bridle token pair, part 2 (peer tokens, [mail] peers opt-out, pairing on project creation) | machine setup |
| 2026-10-09 ~19:37 | br-tnyt load notes: quiet period, 30 min gap, one per machine | performance |
| 2026-10-09 ~21:32 | br-7ufd self-upgrade batched, at most every 3 h | reliability |
| 2026-10-09 ~22:33 | br-m5n2 data-contracts onboarded (daemon on 7407, dc- tasks filed; done in data-contracts) | onboarding |
| 2026-10-10 morning | br-fpde restart stays up when exec fails | reliability |
| 2026-10-10 morning | br-n96z, br-4vmc, br-37r9 (a landing no longer removes a worker that holds other work) | reliability |
| 2026-10-10 ~10:15 | br-hdbj only the owner's clone pushes the integration branch | machine setup |
| 2026-10-10 ~10:35 | br-88d4 self_upgrade = "release" (chvf 2) | machine setup |
| 2026-10-10 ~10:40 | br-re57 benchmark design (designer) | benchmarking |
| 2026-10-10 ~12:50 | br-57ec benchmark script | benchmarking |
| 2026-10-10 ~13:07 | br-ygkc release test checksum-mismatch flake fixed | machine-setup |
| 2026-10-10 ~13:50 | br-751e daemon keeps its workflow checkout at the binary's tag (chvf 3) | machine-setup |
| 2026-10-10 ~14:19 | br-yfv5 one scheduler for timed actions (design) | scheduled-messages |
| 2026-10-10 ~14:22 | br-g8pe link-reminder hook design pass (9dcz) | comment-routing |
| 2026-10-10 ~15:04 | br-hwek a named advisor's unread mail stays in its own inbox | messaging |
| 2026-10-10 ~15:17 | br-bcw6 cli_e2e sigint and events_stream shutdown flakes | reliability |
| 2026-10-10 ~15:45 | br-fv86 install bridle from a release with no clone | machine-setup |
| 2026-10-10 ~16:01 | br-9aj2 message delivery you can check, part 1 | messaging |
| 2026-10-10 ~16:25 | br-36dp machine-wide self_upgrade | machine-setup |
| 2026-10-10 ~16:38 | br-g5y2 scheduled messages slice 2 (role priming) | scheduled-messages |
| 2026-10-10 ~17:05 | br-3zhx message delivery you can check, part 2 | messaging |
| 2026-10-10 ~17:07 | br-k3jk bridle-ui ships with releases: design pass (t3dr) | ui-releases |
| 2026-10-10 ~18:01 | br-syqn ticket fields get standard names (syqn part 1) | everything-is-a-ticket |
| 2026-10-10 ~18:01 | br-2msq send --project to a local daemon; --url with --project | messaging |
| 2026-10-10 ~18:18 | br-ysmu CI watch missed red runs on main | messaging |
| 2026-10-10 ~19:04 | br-g76s load-hold notes (n4w4 recs 4, 5) | performance |
| 2026-10-10 ~19:10 | br-8umh a failed push is an event and an alarm | reliability |
| 2026-10-10 ~19:27 | br-n7cg mail between daemons, slice 2 (forward to home daemon) | messaging |
| 2026-10-10 ~20:24 | br-ks55 one full test run at a time per machine (n4w4 rec 6) | performance |
| 2026-10-10 ~20:25 | br-fzwa audit of periodic daemon loops (n4w4 rec 3) | performance |
| 2026-10-10 ~21:09 | br-3mz4 one orchestrator per machine, in the advisor role and sends | orchestrator-per-machine |
| 2026-10-10 ~21:30 | br-rhba mail between daemons, slice 5 (orchestrator wakes are messages) | messaging |
| 2026-10-10 ~21:50 | br-yw8b fake-claude spawns skip the pyenv shim (n4w4 rec 7) | performance |
| 2026-10-10 ~22:16 | br-cufw mail between daemons, slice 4 (visible state) | messaging |
| 2026-10-10 ~22:46 | br-fd8e commit every edit bridle makes to a reviewed document or ticket | comment-routing |
| 2026-10-10 ~23:32 | br-qac3 mail between daemons, slice 6 (watch through the peer token) | messaging |
| 2026-10-10 ~23:43 | br-7zd4 stale rules-sync lines removed from workflow rules (98xt) | agents-and-cli |
| 2026-10-11 ~00:46 | br-3k7d externals can schedule messages for themselves | scheduled-messages |
| 2026-10-11 ~01:17 | br-ck8a flaky governor_test disabled (ar4d) | reliability |
| 2026-10-11 ~01:28 | br-epvy project-qualified ticket and task IDs (syqn part 2) | everything-is-a-ticket |
| 2026-10-11 ~02:09 | br-wrmu ticket migration (syqn part 3) | everything-is-a-ticket |
| 2026-10-11 ~02:35 | br-bpku every ticket gets a row; readiness moves to the ticket | everything-is-a-ticket |
| 2026-10-11 ~03:26 | br-3v75 ticket links live in the frontmatter | everything-is-a-ticket |
| 2026-10-11 ~05:10 | br-72t9 a ticket's thread moves to the ticket (first slice) | everything-is-a-ticket |
| 2026-10-11 ~05:35 | br-8ay6 direct-to-main docs commits pushed straight after | reliability |
| 2026-10-11 ~06:00 | br-cbbn scheduled nightly restart of an interactive session | scheduled-messages |
| 2026-10-11 ~06:20 | br-q8cs user-facing rename: bridle ticket replaces bridle task | everything-is-a-ticket |
| 2026-10-11 ~06:42 | br-sehn user-facing rename slice 2 (roles, rules, skills, docs) | everything-is-a-ticket |
| 2026-10-11 ~07:03 | br-aejj user-facing rename slice 3 (gateway routes) | everything-is-a-ticket |
| 2026-10-11 ~07:52 | br-gd43 comment IDs never repeat after deletes (bridle half of ui-vnuu) | human-ui |
| 2026-10-11 ~07:58 | br-jstr recipient hint for a bare advisor/<name> | agents-and-cli |
| 2026-10-11 ~08:22 | br-95t9 code rename: no type is called Task | everything-is-a-ticket |
| 2026-10-11 ~09:14 | br-92za code rename slice 2 (wire types) | everything-is-a-ticket |
| 2026-10-11 ~09:39 | br-res3 code rename slice 3 (internal identifiers) | everything-is-a-ticket |
| 2026-10-11 ~10:26 | br-ft3b per-role handover instructions in the workflow | scheduled-messages |
| 2026-10-11 ~11:03 | br-e7e2 migration: backfill ticket kind and two-way links (v3dk slice B) | migrations |
| 2026-10-11 ~11:31 | br-cy4q human to-dos are normal tickets (22ab step 7b) | everything-is-a-ticket |
| 2026-10-11 ~14:19 | br-e9wg push alarm on a one-off race; git's reason shown | reliability |
| 2026-10-11 ~14:40 | br-dd79 workers' final just check no longer runs twice | reliability |
| 2026-10-11 ~15:30 | br-w6ww a land cut off by a restart no longer strands the task | reliability |
| 2026-10-11 ~15:33 | br-6rz4 ticket new: no second empty '## The ask' | tickets-and-release |
| 2026-10-11 ~16:10 | br-ydyk ticket new in another project writes the right id prefix | tickets-and-release |
| 2026-10-11 ~16:33 | br-stx8 held and built are task states | everything-is-a-ticket |
| 2026-10-11 ~16:53 | br-zqnp bridle link emits the ticket page URL | tickets-and-release |
| 2026-10-11 ~17:11 | br-e4rv stx8 part 2: held and built labels in the TUI and gateway | everything-is-a-ticket |
| 2026-10-11 ~17:51 | br-uwcw flaky renew_test (macOS) | reliability |
| 2026-10-11 ~19:26 | br-t3dr bridle-ui ships with each bridle release | ui-releases |
| 2026-10-11 ~20:51 | br-ju5q benchmark report | benchmarking |
| 2026-10-11 ~21:35 | br-eurm token pair mints visitors with a home | kx5u |
| 2026-10-11 ~23:28 | br-2wnn compare and sync the [projects] list across machines | kx5u |
| 2026-10-11 ~23:58 | br-6b8a orchestrator relaunch liveness (jf9u) | reliability |
| 2026-10-12 ~00:43 | br-btdn restart and self-upgrade no longer blocked by a stuck spawning flag | reliability |
| 2026-10-12 ~01:16 | br-ga6s 53gy A: one-shot take-over, owner.toml machine identity | kx5u |
| 2026-10-12 ~01:55 | br-ajp5 a release daemon no longer guesses bridle's repo from origin | kx5u |
| 2026-10-12 ~02:16 | br-n4ug 53gy B: project stop with confirmed pushes and released mark | kx5u |
| 2026-10-12 ~02:53 | br-wayb ticket links from bridle link open (bare ids) | tickets-and-release |
| 2026-10-12 ~02:57 | br-86qp 53gy C: project start, token re-pair, scheduled messages carried over | kx5u |
| 2026-10-12 ~02:59 | br-kt25 project move design pass | kx5u |
| 2026-10-12 ~05:45 | br-g5dm products design pass | products |
| 2026-10-12 ~06:07 | br-k9j4 grilling skill design | product-process |
| 2026-10-12 ~09:06 | br-ve74 a handover request reaches the new session | reliability |
| 2026-10-12 ~09:15 | br-mtph bridle-grill skill built | product-process |
| 2026-10-12 ~09:37 | br-10c0 orchestrator identity and disaster recovery (7d62) | reliability |
| 2026-10-12 ~10:05 | br-3ch7 K1: refuse a move while committed config has absolute paths | kx5u |
| 2026-10-12 ~10:41 | br-bb2n K2: project preflight | kx5u |
| 2026-10-12 ~11:09 | br-bs85 bridle-grill asks how the human will answer | product-process |
| 2026-10-12 ~13:08 | br-3v2m K6: mail bridge and gateway follow a moved project | kx5u |
| 2026-10-12 ~13:17 | br-mzy4 K3: project stop refuses unless the project is stable | kx5u |
| 2026-10-12 ~13:55 | br-kefh K4: project drain | kx5u |
| 2026-10-12 ~14:07 | br-d6h6 K9: return trip (old database backed up, outbox counter carried) | kx5u |
| 2026-10-12 ~14:30 | br-3vv3 K5: after a stop, the old machine can never serve the project | kx5u |
| 2026-10-12 ~14:51 | br-ep5z K10: the MVP runbook | kx5u |
| 2026-10-12 ~15:19 | br-yt5v K4 follow-up: the drain's hold survives a restart | kx5u |

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
- 2026-10-09 19:25 ET: re79 refined (reviewer front matter; the human), task br-re79 low. New epic `comment-routing` (theme human-ui, low) with br-re79 and br-4cgx, before `products`.
- 2026-10-09 17:41 ET: br-8c25 (token pair part 1) delivered; br-jw9e (part 2) now ready to build.
- 2026-10-09 18:01 ET: br-v6kr baseline set to start 4:00 AM ET 2026-10-10 (the human, via the orchestrator).
- 2026-10-09 18:02 ET: br-d48r (machine load notes go to aide; the human, via the orchestrator) placed in agents-and-cli, no epic.
- 2026-10-09 18:03 ET: filed by the aide from the human's words, readied, high: br-7ufd (self-upgrade at most every few hours, batched; critical fixes at once), theme reliability.
- 2026-10-09 18:03 ET: br-d48r dropped (orchestrator misread the human); the aide files the real ticket (load notes repeat with no quiet period).
- 2026-10-09 18:05 ET: the human turned self_upgrade off until br-7ufd lands; their to-do br-x7fx (turn it back on) added.
- 2026-10-09 18:05 ET: filed by the aide from the human's words, readied, high: br-tnyt (load notes: quiet period, hysteresis, one per machine), theme performance.
- 2026-10-09 18:29 ET: br-jw9e (token pair part 2) built; waits on the human's landing review. br-g76s now blocked by br-tnyt (same file).
- 2026-10-09 18:57 ET: br-k6jd and br-jw9e (token pair part 2, the human: "land jw9e") delivered. br-tnyt planned.
- 2026-10-09 19:11 ET: br-8z7j hold released (dotfiles hooks cleaned up); its build is br-hdbj (planned), br-8z7j dropped. Ticket sweep: 96 tickets whose tasks were all done resolved (the human: the PdM resolves tickets).
- 2026-10-09 19:28 ET: follow-ups from the ticket sweep (the human approved): readied br-fpde (restart stays up when exec fails; reliability) and br-785a (UI install follows landings, tc7t option C; tickets-and-release), normal; readied low ui-jafg (rk7k UI button; human-ui), br-sfpg (researcher role, 2mtr; agents-and-cli), br-7zd4 (stale rule lines, 98xt; agents-and-cli); filed unscheduled br-kaez (rcvb slice 2; human-ui) and br-zpc7 (gdyy part 2; reliability). None in an epic.
- 2026-10-09 19:38 ET: br-tnyt delivered, ticket tnyt resolved; br-g76s unblocked. Placed br-8ff8 (CI flake, critical) in reliability. br-785a waits on a designer pass on tc7t (queued); br-sfpg needs a designer later.
- 2026-10-09 20:47 ET: br-syqn planned as part 1 of 3 (pm-1 files parts 2 and 3 when it lands).
- 2026-10-09 23:05 ET: br-7ufd delivered, ticket 7ufd resolved; data-contracts onboarded (m5n2 resolved). Queue re-tiered by pm-1 to the roadmap (88d4, fpde, 751e, hdbj ahead) after the human found low work running ahead of machine setup (trial lesson 14). New epic `messaging` (rank 2: 3haz, fvkq, 2msq, n7cg, new rhba, ysmu, cufw); `scheduled-messages` moved to rank 3; br-g5y2 approved. br-1ddd and br-kuvh dropped (the human chose the real fix). Placed br-xxq8, br-p29s, br-bnfs, br-4vmc.
- 2026-10-10 1:45 PM ET: regenerated (stale since 2026-10-09 23:05). Machine setup split (the human: "Split the epics and write a proper 'when done' statement"): `machine-setup` is phase 1 only, done when v0.6.0 is out, the PC runs bridle from the release with no clone, the PC and NUC self-upgrade by release, and chvf and mrhe are resolved; br-3932 (mrhe umbrella) and br-ygkc moved in. New epic `machine-sync` (rjd5, f8f9, xccp, v7ug) after one orchestrator per machine; kt25 to `machine-transparent`; eadm unscheduled. Also new today: epics benchmarking, background-agents, orchestrator-per-machine, machine-transparent, ui-cleanups; hwek (incident fix, messaging); 9dcz and g8pe (link hook, comment-routing).
- 2026-10-10 2:10 PM ET: the human expects machine setup to cover installing bridle with no clone, running and self-upgrading (daemon, gateway, UI fixed to a bridle version). Filed fv86 (install from a release, no clone; readied, high) into `machine-setup` and its done-when; new epic `ui-releases` (t3dr, with br-785a) fifth, the UI being a separate piece of work (the human).
- 2026-10-10 2:40 PM ET: xccp out of machine-sync to unscheduled (the human hadn't decided it; aide wrote option A, no new credentials, into the ticket). Design task br-k3jk added to ui-releases.
- 2026-10-10 1:55 PM ET: stx8 (task states) into `everything-is-a-ticket`, last, blocked by br-72t9 (the human: "do all the ticket task work first because they're dependent and interrelated ... it can come later"); off the reviews list for now. Note: today's earlier PM entries above were stamped about an hour late (the previous session's ET clock was off).
- 2026-10-10 2:05 PM ET: rule for epics (the human approved): a ticket is in an epic only if it is on the path to the epic's done-when; ideas, questions and investigations not yet committed to stay in their theme, not in an epic. br-d9wq (PdM research) out of everything-is-a-ticket into product-process, not in an epic.
- 2026-10-10 2:10 PM ET: new p896 (agent evaluations, the human: long-term, low, not now) in agents-and-cli, not in an epic. br-g9xe, br-751e and br-3932 delivered.
- 2026-10-10 2:25 PM ET: v0.6.0 tagged. New 36dp (machine-wide self_upgrade, from the orchestrator) into `machine-setup`, readied high: the NUC's switch to release upgrades waits on it, so no commits to the human's existing projects.
- 2026-10-10 7:30 PM ET: regenerated. 36dp, chvf, hwek resolved; fv86, 3zhx, g5y2, syqn, 2msq, ysmu, bcw6, g76s landed; fd8e placed in comment-routing. New ve74 (handover request reaches the new session; the human via aide: investigate, low) in reliability, not in an epic.
- 2026-10-10 10:20 PM ET: 8umh, 3mz4, n4w4 resolved (n4w4: all 9 recommendations landed). 3haz slices 4 and 5 landed; 3haz stays open for slice 6, br-qac3 (readied, the human's Q3), in `messaging`. syqn parts 2 and 3 (br-epvy, br-wrmu) readied, in `everything-is-a-ticket` after br-syqn.
- 2026-10-11 12:07 AM ET: 95mu moved from everything-is-a-ticket to reviews-enforced, renamed "Gates: design before build, review before landing" (the human: "yes, agreed move it"). cbbn design decided (plain [[restart]] config), readied; new 3k7d (externals schedule for themselves, high, tonight) in scheduled-messages; new jstr (recipient hint, low) in agents-and-cli. 3haz, fd8e resolved; qac3, fd8e landed.
- 2026-10-12 5:30 PM ET: tables regenerated (stale since 10-10); Delivered list brought up to date (epic kx5u MVP complete: K1-K6, K9, K10 and yt5v; K7 is V1, later; K8 moved to epic ehg7).
- 2026-10-12 6:50 PM ET: new epic `usage` (br-rdkn, theme performance), first in order: 9y7v (negative costs), hd28 (count external sessions; answers 368g), yydm (limit charts), all readied on the human's go.
- 2026-10-12 6:55 PM ET: usage epic: yydm narrowed to the gateway route, ui-u6uz filed (UI page); new xfy7 + ui-mxkj (chart one agent instance; the human), done-when item 4.
- 2026-10-12 7:16 PM ET: br-yydm and ui-u6uz delivered and resolved (the Usage page shows on dalek after its next self-upgrade). New br-kyjg (bug: `ticket built` fails with a bare 'unknown' on bridle-ui; from the orchestrator) in reliability, not in an epic, pending the human.
- 2026-10-17 6:20 PM ET: epic 2 `ui-releases` done: br-785a (dev machine installs bridle-ui landings, tc7t option C) folded into 1c (jw75), the human: "Ok sure". tc7t resolved. Stages MVP (tj7q) children 9mqh, quvh, ussh, ewhy, 6c8x readied high and first in the work order (the human, ~5:15 PM ET).
