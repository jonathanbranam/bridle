---
id: br-kt25
title: Move a project between machines with one command (bridle project move)
kind: feature
created: 2026-10-09
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-24mj, br-kuw2, br-hw6c, br-rjd5, br-sk7p, br-n63z, br-3haz]
parent: br-kx5u
theme: multi-machine
resolved: 2026-10-17T02:08:19Z
---

## The ask

The human, 2026-10-09 ~10:15 AM ET, verbatim (to advisor product-manager): "Put tickets in there that would enable basically efficient and pretty direct setup for a new machine. ... I want that as a priority so that I can use it to set up the new Windows machine and add it to the network. So it would include everything for configuring background daemons, launching tokens between machines, tokens between projects, and then as a as a additional feature I would like to see if we can get it in is automated project transfer between machines."

Workstream: machine setup, last phase ("an additional feature"). Today moving a project is the 7-step manual runbook in docs/context/nuc-host.md; the inbox, usage history and session transcripts stay behind, and services, mail, gateway and tokens are redone by hand. Prior design: 24mj Part 2 (resolved ticket, unbuilt) and kuw2 item 5 (machine daemon, waiting on the human's review, br-efs2).

The ask: `bridle project move <P> --to <machine>`: release on the old machine (owner.toml released + SHAs), stop and uninstall its service, take over on the new one (clone if needed, verify SHAs, install the unit, retarget mail and gateway), update `[projects]` everywhere, re-mint tokens, and carry or forward the unread inbox.

Needs a design (the designer role) and the human's decisions first:
- Does the old machine drive the new one over SSH, or only authorise it?
- What happens to uncommitted worktree work and agent sessions that can't resume?
- A CLI over SSH first, or wait for the machine daemon (kuw2)?
Builds on sk7p/n63z (tokens), rjd5 (sync), 3haz slice 2 (br-n7cg, mail follows the move).

## Now: a design pass that splits the move into build tickets

The human, 2026-10-12 ~12:00 AM ET (to advisor product-manager): "This is actually a massive
ticket that is likely an epic itself. I asked for a rewview and design I see related tickets but
not sure where the design it."

PdM decision: kt25 is the design for the rest of the move, not one build. The epic stays kx5u
(its done-when is this move); the design ends in build tickets with `parent: br-kx5u`, each
small enough for one worker. The designer (Sonnet) writes `## Design` in this file:

1. **Inputs.** The human's decisions on this thread (2026-10-11: the DB is NOT copied; list what
   is lost and ask; move scheduled messages; recreate tokens and config). The 3pc2 inventory
   (`inventory-everything-a-project-has-on-a-machine-and-what-mov-3pc2.md`), Sections 3, 4 and 5.
   What ga6s, n4ug and 86qp (53gy A, B, C) already build.
2. **A coverage table**: every item in 3pc2 Sections 4 and 5, one row each: covered by
   ga6s/n4ug/86qp, a new child here, named in the pre-move listing only, moot (because the DB
   is not copied), or for jw75 (moving bridle itself: 3pc2 Section 2). Nothing silently dropped.
3. **Answers proposed** for the three open questions above (SSH-driven or authorise-only; dirty
   worktrees and sessions; CLI over SSH now or wait for kuw2), each with a recommendation. The
   human's rule: one command, nothing by hand, nothing left behind.
4. **The children**, in build order, each with its ask, acceptance and size. Highest risks from
   3pc2 Section 5 first (committed absolute `system_prompt` paths; unpushed branches and dirty
   worktrees; the launchd plist left loaded on OLD).

Then the human reviews the design; only after that are the children readied.

## Design

By wkt25 (designer, Sonnet), 2026-10-12. Design only; nothing is built or filed. The children
below are text until the human reviews this; the PdM then files them with `parent: br-kx5u`.

### 1. Inputs

- **Human decisions, this thread.** The database is NOT copied (2026-10-11): list what stays,
  move what can move (scheduled messages), recreate what can't (tokens, config). Drive the move
  from the owning machine (OLD); if OLD is dead, take over from the receiving machine (NEW)
  with what is on GitHub. Reuse the existing sync, `token pair` and 53gy commands; keep each
  child small. The move is a supervised drain, not a cut: announce to every principal, wait for
  work to land or park, tell the human. (2026-10-12)
