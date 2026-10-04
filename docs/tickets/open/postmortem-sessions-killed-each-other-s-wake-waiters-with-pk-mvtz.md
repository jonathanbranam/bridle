---
id: mvtz
title: "Postmortem: sessions killed each other's wake waiters with pkill -f (h3ar)"
kind: research
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [h3ar, fx7x, m7mp, 98xt, j28f, kuw2, cr7t, 75h2]
tasks: []
---

The kind is `research` until a `postmortem` kind exists
([[add-a-postmortem-ticket-kind-the-full-write-up-after-an-inci-cr7t|cr7t]]); then it becomes
`postmortem`.

## The ask

The human, verbatim (2026-10-04 ~7:15 PM ET, to the advisor):

> Let's introduce a new ticket type called "postmortem" or something. Actually, making this a ticket
> might be kind of weird, but let's go with it for now.
>
> I'd like to record some of the learnings that we run into, particularly this PKILL issue that we're
> hitting where the agents were shutting down their waiters. I'd like to have a full write-up of this
> that we can refer back to about exactly what happened, what the permissions were that allowed this,
> and the rules that allowed this.
>
> It seems a little bit strange to me that we are using something like `kill`. I want to understand
> more about that. Why not just block that from the agents altogether? Is that really an essential
> tool that these agents need? Is there a safer way to kill processes? I guess these processes are
> just spawned separately, and they're not owned by the agent process ID, so I guess that's maybe
> why we're doing this. There's no PID reported. It seems like we should never be doing things like
> this. We should just be killing by process ID directly, and we shouldn't have to search for it.
>
> I think there are a couple of solutions here and approaches that could help, but I still want a
> postmortem, no matter what, written up about this. [...]

The fix the human asked for in the same message is
[[a-waiter-is-stopped-through-bridle-never-by-a-kill-a-new-wai-75h2|75h2]].

## Summary

On 2026-10-04, interactive sessions (aides and advisors, across the bridle, track-web and bridle-ui
projects) killed each other's `bridle agent wake` waiters at least 13 times between 16:53Z and
22:45Z. Each session meant to stop only its own waiter. It ran `pkill -f "bridle agent wake
external:aide"` (or `external:advisor`), and that pattern matches every aide's or advisor's waiter on
the machine. A killed waiter ends with `exited with code 144` and no output, so the victim stops
listening for messages until it notices and starts a new one.

This was the **second** pattern-kill incident in five days. On 2026-09-29 a worker's `pkill -f "just
check"` killed the orchestrator twice (fx7x). That incident produced a rule against killing by name.
The rule wasn't enforced, didn't list the aide, and didn't reach interactive sessions at all.

## Impact

- Every victim stopped hearing messages and task changes until it noticed and started a new waiter.
  The aides and advisors noticed, from the "failed" task notification. No lost messages were found.
  The daemon marks a message read when it answers a waiter, and a waiter killed mid-wait gets no
  answer.
- A few hours of agent time and human attention went into explaining exit 144. The aide collected
  evidence from Claude Code's task output files and transcripts, and the human called it an incident.
- Trust: a second process-kill incident after the rule written for the first one.

## Timeline (2026-10-04; UTC, with ET in brackets)

The complete list of kills is in
[[wake-waiters-in-interactive-sessions-die-with-exit-144-in-pa-h3ar|h3ar]] ("Cause found").

- 16:53:23Z (12:53 ET): the track-web advisor runs `pkill -f "bridle agent wake external:advisor"`.
  It kills two bridle advisors (main and doc-review).
- 18:23:38Z (14:23 ET): the bridle main advisor (this ticket's author) started a waiter wrongly: with
  `&` and its output sent to `/dev/null`, outside Claude Code's task tracking. That waiter would have
  marked messages read and thrown them away. To stop it, the advisor ran `pkill -f "bridle agent wake
  external:advisor --timeout 5400"`, which also killed the track-web advisor's waiter. The advisor
  told the human the pattern might have hit track-web, and went on.
- 21:28Z, 22:20Z, 22:37Z, 22:45Z (17:28 to 18:45 ET): aides run `pkill -f "bridle agent wake
  external:aide"`. Each kills the other projects' aides, and the last also kills two of bridle-ui's
  own waiters.
- ~22:30Z (~6:30 PM ET): the human, via the aide: "this constitutes an incident". h3ar is filed.
- 22:46Z: the aide matches each kill to a `pkill` in the transcripts, to the second. Cause found.
- 22:47Z to 22:48Z: the aide and advisor prompts are told to replace a waiter with `TaskStop`
  (c28b48cc). The orchestrator tells every interactive session to stop using `pkill -f`. The
  rule's missing `aide` role is noted on h3ar.

## How it happened

1. **A session wanted to replace a waiter it had started.** In every case that was asked about, it
   had started the waiter wrongly: backgrounded with `&`, or with its output discarded. That put the
   waiter outside Claude Code's background-task tracking, so `TaskStop` couldn't reach it. A waiter
   whose output is thrown away is dangerous, because `bridle agent wake` marks the messages it
   delivers read. So the session had a good reason to stop it.
2. **It had no handle on the process.** `bridle agent wake` prints no pid, the session hadn't kept
   `$!`, and the daemon has no notion of "this session's waiter" that bridle could end. The only way
   left to find the process was to search for it by name.
3. **The name wasn't unique.** Every project's aide is `external:aide`. `external:advisor` is a
   prefix of `external:advisor/doc-review` and the other named advisors. And `pkill -f` matches
   anywhere in any process's full command line, so the search caught other sessions' waiters on
   other daemons.
4. **Nothing stopped the command.** See the next two sections.

## The permissions that allowed it

- **Interactive sessions** (orchestrator, advisors, aides started by `bridle session`) run in the
  human's own Claude Code permission mode (auto mode, for this advisor's session). The settings `bridle session`
  passes deny a list of tools (`LEAN` in `crates/bridle/src/session.rs`), but no shell commands. So
  `pkill` was allowed: auto mode's classifier let a session's "stop my own process" through every
  time.
