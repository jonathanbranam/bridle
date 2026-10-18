---
id: br-gb97
title: A machine admin role takes the orchestrator's admin work; orchestrator and admin are per machine
kind: feature
created: 2026-10-11
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kuw2, br-ma8e, br-j2vq, br-3827]
parent: br-j6d3
theme: agents-and-cli
---

## The ask

The human, 2026-10-11 ~1:20 PM ET (to advisor product-manager), verbatim: "we need a theme/epic for security + hardening; future work in that realm is: * agents need limited access to SSH * I think switch to PAT for github to reduce permissions as well * split admin responsibilities that orch has to a separate machine admin role (related: orch + admin become per-machine) * related is identifying agents that make changes" (the third item).

## What's wanted

A machine admin role takes the machine-wide admin work the orchestrator does today, so the
orchestrator only steers the work and holds fewer permissions. The orchestrator and the admin are
each one per machine.

A design pass first: inventory what the orchestrator does that is admin (daemon restarts and
upgrades, installs, signing, launching sessions and windows, disk and process checks, tokens,
scheduled maintenance) versus steering (queue, workers, landings); what permissions each role then
needs (see the SSH and PAT tickets under the same epic, and j2vq on stop/rm); whether the admin is a
session or part of the machine daemon (kuw2). Related: ma8e, the epic one orchestrator per machine.
Then the human reviews.

## Design options

Focus: both. Interface (who types what, which commands move) and internal architecture (which
principal the daemon lets call what). Design only; nothing is built until the human approves.

Problem in a sentence: the orchestrator holds machine-wide admin rights it uses rarely, and the
human wants steering and admin split so each role holds only what it needs, one of each per machine.

### What exists today

- **Orchestrator** (`workflow/base/roles/orchestrator.md`): one per machine, in `<host>-home`,
  principal `external:orchestrator`, interactive tmux session kept alive by the daemon's supervisor
  (orchestrator-supervision.md). On other projects' daemons it holds a visiting token
  `orchestrator@<machine>` with full orchestrator rights (br-j86p).
- **What the orchestrator may do that is admin** (api.md, cli.md): `POST /v1/restart` /
  `bridle daemon restart [--upgrade]` (human and orchestrator only); `bridle daemon restart
  --cancel|--postpone` (also managers); `POST /v1/project/hold` / `bridle project drain` (human and
  orchestrator); `bridle token check` (read-only); the machine-load response (hold spawns, find the
  cause); the incident log. Wakes about restarts, upgrades and load land in its waiter.
