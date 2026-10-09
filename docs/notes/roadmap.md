# Roadmap

Status: working notes, kept by hand by advisor (product-manager) during the PdM trial
(`docs/notes/product-manager-trial.md`). Treat it as a database: it changes whenever a task is
delivered, planned, held or dropped, or a workstream pauses. Later this becomes a bridle feature.

Terms (provisional): a **workstream** is a goal-sized area of work with its own sequence; each
open task sits in exactly one. Research on the usual PdM terms (theme, initiative, epic, bet) is
under way and may rename these.

How to read the tables: **State** is the task's state in bridle. **Next** is what has to happen
before it moves: "waits on the human" is a pending task only the human can approve; "to plan"
is the project manager's; "ready to build" is in the queue; "HELD" is planned in bridle but
stopped by the human.

Last updated: 2026-10-09 15:40 ET.

## Needs the human now

In the order the PdM suggests:

0. **Machine setup** (your priority, 2026-10-09): two decisions unblock phase 2. (a) done 2:20 PM: sk7p
   design approved; 8c25 being re-briefed. (b) j7r4: answered 2:05 PM (one pusher, enforced:
   8z7j to the designer; k6jd, 8ay6, 8umh readied; xccp later). Also your review of kuw2
   (machine daemon, br-efs2) gates project transfer's design.

0b. **Everything is a ticket** (22ab, new workstream): approve the plan in the ticket ("Plan"
   section), and answer its Q1-Q5 (field names; epic or initiative, which also settles d9wq; ticket
   layout; a row per ticket; purging resolved/). Step 1 waits on Q1.
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
7. **Term for these groupings** (d9wq, PdM research): keep "workstream" or switch to
   "initiative" (an outcome and a "Done when" per grouping), with "theme" as a tag?

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

## Workstreams

### Everything is a ticket

The human, 2026-10-09: one record per piece of work; tickets that change code are tracked as rows with a thread on the state branch; fields renamed; task renamed to ticket in names and code. Ticket 22ab holds the decisions, design and plan (child tickets; Q1-Q5 answered, step 5 stx8 moved out); the plan waits on the human's approval.