- **No hook checks shell commands.** The only `PreToolUse` hook is `bridle arch-guard`, which checks
  Edit and Write (`workflow/base/hooks/PreToolUse.json`).
- **Daemon-spawned agents differ by role.** Managers and the project manager run under `dontAsk`
  with a list of allowed commands that has no `kill`. They can't kill anything: manager-2 couldn't
  stop a duplicate spawn earlier the same day. Bridle's own workers are allowed plain `Bash`
  (`.bridle/config.toml`, the worker's `allowed_tools`), so any command, `pkill` included. That's
  how fx7x happened.

## The rules that allowed it

- **`no-kill-by-name`** (`workflow/base/rules/no-kill-by-name.md`, severity `must`, written by
  br-15a7 after fx7x). It is exactly right ("Never use `pkill -f` ... Kill only a pid you started and
  own"), but:
  - It says itself "Not mechanically enforced yet". Five days later it was broken again.
  - Its `roles:` list omits `aide`, so even a correct rule delivery leaves the aides out. It's still
    missing as this is written.
  - **Interactive roles got no rules at startup at all.** `bridle prime <role>` printed none, a gap
    already recorded in 98xt. m7mp (rules for interactive roles) was integrated on 2026-10-04, but
    sessions started before their binary had it never saw a rule. The advisor's `bridle prime
    advisor` output that morning had no rules section.
- **The role prompts said how to start a waiter, not how to stop one.** The advisor and aide prompts
  said to wait with one background command, with no shell loop. They didn't say not to use `&`, not
  to discard its output, or how to replace a waiter. That was added after the cause was found
  (c28b48cc: `TaskStop`, or the pid you started).
- **fx7x's fix treated the victim, not the weapon.** After fx7x the orchestrator's opening prompt was
  shortened, so a pattern was less likely to match it (the comment above `ORCHESTRATOR_PROMPT` in
  `session.rs`), and a rule was written. Nobody's ability to kill by pattern was removed.

## The human's questions

**Why do agents use `kill` at all; is it essential?** Interactive sessions don't need it. Claude Code
gives every background command a task id, and `TaskStop` ends that task. That is the owned,
by-handle way, and needs no pid. Sessions reached for `kill` only after they had lost the handle by
starting the waiter outside Claude Code's tracking. Workers sometimes need to stop a process they
started, such as a dev server or a hung test run. `kill $!`, `kill %1` and `pkill -P $$` do that
safely, because they name a process the worker owns. Managers and the PM already can't kill. So no
role needs to kill by pattern, and interactive sessions don't need `kill` at all.

**Why not block it?** Pattern kills (`pkill`, `killall`, `pgrep` piped to `kill`) can be denied for
every role at once: in the session settings' deny list and the daemon roles' `disallowed_tools`,
backed by a `PreToolUse` check for the piped forms. Blocking plain `kill <pid>` costs workers a real
tool and buys little once the pattern forms are gone. 75h2 proposes the deny.

**Is there a safer way, without searching?** Yes, and it shouldn't need a pid either. The daemon
already holds every waiter's open request. So bridle can end a waiter itself: when the same session
starts a new wait, or on an explicit `bridle agent wake --stop`. That leaves the processes and
signals out of it. 75h2 has the design, the pid-printing idea included.

## What went well

- The aide treated the deaths as worth explaining instead of just re-arming. It found the cause from
  the transcripts, to the second, within about 15 minutes of the human calling it an incident.
- No lost messages were found.
- The prompt fix and a machine-wide warning went out within minutes of the cause.

## Learnings

1. **A `must` rule that isn't enforced is a hope.** Both incidents broke a rule, or a convention
   everyone knew. Where a command is never right, deny it mechanically for every role.
2. **Fix the capability, not the victim.** Making the orchestrator's argv harder to match (fx7x)
   left every other process exposed.
3. **Rules must reach every role that runs commands.** That includes the interactive sessions, which
   run in the human's own permission mode and so have the most power. Check a rule's `roles:` list
   against every role that could break it.
4. **Give agents a handle for everything they start, and a bridle command to stop it.** An agent
   that has lost its handle will search, and searching by name hits other agents' processes.
5. **Names aren't unique across projects or sessions.** Never select processes by identity
   strings. This also bears on the identity discussions (j28f, kuw2).
6. **An unexplained exit is a symptom.** The exit-144s should have been treated as an incident at the
   first pair at 12:53 ET, not after ten.

## Actions

| Action | Where | State |
|---|---|---|
| Aide and advisor prompts: replace a waiter with `TaskStop`, never `pkill -f` | c28b48cc | done |
| Interactive roles get their rules at startup | m7mp | integrated |
| Add `aide` to `no-kill-by-name`'s roles | h3ar fix 1 | open |
| A new wait replaces the old; `bridle agent wake --stop`; pattern kills denied for every role | 75h2 | filed |
| Warn every interactive session that has no waiter, not just the orchestrator | h3ar fix 4 | open |
| A `postmortem` ticket kind | cr7t | filed |