- **What 53gy built, on main.** `project stop` (n4ug): lists what it leaves behind
  (undelivered outbox mail, unread messages, schedules, unpushed branches, dirty worktrees) and
  asks; stops agents and the daemon; pushes `bridle/state` and the integration branch and
  confirms origin has both; commits `schedules.json`; pushes a `released` mark in `owner.toml`;
  unloads and disables the daemon and mail units; refuses a newer-schema DB; `--machine M` runs
  over ssh. `project take-over` (ga6s): refuses without the mark (`--force` for a dead machine),
  commits `owner.toml` with `[machine] name`, moves the `[projects]` entry, runs
  `projects sync --apply`. `project start` (86qp): installs the unit (systemd `--project`),
  enables and starts it, re-creates the project's tokens on every machine with `token pair`'s
  code (eurm: visitors with homes), re-adds the schedules; `--machine M` over ssh.
- **So the move is already these three in order.** What is missing is what surrounds them:
  checks before, parking and draining, after-care, one command with one confirm, and the dead-OLD
  path.

### 2. Answers to the three open questions (recommendations)

1. **SSH-driven or authorise-only?** Driven, from OLD, over ssh: `project move P --to NEW` runs
   stop locally and take-over and start on NEW through the same ssh helper `--machine` already
   uses. Authorise-only adds a second mechanism for nothing; the human already said "drive from
   the owning machine". The recovery path (OLD dead) runs on NEW with no ssh to OLD (child K8).
2. **Dirty worktrees and sessions.** Dirty agent worktrees and unpushed `bridle/*` branches are
   parked by the move: a WIP commit on the agent's branch, pushed as `bridle/<name>`, shown in
   the listing before the confirm (K3). A dirty main checkout is refused until committed (or
   named LEFT with `--leave`), because the move cannot guess whose edits they are. Agents are
   not resumed on NEW (transcripts stay on OLD, 3pc2 F1): the drain asks each to finish or hand
   over, releases claims, and a fresh agent starts from the ticket on NEW. Interactive sessions
   (orchestrator, aide, advisors, PdM) are asked for a handover note (`session restart
   --handover`, notes travel on the state branch); the move then tells the human which to start
   on NEW rather than starting them (supervision is local, 3pc2 Section 2.2).
3. **CLI over SSH now, or wait for the machine daemon (kuw2)?** Now. Stop, take-over and start
   already work over ssh; kuw2 is unreviewed (efs2) and would only replace the transport later.
   Nothing here depends on it.

### 3. Coverage table (3pc2 Section 4 and 5, one row per item)

Key: **built** = ga6s/n4ug/86qp/eurm/2wnn on main; **K#** = a child below; **listed** = named in
the pre-move listing only, with the impact (no mechanism); **moot** = the DB is not copied;
**jw75** = moving bridle itself (br-jw75), not this design.

Section 4, "53gy as written still misses":

| # | Item | Disposition |
|---|------|-------------|
| 4.1 | Stop confirms both pushes, fails instead of warning | built (n4ug) |
| 4.2 | Pre-stop listing: dirty worktrees, unpushed branches, outbox, schedules | built (n4ug) |
| 4.3 | Pre-stop listing: claims, running agents, sessions, in-flight land or upgrade | K4 (the drain waits on and lists them) |
| 4.4 | Unload or disable the daemon and mail units on OLD (launchd exit-78 loop) | built (n4ug); K5 adds the tools-only entry so OLD cannot serve P |
| 4.5 | One host identity (`owner.toml` vs `[machine] name`) | built (ga6s) |
| 4.6 | Take-over checks/sets `[projects]` so the daemon binds on NEW | built (ga6s moves the entry and syncs) |
| 4.7 | Linux start with `--project` | built (86qp) |
| 4.8 | Refuse a newer-schema DB | built (n4ug) |

Section 4, "kt25 should add":

