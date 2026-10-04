---
id: 75h2
title: "A waiter is stopped through bridle, never by a kill: a new wait replaces the old, bridle agent wake --stop, and pattern kills are denied"
kind: feature
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [h3ar, mvtz, fx7x, m7mp, j28f, kuw2]
tasks: [br-75h2]
---

## The ask

The human, verbatim (2026-10-04 ~7:15 PM ET, to the advisor):

> It seems a little bit strange to me that we are using something like `kill`. I want to understand
> more about that. Why not just block that from the agents altogether? Is that really an essential
> tool that these agents need? Is there a safer way to kill processes? I guess these processes are
> just spawned separately, and they're not owned by the agent process ID, so I guess that's maybe
> why we're doing this. There's no PID reported. It seems like we should never be doing things like
> this. We should just be killing by process ID directly, and we shouldn't have to search for it.
>
> I think there are a couple of solutions here and approaches that could help [...]. Let's explore
> options to make it much easier for agents to kill their waiters. They should probably just be a
> command, right? Like, `bridle stop waiter`. I don't know. That seems like maybe a direct thing.
> Let `bridle` deal with it. If you issue the command the same way, then `bridle` knows how to do
> that and looks it up properly, and then refuses if it's not appropriate.
>
> My first suggestion was to have the `wait` command return its PID to standard out. That seems also
> like a pretty obvious solution to make this more robust. Do the postmortem, write it up in a pilot
> somewhere, and then also write up a design ticket, a feature ticket for a good solution for this.

What happened, and why `kill` was used at all: the postmortem
[[postmortem-sessions-killed-each-other-s-wake-waiters-with-pk-mvtz|mvtz]] and the incident
[[wake-waiters-in-interactive-sessions-die-with-exit-144-in-pa-h3ar|h3ar]]. In short: a session that
had lost the handle on its own waiter searched for it by name, and the name matched other sessions'
waiters on the machine. h3ar's "Fix" lists a takeover (fix 2). This ticket is the build for that and
for the rest of the human's ask.

## Today

- `bridle agent wake <identity>` (and `bridle orchestrator wait-for-wake`) is a long poll. The
  daemon's handler (`principal_wake` in `crates/bridle-daemon/src/server.rs`) waits until there is
  a reason to wake, or the timeout. It keeps only a count of open waiters (`Waiters` in `wake.rs`),
  not who they are. Two waits for one identity both run. Whether both get the same message is
  unverified.
- The CLI prints nothing until the poll returns: no pid, no waiter id.
- Delivering a message marks it read, so a waiter whose output is discarded loses messages. That is
  why a session that started one wrongly wants it gone.
- Nothing denies `pkill` or `killall` to interactive sessions, or to bridle's workers (plain `Bash`).

## Proposals

- **P1. A new wait replaces the old one.** When a session starts a wait, the daemon ends any open
  wait from **the same session**. The old CLI exits with a new code (5, "superseded by a newer
  wait") and doesn't mark anything read. Starting a fresh waiter is then always safe and always
  enough, so nobody needs to stop one first. Replacement is keyed to the **session, not the
  identity**, because identities are shared:
  - every project's aide is `external:aide`;
  - two unnamed advisors on one project are both `external:advisor` (that happened on 2026-10-04);
  - the owner may wait for a named advisor's identity.

  If it were keyed to the identity, two sessions with one identity would keep ending each other's
  waits. The session comes from the launcher. `bridle session` registers its pid and start time
  (`SessionRegister`) and exports it to the session's environment, and `bridle agent wake` sends it.
  A wait with no session (a bare shell) replaces nothing.
- **P2. `bridle agent wake --stop`**, the human's "`bridle stop waiter`". It ends this session's
  open wait through the daemon, with no signals or pids. With `<identity>` it ends that identity's
  waits instead. The same check as for waiting applies: you may stop only your own, and the human may
  stop any. It prints what it stopped, or "no wait open". Exit code 5 for the stopped waiter, as in
  P1. A subcommand form (`bridle waiter stop`) works as well; the name is the human's to pick (Q1).
- **P3. Deny pattern kills for every role.** Add `Bash(pkill *)`, `Bash(killall *)` to:
  - the session settings' deny list (`LEAN` in `crates/bridle/src/session.rs`);
  - every daemon role's built-in `disallowed_tools`.

  A `PreToolUse` Bash hook (`bridle kill-guard`, beside `arch-guard`) refuses the piped forms the
  deny rules can't match: `pgrep ... | xargs kill`, `kill $(pgrep ...)`, and `pkill` inside a
  compound command, if Claude Code's deny rules don't catch that (unverified). Its refusal points at
  `TaskStop`, `kill $!` and P2. `no-kill-by-name` then says what enforces it, and `roles:` gains
  `aide`, or becomes every role.
- **P4. Keep `kill <pid>`.** Workers sometimes need to stop a process they started (a dev server, a
  hung test run). `kill $!`, `kill %1` and `pkill -P $$` name a process they own, which is what the
  rule asks for. Interactive sessions have `TaskStop`, so they need no kill at all. Denying plain
  `kill` to them is possible but not proposed (Q2).
- **P5. The waiter states its pid on stderr** (the human's first suggestion):
  `waiting as external:advisor (pid 12345, timeout 5400 s)`. It's on stderr so `--json` output stays
  clean. It's cheap, and it gives a session that lost its task id an exact pid instead of a search.
  With P1 and P2 it should rarely be needed. It stays as a fallback, and as a record in the task's
  output of which process was waiting.
- **P6. Prompts and docs.** The waiting sections of the orchestrator, aide and advisor prompts say:
  - start a waiter only with Claude Code's background command: never `&`, never output discarded;
  - to replace one, just start a new one (P1); to stop one, run `bridle agent wake --stop`.

  Update `docs/design/cli.md` and `docs/design/agent-host/` (wake, exit codes).

## Order

P3 first. It's small, needs no daemon change, and stops a third occurrence on its own. Then P1
with P2 (one daemon change: track open waits by session), then P5 and P6.

## Decided (the human, 2026-10-04 ~7:40 PM ET, approving)

The human, verbatim, to the advisor:

> Yeah, for 75 Hotel 2, I think I agree with 1. I just don't see any reason for `kill` or `kill
> -all`, and I agree with 2. That seems good. We need to explain that behavior clearly in the rule,
> and I also think 3 is a great idea, something to add. I think the syntax there is fine, and I don't
> see a problem with 4 either. I think it's great for traceability.
>
> [...]
>
> Doesn't seem like we need to block plain kill for interactive sessions or anything like that.
>
> I approve that ticket with those changes.

The human's 1 to 4 are the advisor's summary of this ticket, in the Order above: 1 is P3, 2 is P1, 3 is
P2 and 4 is P5.

- **P3 approved:** deny `pkill` and `killall` to every role.
- **P1 approved, with a change:** the rule must explain the replacement clearly. `no-kill-by-name`
  (and the waiting sections in P6) say that a new wait from the same session replaces the old one,
  and that this is matched by session, not identity. They also say what the old waiter prints and
  its exit code, and how to stop a waiter without replacing it (P2).
- **P2 approved, as `bridle agent wake --stop`** (Q1).
- **P5 approved,** for traceability (Q3).
- **Q2: no.** Plain `kill <pid>` isn't blocked, for interactive sessions or anyone else.