| Task | Title | State | Next |
|---|---|---|---|
| [br-d9wq](http://dalek.tailbc91f5.ts.net:7878/task?id=br-d9wq) | Research: product manager roles, human and agentic (BMAD and others), and what to call ... | pending | research done; the human reads it and picks the term |
| [br-95mu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-95mu) | A change spec (proposal and design) reviewed for risk and impact before any worker buil... | pending | to the designer once the human has reviewed the ticket |
| [br-stx8](http://dalek.tailbc91f5.ts.net:7878/task?id=br-stx8) | A task's state says what's really happening: held and built-awaiting-landing are states... | planned | HELD: needs a proposed design and the human's approval |

### Machine setup (PRIORITY)

The human, 2026-10-09: efficient, direct setup of a new machine, first the Windows PC: background daemons, tokens between machines and between projects, sync; then moving a project between machines. Phase 1 makes the PC usable; phase 2 one command to set up and sync; phase 3 project transfer.

| Task | Title | State | Next |
|---|---|---|---|
| [br-jgdb](http://dalek.tailbc91f5.ts.net:7878/task?id=br-jgdb) | Windows PC: follow the WSL2 setup guide (install WSL2 + Ubuntu, wslconfig, Tailscale, s... | claimed | the human's to-do |
| [br-h7mu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-h7mu) | Pick a name for the Windows PC (docs/context/naming.md) (v7ug) | claimed | the human's to-do |
| [br-hua2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-hua2) | Add a machine: one setup guide from bare OS to on the network (config, tokens, services... | integrated | delivered |
| [br-gdf3](http://dalek.tailbc91f5.ts.net:7878/task?id=br-gdf3) | Peer-token setup guidance: a token per receiving project per sending machine, minted on... | integrated | delivered |
| [br-8c25](http://dalek.tailbc91f5.ts.net:7878/task?id=br-8c25) | bridle token pair, part 1: the token-role list, the spec, and role tokens across machin... | planned | design approved (sk7p); pm-1 re-briefs, then build |
| [br-jw9e](http://dalek.tailbc91f5.ts.net:7878/task?id=br-jw9e) | bridle token pair, part 2: peer tokens, the [mail] peers opt-out, and pairing on projec... | planned | ready to build |
| [br-88d4](http://dalek.tailbc91f5.ts.net:7878/task?id=br-88d4) | self_upgrade = "release": fetch, verify and swap the release binary (chvf 2) | planned | ready to build |
| [br-751e](http://dalek.tailbc91f5.ts.net:7878/task?id=br-751e) | Daemon keeps its own workflow checkout at the binary's tag (chvf 3) | planned | ready to build |
| [br-57nt](http://dalek.tailbc91f5.ts.net:7878/task?id=br-57nt) | bridle gateway restart takes a launchd-managed gateway out of launchd and inherits the ... | integrated | delivered |
| [br-xrkh](http://dalek.tailbc91f5.ts.net:7878/task?id=br-xrkh) | systemd uninstall, and an owner refusal never crash-loops a launchd or systemd unit aft... | planned | ready to build |
| [br-rjd5](http://dalek.tailbc91f5.ts.net:7878/task?id=br-rjd5) | Commands to set up and sync a project everywhere: sync all seven places per machine, an... | pending | phase 2: needs a design (source of truth for the project list), then the human's approval |
| [br-f8f9](http://dalek.tailbc91f5.ts.net:7878/task?id=br-f8f9) | The NUC recovers everything on boot (4r3k) | pending | phase 2: 'bridle up' after boot needs a small design |
| [br-v7ug](http://dalek.tailbc91f5.ts.net:7878/task?id=br-v7ug) | Run bridle's heavy work on the Windows PC under WSL2 | pending | umbrella; audit and guide delivered |
| [br-kt25](http://dalek.tailbc91f5.ts.net:7878/task?id=br-kt25) | Move a project between machines with one command (bridle project move) | pending | phase 3: needs a design and the human's decisions (see ticket) |
| [br-8z7j](http://dalek.tailbc91f5.ts.net:7878/task?id=br-8z7j) | Only the owner's clone can push the integration branch: enforced, not a rule | reopened | Option A approved; blocked until dotfiles-local shared git hooks are removed on both machines (dotfiles-local aide, NUC) |
| [br-k6jd](http://dalek.tailbc91f5.ts.net:7878/task?id=br-k6jd) | Managers and the orchestrator fetch origin; divergence from origin is warned (N ahead, ... | planned | ready to build |
| [br-xccp](http://dalek.tailbc91f5.ts.net:7878/task?id=br-xccp) | A git identity per machine, so commits show which clone made them | pending | later (the human: non-urgent); needs the human's keys or tokens |

### Product process and gates

How work gets from an idea to ready: the PdM, the designer, change specs and reviews, task states. The ladder the human wants for every non-trivial feature.

| Task | Title | State | Next |
|---|---|---|---|
| [br-6h65](http://dalek.tailbc91f5.ts.net:7878/task?id=br-6h65) | A product manager that relates every ticket to open and planned work: links, merges, an... | pending | this trial is the stand-in (docs/notes/product-manager-trial.md) |
| [br-gtzx](http://dalek.tailbc91f5.ts.net:7878/task?id=br-gtzx) | Seats: every role is a named, tracked seat that outlives its sessions, with its own inb... | open | HELD: waits on the human's review of P1-P10, Q1-Q4 |
| [br-91b3](http://dalek.tailbc91f5.ts.net:7878/task?id=br-91b3) | Reviews enforced by bridle, signed on the task (v2va): umbrella, slices A-D | planned | ready to build |
| [br-6ba3](http://dalek.tailbc91f5.ts.net:7878/task?id=br-6ba3) | Reviews A: in_review state, ready-for-review, review requirements on tasks (v2va slice A) | planned | ready to build |
| [br-46fe](http://dalek.tailbc91f5.ts.net:7878/task?id=br-46fe) | Reviews B: signed review records on the task (v2va slice B) | planned | ready to build |
| [br-cf00](http://dalek.tailbc91f5.ts.net:7878/task?id=br-cf00) | Reviews C: the daemon spawns the required reviewers; comment-only talk; cost recorded (... | planned | ready to build |
| [br-f610](http://dalek.tailbc91f5.ts.net:7878/task?id=br-f610) | Reviews D: the gate: no landing until every required review is signed (v2va slice D) | planned | ready to build |
| [br-519b](http://dalek.tailbc91f5.ts.net:7878/task?id=br-519b) | Task watchers: a creator field, a watchers list, and wakes that say what changed and ar... | pending | waits on the human (approve to ready) |
| [br-cr7t](http://dalek.tailbc91f5.ts.net:7878/task?id=br-cr7t) | Add a postmortem ticket kind: the full write-up after an incident | pending | waits on the human (approve to ready) |
| [br-avu7](http://dalek.tailbc91f5.ts.net:7878/task?id=br-avu7) | vk3y slice 2: 'bridle task new' requires --ticket (no-ticket sentinel) and a rule for e... | pending | waits on the human (approve to ready) |

### Scheduled messages and timed actions

Ship scheduled messages and one scheduler for timed actions (yfv5).

| Task | Title | State | Next |
|---|---|---|---|
| [br-9xze](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9xze) | Scheduled messages, first slice: an agent schedules a message to itself (one-time or re... | integrated | delivered |
| [br-g5y2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-g5y2) | Scheduled messages slice 2: role priming: wait at the maximum timeout and schedule a me... | pending | waits on the human; 9xze has landed |
| [br-yfv5](http://dalek.tailbc91f5.ts.net:7878/task?id=br-yfv5) | One scheduler for timed actions: scheduled messages (hrcn), nightly session restarts (c... | pending | needs a design (designer) for the rest: cbbn, 3nyk, cy2v; then the human's review |
| [br-cbbn](http://dalek.tailbc91f5.ts.net:7878/task?id=br-cbbn) | Scheduled nightly restart of an interactive session at a clock time (e.g. 3 AM) | pending | waits on the human (approve to ready) |
| [br-ft3b](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ft3b) | Per-role handover instructions in the workflow, with project overrides | pending | waits on the human (approve to ready) |

### Documents and attachments on tickets and tasks

Several documents (a design, specs) and later images on one ticket or task; the human's MIME idea. No ticket yet.

No tasks yet.

### Orchestrator and daemon reliability

The orchestrator and daemons stay up, relaunch once, upgrade cleanly and recover after sleep or reboot.

| Task | Title | State | Next |
|---|---|---|---|
| [br-4zfa](http://dalek.tailbc91f5.ts.net:7878/task?id=br-4zfa) | Incident: the bridle orchestrator was killed (SIGTERM) at 11:18 PM ET and nothing relau... | pending | waits on the human (approve to ready) |
| [br-f4xu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-f4xu) | Flaky on macOS CI: process_test sigterm_via_signal_group_exits_143 exits 1, not 143 | integrated | delivered |
| [br-h7gt](http://dalek.tailbc91f5.ts.net:7878/task?id=br-h7gt) | Flaky on Linux CI: upgrade_test self_upgrade_restarts_only_after_the_mid_turn_agent_fin... | integrated | delivered |
| [br-8ay6](http://dalek.tailbc91f5.ts.net:7878/task?id=br-8ay6) | Direct-to-main docs commits are pushed straight after, on the owner's clone | planned | ready to build |
| [br-8umh](http://dalek.tailbc91f5.ts.net:7878/task?id=br-8umh) | A failed push is an event and an alarm; an agent that can't send puts the blocker on th... | planned | ready to build |
| [br-vabu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-vabu) | Flaky on Linux CI: store cancelled_blocking_task_returns_shutting_down_not_a_panic | integrated | delivered |
| [br-b6mu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-b6mu) | Self-upgrade drain may never restart when it starts during a spawn | integrated | delivered |
| [br-6b8a](http://dalek.tailbc91f5.ts.net:7878/task?id=br-6b8a) | Orchestrator relaunch liveness: one clock, never a second orchestrator (jf9u) | planned | ready to build |
| [br-96a6](http://dalek.tailbc91f5.ts.net:7878/task?id=br-96a6) | Hold the orchestrator relaunch without restarting the daemon (8fsx) | pending | waits on the human (approve to ready) |
| [br-9966](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9966) | bridle orchestrator hold / release: runtime switch for the relaunch (8fsx) | planned | ready to build |
| [br-10c0](http://dalek.tailbc91f5.ts.net:7878/task?id=br-10c0) | Orchestrator identity and disaster recovery (7d62) | pending | waits on the human (approve to ready) |
| [br-btdn](http://dalek.tailbc91f5.ts.net:7878/task?id=br-btdn) | Daemon restart and self-upgrade blocked forever: manager.spawning() stays true with no ... | pending | waits on the human (approve to ready) |
| [br-y455](http://dalek.tailbc91f5.ts.net:7878/task?id=br-y455) | Incident: bridle's daemon couldn't restart or self-upgrade for ~23 h: a stuck 'spawning... | planned | ready to build |
| [br-aqa7](http://dalek.tailbc91f5.ts.net:7878/task?id=br-aqa7) | Self-upgrade refused good builds 3 times overnight: the new binary's self-check timed o... | pending | waits on the human (approve to ready) |
| [br-5j35](http://dalek.tailbc91f5.ts.net:7878/task?id=br-5j35) | bridle session restart run from inside the session stops it and never relaunches: the d... | pending | waits on the human (approve to ready) |
| [br-s5ah](http://dalek.tailbc91f5.ts.net:7878/task?id=br-s5ah) | A daemon without an aide leaves its orchestrator no route to the human | pending | waits on the human (approve to ready) |
| [br-2ax5](http://dalek.tailbc91f5.ts.net:7878/task?id=br-2ax5) | Incident: br-3haz broke every cross-project message for ~22 h: the CLI needed an outbox... | planned | ready to build |
| [br-zcqv](http://dalek.tailbc91f5.ts.net:7878/task?id=br-zcqv) | dalek slept in a bag 8:42 AM-1:38 PM ET on 10-06: bridle froze, but the workforce had a... | pending | waits on the human (approve to ready) |
| [br-vn54](http://dalek.tailbc91f5.ts.net:7878/task?id=br-vn54) | Incident: remote control dropped for the dalek aide session after a tmux detach and lap... | pending | waits on the human (approve to ready) |
| [br-ysmu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ysmu) | CI watch missed 13 red runs on main; first ci_failed wake came 40 minutes late | pending | waits on the human (approve to ready) |

### Load and resource cost

Bridle's own cost on the machine: measure against a baseline, then cut it (n4w4, v6kr).

| Task | Title | State | Next |
|---|---|---|---|
| [br-n4w4](http://dalek.tailbc91f5.ts.net:7878/task?id=br-n4w4) | Postmortem: bridle's own 'ps' polling (every daemon, test daemons at 200 ms) drove dale... | pending | waits on the human (approve to ready) |
| [br-v6kr](http://dalek.tailbc91f5.ts.net:7878/task?id=br-v6kr) | A system architect role, and measuring bridle's own resource cost against a baseline | planned | ready; baseline must use the macOS memory measures (ticket) |
| [br-jxwr](http://dalek.tailbc91f5.ts.net:7878/task?id=br-jxwr) | Track and report how long a task takes from pickup to merge, split into agent work, bui... | pending | the human's ask (via aide); would show where the 30-40 min per task goes; low (the human): on a workstream, not queued now |
| [br-6nzj](http://dalek.tailbc91f5.ts.net:7878/task?id=br-6nzj) | Test daemons stop polling at 200 ms; a resource-budget test; log the incident in docs/c... | integrated | delivered |
| [br-g76s](http://dalek.tailbc91f5.ts.net:7878/task?id=br-g76s) | Load-hold notes: one per machine, name bridle-owned top consumers, honest text, load.ho... | planned | ready to build |
| [br-fzwa](http://dalek.tailbc91f5.ts.net:7878/task?id=br-fzwa) | Audit every periodic daemon loop for what it forks or reads per tick; list them with co... | planned | ready to build |
| [br-ks55](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ks55) | Only one full test run at a time per machine: just check takes a machine-wide lock (n4w... | planned | ready to build |
| [br-yw8b](http://dalek.tailbc91f5.ts.net:7878/task?id=br-yw8b) | fake-claude spawns skip the pyenv shim: resolve the interpreter once (n4w4 rec 7) | planned | ready to build |
| [br-z7y5](http://dalek.tailbc91f5.ts.net:7878/task?id=br-z7y5) | Incident: syspolicyd and Spotlight pegged, builds and app launches stalled: each spawn ... | pending | waits on the human (approve to ready) |

### Many projects, many machines

One watcher for all projects, mail between daemons, tokens between machines, the NUC and the Windows PC.

| Task | Title | State | Next |
|---|---|---|---|
| [br-1ddd](http://dalek.tailbc91f5.ts.net:7878/task?id=br-1ddd) | One watcher for every project: 'bridle agent wake --all-projects' | planned | ready to build |
| [br-kuvh](http://dalek.tailbc91f5.ts.net:7878/task?id=br-kuvh) | Remove 'bridle agent wake --all-projects' once 3haz lands and rolls out: warn first, th... | pending | waits on the human (approve to ready) |
| [br-n7cg](http://dalek.tailbc91f5.ts.net:7878/task?id=br-n7cg) | Mail between daemons, slice 2: mail for a visitor is forwarded to its home daemon (3haz... | pending | waits on the human (approve to ready) |
| [br-cufw](http://dalek.tailbc91f5.ts.net:7878/task?id=br-cufw) | Mail between daemons, slice 4: visible state: outbox status, message show (queued/arriv... | pending | waits on the human (approve to ready) |
| [br-3932](http://dalek.tailbc91f5.ts.net:7878/task?id=br-3932) | Run bridle on a project without a local bridle clone (mrhe) | pending | waits on the human (approve to ready) |
| [br-u6w9](http://dalek.tailbc91f5.ts.net:7878/task?id=br-u6w9) | Human interaction time: daemon serves the prompt log; gateway collects across machines ... | pending | waits on the human (approve to ready) |
| [ui-9hq8](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-9hq8) | Documents of projects on another machine (the NUC): read and comment in bridle-ui | planned | ready to build |

### The human's interface: web UI, documents, focus

What the human sees and touches: to-dos, document review, links, usage, quiet hours.

| Task | Title | State | Next |
|---|---|---|---|
| [br-1665](http://dalek.tailbc91f5.ts.net:7878/task?id=br-1665) | A web UI for the human: my to-dos and decisions, to run through and check off | pending | waits on the human (approve to ready) |
| [br-pa8h](http://dalek.tailbc91f5.ts.net:7878/task?id=br-pa8h) | Document review: keep the review list in the database, and scan for unresolved comments... | pending | waits on the human (approve to ready) |
| [br-yydm](http://dalek.tailbc91f5.ts.net:7878/task?id=br-yydm) | bridle-ui: a usage page with week-to-week charts of the five-hour and seven-day limits ... | pending | waits on the human (approve to ready) |
| [br-enx3](http://dalek.tailbc91f5.ts.net:7878/task?id=br-enx3) | bridle link: the unified URL scheme (/p/{project}/tasks/{id}, ...), document and spec l... | planned | ready to build |
| [br-4ge4](http://dalek.tailbc91f5.ts.net:7878/task?id=br-4ge4) | Show the current budget and focus settings from the CLI, consistently | pending | waits on the human (approve to ready) |
| [br-44ms](http://dalek.tailbc91f5.ts.net:7878/task?id=br-44ms) | Start an unplanned focus period now, through an agent (for example quiet for sleep) | pending | waits on the human (approve to ready) |
| [br-yy88](http://dalek.tailbc91f5.ts.net:7878/task?id=br-yy88) | Interactive sessions reply in quiet hours: the focus gate never reaches background wakes | pending | waits on the human (approve to ready) |
| [br-v3b7](http://dalek.tailbc91f5.ts.net:7878/task?id=br-v3b7) | Postmortem: quiet hours didn't reach replies to background wakes | pending | waits on the human (approve to ready) |
| [br-9s8u](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9s8u) | Background wakes carry the session's prompt context: quiet hours, and every hook that s... | pending | waits on the human (approve to ready) |
| [br-sdrw](http://dalek.tailbc91f5.ts.net:7878/task?id=br-sdrw) | Turn off Claude Code prompt suggestions in every bridle session | pending | waits on the human (approve to ready) |
| [br-767c](http://dalek.tailbc91f5.ts.net:7878/task?id=br-767c) | Drop the Write(path) focus-file deny rules: Claude Code warns on every session start | pending | waits on the human (approve to ready) |
| [ui-hu3k](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-hu3k) | Review the document picker prototypes and choose A, B or C (ui-m2pz) | claimed | the human's to-do |
| [ui-7jg4](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-7jg4) | [at restart] Test the new comment selection (ui-bpsd) on laptop and phone | claimed | the human's to-do |
| [ui-ha6m](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-ha6m) | The human can delete a resolved comment thread (kept in git history) | integrated | delivered |
| [ui-wtr3](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-wtr3) | An expanded comment thread can be collapsed again (a [-] button) | integrated | delivered |
| [ui-vnuu](http://dalek.tailbc91f5.ts.net:7878/task?id=ui-vnuu) | Comment IDs never repeat after deletes: a counter in the document's front matter | planned | ready to build |
| [br-gd43](http://dalek.tailbc91f5.ts.net:7878/task?id=br-gd43) | Comment IDs never repeat after deletes: assign_ids reads and bumps a front-matter count... | planned | ready to build |
| [br-twg8](http://dalek.tailbc91f5.ts.net:7878/task?id=br-twg8) | Try document review (x8jt) on gtzx: install the UI, review add, start the gateway, comment | claimed | the human's to-do |

### Agents and the CLI

Agents use bridle correctly: one name per command, clear help, the workflow reaching every agent.

| Task | Title | State | Next |
|---|---|---|---|
| [br-163f](http://dalek.tailbc91f5.ts.net:7878/task?id=br-163f) | Group the CLI's 54 top-level commands; split commands.rs/cli.rs by group (a67t) | reopened | reopened |
| [br-ts6b](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ts6b) | One command tree for interactive sessions: bridle session <verb> <seat>, retiring bridl... | pending | waits on the human (approve to ready) |
| [br-fne2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-fne2) | bridle session advisor and bridle advisor start: near-identical commands that do differ... | pending | the designer's first job (ukpm) |
| [br-abnq](http://dalek.tailbc91f5.ts.net:7878/task?id=br-abnq) | One command for any agent's status and context | pending | waits on the human (approve to ready) |
| [br-m9sd](http://dalek.tailbc91f5.ts.net:7878/task?id=br-m9sd) | Agents don't know how to use the bridle CLI correctly: help, skills, shorter primes or ... | pending | waits on the human (approve to ready) |
| [br-a9g2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-a9g2) | agent spawn --allow-tool silently does nothing for a tool outside the role's --tools se... | pending | waits on the human (approve to ready) |
| [br-79c3](http://dalek.tailbc91f5.ts.net:7878/task?id=br-79c3) | Reserve role names, by prefix, for agent names (2vja) | planned | ready to build |
| [br-9z2d](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9z2d) | Incident: the orchestrator didn't use bridle advisor start or per-advisor addresses, an... | pending | waits on the human (approve to ready) |
| [br-86c6](http://dalek.tailbc91f5.ts.net:7878/task?id=br-86c6) | The workflow doesn't reach agents: resolved rules, hooks and overrides stop at the CLI | pending | waits on the human (approve to ready) |
| [br-h3ar](http://dalek.tailbc91f5.ts.net:7878/task?id=br-h3ar) | Interactive sessions kill each other's wake waiters with pkill -f (exit 144) | pending | waits on the human (approve to ready) |
| [br-mvtz](http://dalek.tailbc91f5.ts.net:7878/task?id=br-mvtz) | Postmortem: sessions killed each other's wake waiters with pkill -f (h3ar) | pending | waits on the human (approve to ready) |
| [br-qdw8](http://dalek.tailbc91f5.ts.net:7878/task?id=br-qdw8) | A haiku worker reported done before its check finished, then lost its task on renewal a... | pending | waits on the human (approve to ready) |
| [br-ytqu](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ytqu) | Postmortem: a haiku worker waited on a check it couldn't see, reported done early, and ... | pending | waits on the human (approve to ready) |

### Tickets, changelog and release

Ticket tooling in every project, migrations, the changelog, self-upgrade from releases.

| Task | Title | State | Next |
|---|---|---|---|
| [br-g3az](http://dalek.tailbc91f5.ts.net:7878/task?id=br-g3az) | Status line token setup in the docs writes an empty file: token create needs --print now | integrated | delivered |
| [br-01ff](http://dalek.tailbc91f5.ts.net:7878/task?id=br-01ff) | Tickets through the bridle binary in every project: new, frontmatter, check, resolve; a... | pending | waits on the human (approve to ready) |
| [br-e7e2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-e7e2) | Migration: backfill ticket kind and two-way task links in every bridle project (v3dk sl... | planned | ready to build |
| [br-ubjd](http://dalek.tailbc91f5.ts.net:7878/task?id=br-ubjd) | A CHANGELOG line for every landed task, written on the branch, and one section per kind... | pending | waits on the human (approve to ready) |

### Executable specs

The spec tooling (built, not wired in) and its rough edges.

| Task | Title | State | Next |
|---|---|---|---|
| [br-pakx](http://dalek.tailbc91f5.ts.net:7878/task?id=br-pakx) | Specs: a way to run executable specs in CI without a bridle checkout | pending | waits on the human (approve to ready) |
| [br-2d6x](http://dalek.tailbc91f5.ts.net:7878/task?id=br-2d6x) | Specs: vitest-bridle setup leaves step files untypechecked | pending | waits on the human (approve to ready) |
| [br-m5kf](http://dalek.tailbc91f5.ts.net:7878/task?id=br-m5kf) | Specs: say what happens to unit tests a scenario now covers | pending | waits on the human (approve to ready) |
| [br-awh4](http://dalek.tailbc91f5.ts.net:7878/task?id=br-awh4) | Specs: step text can't quote code or markup | pending | waits on the human (approve to ready) |
| [br-dbvd](http://dalek.tailbc91f5.ts.net:7878/task?id=br-dbvd) | Specs: bridle spec id duplicates ledger entries for hand-written IDs | pending | waits on the human (approve to ready) |
| [br-s4ve](http://dalek.tailbc91f5.ts.net:7878/task?id=br-s4ve) | vitest-bridle: a step can't skip a scenario at runtime | pending | waits on the human (approve to ready) |

### Onboarding projects and packs

New projects under bridle and the packs they need.

| Task | Title | State | Next |
|---|---|---|---|
| [br-a16f](http://dalek.tailbc91f5.ts.net:7878/task?id=br-a16f) | Onboarding survey: file-db | pending | waits on the human (approve to ready) |
| [br-c265](http://dalek.tailbc91f5.ts.net:7878/task?id=br-c265) | Onboarding survey: track-web and harness (pi) | pending | waits on the human (approve to ready) |
| [br-5299](http://dalek.tailbc91f5.ts.net:7878/task?id=br-5299) | Onboarding survey: otters (otter-life and otters-back) | pending | waits on the human (approve to ready) |
| [br-f070](http://dalek.tailbc91f5.ts.net:7878/task?id=br-f070) | dotfiles-local as a bridle project, directly on main (35mw) | pending | waits on the human (approve to ready) |
| [br-22n2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-22n2) | Web/mobile pack rule: non-prose inputs turn off autocapitalize, autocorrect and spellcheck | pending | waits on the human (approve to ready) |
| [br-9xbk](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9xbk) | Research: web design rule sources to steal from, proposed web/mobile pack rules, attrib... | pending | waits on the human (approve to ready) |
| [tw-5915](http://dalek.tailbc91f5.ts.net:7878/task?id=tw-5915) | Push bridle-adopt to dev and main (deploys production) | claimed | the human's to-do |
| [tw-57dv](http://dalek.tailbc91f5.ts.net:7878/task?id=tw-57dv) | Decide: delete the bridle-adopt branch (track-web now works on dev) | claimed | the human's to-do |
| [tw-mbzn](http://dalek.tailbc91f5.ts.net:7878/task?id=tw-mbzn) | Review the engagement-tracking + feedback design (openspec change games-engagement-and-... | claimed | the human's to-do |

### The human's reading and review queue

Chores that are the human's own: expand on an idea, review a ticket or branch.

| Task | Title | State | Next |
|---|---|---|---|
| [br-a3b9](http://dalek.tailbc91f5.ts.net:7878/task?id=br-a3b9) | Sat 10-03: review and land the parked branches (br-6b8a, br-2718, br-2672, br-8b98, br-... | claimed | the human's to-do; aide asks whether to close |
| [br-7575](http://dalek.tailbc91f5.ts.net:7878/task?id=br-7575) | Expand on 8r5x: keep your interactive session logs (which sessions, how long, where) | claimed | the human's to-do |
| [br-3a42](http://dalek.tailbc91f5.ts.net:7878/task?id=br-3a42) | Expand on z485: make the orchestrator non-interactive (what it gives you, who you talk ... | claimed | the human's to-do |
| [br-da2b](http://dalek.tailbc91f5.ts.net:7878/task?id=br-da2b) | Expand on 2tpm: evaluate Go instead of Rust (why, what would decide it) | claimed | the human's to-do |
| [br-9667](http://dalek.tailbc91f5.ts.net:7878/task?id=br-9667) | Expand on 67qw: improve the base system's architecture (which parts, what's wrong) | claimed | the human's to-do |
| [br-tkph](http://dalek.tailbc91f5.ts.net:7878/task?id=br-tkph) | Follow up on the workflow review: 6 decisions, 2 waiting on others (34bw, vp9e, sk52, 7... | claimed | the human's to-do |
| [br-efs2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-efs2) | Review and comment on kuw2, the machine daemon design (docs/tickets/open/a-machine-daem... | claimed | the human's to-do |
| [br-cfb2](http://dalek.tailbc91f5.ts.net:7878/task?id=br-cfb2) | Review ticket v8uu: seeing what background agents do (findings + 5 possible features) | claimed | the human's to-do |

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