| # | Item | Disposition |
|---|------|-------------|
| 4.9 | DB carry procedure and fix-ups (copy `tokens/`, `agents/`, `mail/`, `inbox/`; mark agents not resumable; clear `ports`, `meta.upgrade.built`, `meta.restart.resume`; release claims) | moot (nothing copied; NEW starts clean; claims are rebuilt from the state branch, and K4 releases them before the stop) |
| 4.10 | Principal re-labelling (rotate bare roles, `<role>@OLD` visitors, re-key credentials, `peer:NEW`, `[human.NEW]`) | built (86qp re-runs `token pair` on every machine; eurm's homes); a fresh DB makes the rotation moot. The orchestrator's home after bridle moves: jw75 |
| 4.11 | Committed absolute paths in project configs (`workflow`, `system_prompt`) checked or refused | K1 |
| 4.12 | Machine files: `mail.key`, `[mail]` | K6 |
| 4.13 | Machine files: `[gateway] public_url`, `[[focus]]`, `[budget]` | listed (per machine, not per project; set up once per machine by kx5u item 1) |
| 4.14 | Machine files: `pdm-roadmap.py`, `.claude/settings.local.json` | jw75 (both are bridle's; a bridle-ui or smaller project has neither) |
| 4.15 | Uncommitted `review-documents.txt`, `migrations.toml`, `migrations.log` | K3 (refuse until committed, or `--leave`) |
| 4.16 | NEW machine prerequisites (H): binary version, `claude` login, `gh auth`, git push, toolchain, tmux, Tailscale name, hostname, timezone, AWS for mail, WSL systemd and linger | K2 |
| 4.17 | OLD afterwards: `tools_only` entry, registry entry, units, upgrade worktree pruned | K5 (units: built, n4ug) |
| 4.18 | bridle and bridle-ui specifics (orchestrator home, workflow source, binary upgrades, `~/.bridle/ui`) | jw75 |
| 4.19 | Interactive sessions must move with P | K4 asks for handovers; the move tells the human what to start on NEW (answer 2) |
| 4.20 | Lists that don't move: transcripts, prompt log, daemon/governor/mail logs, `last-full-test-count`, focus state | listed (K7's listing text; all history, none needed to run) |
| 4.21 | Rollback path | K7 (abort before take-over restarts OLD); K9 (return trip) |

Section 5, "no proposal or command today":

| # | Item | Disposition |
|---|------|-------------|
| 5.1 | `bridle.db`, inboxes, outbox, schedules, conflicts, agent history | moot, except: schedules built (86qp); undelivered outbox and unread inbox counted and the human chooses stop/continue (built, n4ug); events, usage, agents, conflicts listed |
| 5.2 | Principal meaning after a move | built (see 4.10) |
| 5.3 | Unpushed worker branches; built tickets whose branch is only on OLD | K3 |
| 5.4 | Uncommitted worktrees, dirty main checkout | K3 |
| 5.5 | `mail.key`, mail sent-state and attachments | K6 (key and sent-state: open mailed questions are listed as lost if the key is not carried); attachments listed |
| 5.6 | Committed absolute paths, silent preamble-only fallback | K1 |
| 5.7 | Schema skew with no refusal | built (n4ug) |
| 5.8 | `meta.upgrade.built` carried to another binary | moot |
| 5.9 | `owner.toml` hostname vs `[machine] name` on WSL2 | built (ga6s) |
| 5.10 | launchd exit-78 loop on OLD | built (n4ug) |
| 5.11 | `systemd install` without `--project` | built (86qp) |
| 5.12 | Orchestrator home and supervision on OLD after bridle moves; local-only session supervision | jw75 |
| 5.13 | dalek's binary upgrades and `~/.bridle/ui` after bridle / bridle-ui move | jw75 |
| 5.14 | dalek's projects' workflow source after bridle moves | jw75 (K1 refuses the path in the project being moved) |
| 5.15 | `settings.local.json`, `review-documents.txt`, `migrations.toml`, `pdm-roadmap.py` | see 4.14 and 4.15 |
| 5.16 | Claude transcripts for resume | listed: "agents will not resume" is in the confirm text (answer 2) |

Section 5, highest risks:

| # | Risk | Disposition |
|---|------|-------------|
| R1 | Absolute `system_prompt` paths | K1 (first) |
| R2 | DB carried to an older binary | built (n4ug refuses); moot since not copied; K2 checks NEW's binary is not older anyway |
| R3 | Principals carried unchanged | moot (fresh DB) and built (86qp) |
| R4 | Unpushed branches and dirty worktrees lost | K3 |
| R5 | Integration commits in the last 30 s not pushed | built (n4ug pushes and confirms both branches) |
| R6 | Moving bridle leaves dalek unsupervised, un-upgraded | jw75 |
| R7 | launchd plist left loaded on OLD | built (n4ug) |

Nothing is dropped. Rows marked "listed" have no mechanism by design: the cost of building one
exceeds what is lost (history only).

### 4. Children, in build order

All have `parent: br-kx5u`. Sizes: S under half a day, M about a day. Each states its migration
(xebc); unless noted, none: they add commands and listing text, and change no project file or
config.

**K1. Refuse a move while committed config has absolute paths (S).** Addresses R1/5.6. Ask: a
check, `bridle project preflight <P> [--to M]` (read-only, started here), whose first item scans
P's committed `.bridle/config.toml` (and `.bridle/roles`, rules) for absolute or `~` values in
`workflow` and `system_prompt` and refuses with file and line; `bridle doctor` runs the same scan.
Acceptance: a fixture config with `workflow = "/Volumes/..."` is refused naming the key; a
relative or unset one passes; doctor warns. Migration: no automatic rewrite (where the workflow
should come from is a decision: release mode or a checkout, see jw75); bridle-ui, track-web and
data-contracts must be fixed by hand before they move, and the refusal says so.

**K2. Preflight: NEW is ready (M).** Ask: `project preflight` also checks NEW over ssh (H):
`bridle` binary at least OLD's version, `claude` logged in, `gh auth`, git push access to P's
origin, P's toolchain command exists, tmux, `[machine] name` equals the hostname-or-name
`owner.toml` will record, Tailscale name resolves, timezone, AWS credentials when `[mail]` names P,
and on Linux: WSL systemd and linger. Also runs the 2wnn comparison (`projects sync` without
`--apply`) and the clone check (present, or cloned with a confirm). Acceptance: each failing item
prints what to run on NEW; all-green prints nothing alarming; unreachable NEW fails with the ssh
error. Reuses `bridle doctor` checks where they exist.

**K3. Park work before the stop (M).** Addresses R4/5.3/5.4/4.15. Ask: `project stop` gains a
parking step after its listing and confirm: push every local `bridle/*` and `integrate/*` branch
ahead of origin as is; WIP-commit each dirty agent worktree onto its branch and push it
(`WIP: parked by project stop`); verify with `git ls-remote` that every `built` ticket's branch
and commit are on origin (else refuse); refuse a dirty main checkout, including untracked
`review-documents.txt` and `migrations.toml/.log`, unless the human commits it or passes
`--leave` (named LEFT in the listing). Acceptance: test repo with a dirty worktree, an unpushed
branch and a built ticket: after stop all three are on origin; a dirty main checkout blocks;
`--leave` lets it through and prints it.

**K4. The drain (M).** Ask: `project drain <P> [--timeout D]`, used by the move and usable alone.
(1) Announce: message every principal of P (PdM, orchestrator, aides, advisors, managers,
workers) with `bridle send`: P is moving to NEW; finish or hand over; interactive roles write a
handover note. (2) No new work: hold planned tickets (`ticket hold --reason moving`, undone by
abort or on NEW after start) and stop new spawns, as the restart/upgrade drain does. (3) Wait
until no agent is running, no claim is held except parked ones, no land or upgrade is in flight
(btdn's quiet-point logic). (4) Supervise: print progress every minute; a ticket that will not
land, an agent that will not finish, or a failed push is reported (to the orchestrator and to the
human through aide, by the existing message path) with its name, not left silent. (5) On timeout
or on `--now`, list what is still running and ask: wait longer, park and go, or abort.
Acceptance: with fake agents, the announce reaches all principals once; the drain returns when
the last agent stops; a stuck agent produces a stall report and the timeout prompt; abort
unholds. Migration: none (holds are ordinary ticket state).

**K5. OLD after-care (S).** Ask: after stop succeeds, add P's clone on OLD to `[machine]
tools_only` (so OLD can never serve it: `tools_only.rs`), remove the registry entry, prune the
upgrade worktree and `upgrade-*` tree (B15); leave `bridle.db` in place and say so (events and
usage can still be read there). Acceptance: on OLD, `bridle serve --project P` refuses;
`.bridle/bridle.db` still exists; the config edit is idempotent.

**K6. The mail bridge follows (S).** Ask: when `[mail] projects` names P: `project start` on NEW
installs the mail unit (`bridle mail install`), copies the `[mail]` section, and `mail.key`
(0600, over ssh) when open mailed questions exist, else says replies to mail sent from OLD will
fail verification; the bridge on OLD is already unloaded by stop. Fold in br-n7cg (3haz slice 2)
if it is still unbuilt. Acceptance: after a move, NEW's bridge answers; OLD's has no unit.

**K7. `bridle project move <P> --to <machine>` (M).** Ask: the one command, run on OLD.
(1) preflight (K1, K2); (2) one listing and one confirm: what is moved (scheduled messages,
branches, tickets, state), recreated (tokens, config, units) and left behind with its impact
(events, usage, agent turns and transcripts, logs, prompt log, mail attachments, undelivered
outbox: the human chooses stop-and-let-drain or continue); (3) drain (K4); (4) `project stop`
(with K3); (5) over ssh on NEW: `project take-over`, `project start`; (6) check: NEW's
`/health` answers, tokens work in both directions (`token pair` result), the outbox of other
projects reaches it; (7) K5 and K6; (8) tell the human through aide: started, the checks, the
list of interactive sessions to start on NEW. Failure before take-over: unhold and `project start`
on OLD, then report. Failure after: stop, say where it is, and print the one command to resume
(`project start` on NEW). Progress prints per step; `--yes` skips only the confirm, never a
refusal. Acceptance: an integration test with two fake machines (a local second HOME) runs the
whole move; a forced failure at each step leaves a documented state. Migration: none.

**K8. Recovery from NEW when OLD is dead (M).** Ask (PdM: the forced take-over is the mechanism,
not a second one): `project take-over --force`, run on NEW, first prints what may be lost, computed
from origin only: the last `bridle/state` commit time, the integration head, `built` tickets whose
branch is not on origin, claims held by agents that no longer exist (released), `owner.toml`
last host; says "anything not on GitHub is gone: messages, schedules since `schedules.json`
(old if any), events, usage"; asks; then the existing take-over and `project start`. When OLD
comes back, `project stop` or `serve` on it sees `owner.toml` names another machine and only
unloads its units and adds the tools-only entry (K5), never serving. Acceptance: with no
released mark, `--force` prints the list and asks; a returning OLD refuses to serve P.

**K9. Return trip guard (S).** Ask: a project that returns to a machine that held it before must
not restart its outbox ids at `o-0001` (receivers dedup on origin machine, daemon and id and
would silently drop mail, 3pc2 C7). Recommendation: `project stop` records the outbox high-water
mark in the `released` commit; start seeds a fresh DB's counter above it; a left-behind
`bridle.db` for P on that machine is moved to `bridle.db.moved-<date>`. Acceptance: move
P A -> B -> A; mail sent from A after the return is accepted by a receiver that saw A's earlier
mail. Last because it only bites on the second arrival.

Not built here: moving bridle itself (br-jw75: orchestrator home, workflow source, upgrades,
`~/.bridle/ui`, `pdm-roadmap.py`), cross-machine event queries, kuw2.

### 5. Questions for the human

1. **Return trips (K9).** Recommended: move the old DB aside and seed the outbox above its
   high-water mark, rather than reuse a stale DB. OK?
2. **Parking (K3).** Is an automatic WIP commit pushed to `bridle/<name>` for dirty agent
   worktrees acceptable, with the main checkout refused rather than committed? (Recommended.)
3. **Drain timeout (K4).** Default wait before asking: recommended 2 hours, then ask to wait
   longer, park and go, or abort. The human said waiting is fine; is a default needed?
4. **Interactive sessions on NEW.** The move tells you which to start; it does not start them
   (recommended, local supervision). Or should it start the orchestrator and aide on NEW?
5. **Order.** K1 and K2 can land first and are useful alone (`project preflight` before any
   move). K7 comes after K3 to K6; K8 can go in parallel with K7. Agree?
