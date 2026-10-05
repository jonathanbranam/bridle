---
id: gq9r
title: Interactive sessions hand themselves over and restart at 200k instead of only being told to
kind: feature
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: [4s3z]
see: [jttf, e9yu]
tasks: []
---

## The ask

The human, verbatim (2026-10-04 ~10:40 PM ET, via the aide; saved over quiet hours):

> All right, I came here to tell you something, but I couldn't remember what it was. I think it's
> that the agent handovers are not happening properly. The agents cannot hand themselves over, and I
> don't know where that ticket went. That work doesn't seem that hard. I feel like it should have
> been done already. I don't know how it needs to be shipped out to everybody, but those handoffs
> need to be fixed. Agents also need to be able to restart themselves, and the way it's working now
> is just kind of bonkers. They're getting up to 250 tokens and not doing anything about it.
>
> Really, I think we need to change the phrasing of that budget. If the agents are getting higher
> and higher, like the aide and the advisors, they need more authority to do a handoff. They get
> woken up with messages, and things happen when they're not talking to me. They should just say,
> "Hey, I'm at 200. Let's restart." There's not a great reason to keep going after 200. I don't want
> to necessarily force it, but yeah, they're just getting way too big for no good reason.

("250 tokens" is 250k.)

## Where it stands (the aide, 2026-10-05)

- The context steps for interactive sessions are built
  ([[interactive-sessions-context-for-all-daemon-decided-wakes-re-jttf|jttf]], br-jttf integrated;
  `docs/design/agent-host/orchestrator-supervision.md`, "Context steps"): 150k warn, 200k "plan a
  handover unless the human overrides", 250k "the normal ceiling", 300k forced restart. The 200k and
  250k steps only *tell* the session; the restart waits on the human.
- **A session can't restart itself**: `bridle session restart` run from inside the session kills its
  own caller, and `--fresh` is refused inside a session.
  [[bridle-session-restart-says-it-restarted-a-session-that-s-st-4s3z|4s3z]] (br-4s3z, planned, not
  started) has the fix: the restart runs detached or in the daemon.
- Handover notes are now daemon records, one per identity and project (e9yu and task br-cyvf, landed
  2026-10-05).
- Observed 2026-10-04: an advisor at 150k+ and others past 200k carried on.

## What's wanted

1. **Sessions hand themselves over and restart** at 200k: on the 200k step the session says so
   ("I'm at 200k; restarting") and, unless the human is mid-conversation and says keep going, writes
   its handover and restarts itself. The human can still override (`session keep`) or choose a
   fresh restart. Not forced before the hard limit.
2. **The authority is in the prompts every interactive role gets** (aide, advisor; the
   orchestrator keeps its own numbers): the step messages and `workflow/base/roles/aide.md` and
   `advisor.md` say "restart yourself" rather than "plan a handover". This is "shipped out to
   everybody".
3. Needs 4s3z (a session can restart itself, and "restarted" is reported only once it's true).