- **What is already the human's alone**: `bridle daemon stop`, `token create|pair`, `agent stop|rm`
  (the orchestrator's auto mode refuses them, j2vq), `project setup|start|stop|take-over`, `bridle
  sign setup`, `bridle up`, machine config files (denied by hook). Installs and signing run inside
  `just install` and the daemon's self-upgrade, not an agent.
- **Self-upgrade is automatic** (`[daemon] self_upgrade`): most restarts need no agent at all.
  So the admin work an agent does today is small: an occasional manual restart or drain, load
  response, and chasing a failed upgrade or firewall incident.
- Role names are reserved and only the human, the daemon and the orchestrator may spawn an agent
  under one (principals.md, 2vja).
- No near-duplicate: there is no `admin` role, `bridle admin` command or admin principal today.

### What is admin and what is steering

| Admin (machine-wide, rare, high privilege) | Steering (per project, constant) |
|---|---|
| restart and upgrade the machine's daemons, drain and hold a project | queue, ready tickets, messages to manager/PM |
| respond to machine load, disk, stale binary, firewall incidents | verify merges, bring the human decisions |
| token check and pair, project start/stop/setup (human's today) | the incident log, handover, watching wakes |
| daemon units, launchd/systemd, signing, the reboot test | acting PM on a small project |

The honest finding: today's orchestrator admin rights are few (restart, drain/hold). The big
admin verbs are already the human's. So the split buys two things: narrower orchestrator
permissions (the security epic's aim) and a place to put the human's admin verbs later, with
guard rails, if the human wants an agent to run them.

### Options

**A. A standing admin session, a twin of the orchestrator.** `external:admin`, `bridle session
admin`, a `[admin]` entry per project in `credentials.toml` from `token pair`, a pane tag, a
supervisor like the orchestrator's, wakes forwarded to `<host>-home`. The user types
`bridle send external:admin "upgrade every daemon on this machine"`.
Cost: a second always-on session (context and tokens at idle), a second supervisor and handover
path (duplicates orchestrator-supervision.md), a new credential role in `token pair`. Falls short:
KISS/YAGNI; modularity is good but the work is rare, so a standing session mostly idles.

**B. An on-demand admin agent, spawned for a job.** A role `admin` (`workflow/base/roles/admin.md`,
`[roles.admin]` defaults: Bash limited to `bridle daemon *`, `bridle project *`, `bridle token
check`, `bridle status`, read-only otherwise; no Edit/Write outside tickets). The orchestrator (or
the human) spawns it in `<host>-home` when there is admin work: `bridle agent spawn admin` then
`bridle send admin "upgrade the daemons, then check each answers"`. It finishes, reports to the
sender and exits. One per machine because it lives in the home project and the spawn refuses a
second (as for reserved names). Name: `admin`, reserved by the new role; seen from other machines
as `admin@<machine>`. Inside: `POST /v1/restart` and `/v1/project/hold` accept `agent:admin` and the
human, and stop accepting `external:orchestrator` (after a transition, below). To act on the
machine's other project daemons it needs a visiting token (`admin@<machine>`, as `orchestrator@`
today); `token pair` mints it by the same code, one more role in its list.
Cost: one role file, one rights change on two endpoints, one more visiting role in pair. No new
commands, no new session. Falls short: the orchestrator now asks for what it used to type, one
extra step on a manual restart (rare); an agent costs a spawn each time.

**C. The machine daemon does it (kuw2).** A deterministic `bridle machine upgrade|drain` run by the
human, with no agent. Right long term for the mechanical sequence (drain all, upgrade, resume),
and the human wants it ("one place I can say shut everything down"). But kuw2 is a large open
design; waiting on it blocks the permission split, and judgement work (why did the upgrade fail,
is the load a bridle process) still needs an agent. Not rejected: B's admin agent is its natural
caller when it exists. Out of scope here.

**D. Do nothing but narrow.** Drop `external:orchestrator` from `/v1/restart` and `/v1/project/hold`
and let the human do both. Smallest change, meets the security aim, no new role. Falls short: the
human becomes the admin for upgrades, drains and load holds, which the orchestrator handles for them
today; and the orchestrator's own role text says to ask the human for a restart anyway on budget
holds, so little changes. This is the honest baseline: if the human mostly wants narrower rights,
D is enough and B is gold-plating.

### Measured against the principles

- **KISS/YAGNI:** D is smallest; B adds one role and one rights check for a real need (narrow the
  orchestrator, keep admin work off the human); A fails (a standing session for rare work); C is
  a different project's size.
- **One name per action:** B adds no commands. The admin types the same `bridle daemon restart`;
  only who may call it changes. A `bridle admin ...` command family would duplicate them
  (rejected, fne2).
- **Modularity:** B keeps the boundary at the API's per-principal rights, where it is enforced
  today. A adds a second supervisor beside the orchestrator's. C couples it to the machine daemon.
- **The user's side first:** what the human sees stays: the orchestrator says "asking admin to
  upgrade" and the report comes back on the thread. Admin work shows in `bridle agents`.

### Recommendation

**B, phased, with D as its first step.**

1. Add the `admin` role and let `agent:admin` and `admin@<machine>` call `/v1/restart` and
   `/v1/project/hold` (additive: nothing breaks, old daemons ignore it).
2. Update the orchestrator's role text: ask admin for restarts, drains and load response; keep
   `--postpone`/`--cancel` (safety valves for a land) and `token check`.
3. After one release, stop accepting `external:orchestrator` on the two endpoints (the 403 change).
   This is the breaking step; it ships after the role text has been out, per the 3haz lesson
   (staged, with a fallback: the human always keeps the rights).
4. Later, as the human wants, move more verbs (project start/stop, token pair) to admin one at a
   time, each as its own ticket. Not now.

Trade-offs accepted: a spawn per admin job; the orchestrator is one hop from a restart; the admin
agent holds a broad visiting token (so it must have a narrow Bash allow list, and must be
named in provenance, the security epic's fourth item, 2bzw/xccp).

Migration for existing projects: none to edit. The role arrives with `workflow/base` and the
default `[roles.admin]`; a project's own `.bridle/roles/orchestrator.md` (bridle's mentions restart on
budget holds) needs its wording checked. Daemons older than step 1 keep working: the orchestrator
still has the rights there.

How to verify: a test per endpoint that the orchestrator token is 200 before step 3 and 403 after,
and the admin token 200 in both; a live run where the orchestrator asks admin to drain and restart
a non-bridle project's daemon, and the reboot test (mru4) on a second machine.

### Rejected (so the next reader doesn't re-propose them)

- A standing admin session (A), unless the admin work grows to need one.
- A `bridle admin <verb>` command family: duplicates existing commands.
- Moving `agent stop|rm`, `token create` or `daemon stop` to admin now: they are human-only on
  purpose (j2vq) and a separate decision.

### Questions for the human

1. What is the point: narrower orchestrator permissions (D is enough), or getting admin chores
   off the orchestrator's and your plate (B)?
2. Is an on-demand admin agent acceptable, or do you want a standing session like the orchestrator?
3. Should the orchestrator keep `--postpone`/`--cancel` for a restart (recommended), or lose them too?
4. Which of your hand verbs (project setup/start/stop, token pair, sign) should admin take over,
   and when? (Recommended: none until B is in use.)
5. Is C (a machine daemon, kuw2) the real answer and this role only a stopgap?
