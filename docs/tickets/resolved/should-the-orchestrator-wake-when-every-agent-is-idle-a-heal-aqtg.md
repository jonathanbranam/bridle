---
id: aqtg
title: Should the orchestrator wake when every agent is idle? A health check needn't wake a model
kind: question
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [pdmd, f5ww, 789x, r9vh]
tasks: [br-fef3]
closed: 2026-10-09T23:11:02Z
---

## The ask


The human, verbatim (2026-10-03, via the advisor), after the NUC's meta-notes orchestrator
reported "The latest wake was the routine notice that every agent has been idle for 15 minutes.
That's expected: the queue is empty and the manager has nothing to do, so nothing needs action.
The watcher is running again.":

> That message is actually from the Meta Notes orchestrator, I guess. Um, I thought the 90-minute
> wait was working, regardless of the agents. But maybe the orchestrator had shut the agents down
> at some point, so they weren't getting that notification. I don't know. Um, it seems like
> overnight the 90 minutes was working. Uh, but this is another quite big question mark to me. Do
> we need to wake an agent? Do we need to wake the orchestrator because the agents aren't
> working? If there's nothing in the queue and there's nothing to be done, you know, it's like a
> system health check. But why wake up an LLM for a system health check? I'm not sure. I mean, I
> think the answer might be because we're worried we can't find all the deterministic reasons why
> the system might be down, but an agent could. I don't even know if that's true or not. Anyway,
> um, I'm, I'm definitely considering removing that rule from the wake for the orchestrator. I
> just don't see any point in waking these things up all the time. If you, can you, is it
> possible to get a estimate of the token use here? I, it's probably pretty darn small, but it
> just seems like a silly thing to do to wake an agent up when there's just nothing going on at
> all. And it makes me question the scale of what we're working on and in a sense, I mean, right
> now we have one orchestrator per machine. So that's only two, but I don't know. I appreciate
> your thoughts, and uh, maybe we could just write up a ticket about this to track the, like, as
> an open question.

## How `all_idle` works (advisor, checked 2026-10-03)

- `crates/bridle-daemon/src/wake.rs` (`check_idle`): when no agent is `working` or `starting`
  for 15 minutes, the orchestrator gets one `all_idle` wake. It fires **once per idle stretch**:
  it re-arms only after some agent works again. A project with no agents at all counts as idle.
- So a quiet night gives one wake, not one per poll: the NUC's 90-minute waits held overnight
  because nothing worked. The morning wake came 15 minutes after the manager (or another agent)
  last finished a turn.
- Its purpose (orchestrator role, "keep the workers busy"): notice when work stalls, e.g.
  startable tasks but an idle manager. pdmd already listed it as a candidate to drop or send to
  the manager first. f5ww (the daemon tells the manager when the queue changes) now covers one
  cause of stalls deterministically.

## What it costs (advisor, measured 2026-10-03)

From dalek's orchestrator transcripts (`~/.claude/projects/-Volumes-Data-work-bridle-bridle/`,
30 `all_idle` wakes, 2026-09-29 to 10-02, roughly 7 a day on this machine):

- **A typical "nothing to do" wake:** one model call: re-reading the context from the prompt
  cache (45K to 170K tokens, cheap at about a tenth of the input price), about 600 tokens written
  to the cache, and 30 to 150 output tokens. About 10K to 20K input-token-equivalents.
- **When the orchestrator looks around** (reads status, the queue, the manager's note): 3 to 6
  calls, 350K to 900K cache reads, 1K to 2K output. About 50K to 100K input-token-equivalents.
- **Total:** about 7.7M cache-read tokens over those 30 wakes, roughly 2M a day, or about 200K
  input-token-equivalents a day. Small, but it's pure overhead, it grows each orchestrator's
  context (a few hundred to a few thousand tokens per wake, bringing handovers closer), and it
  scales with orchestrators and projects.
- The wake also costs whatever the orchestrator does in response; some of the long ones may have
  started useful work. Not measured.

## Options (advisor)

- **A. Drop `all_idle` from the orchestrator's wakes.** Simplest. Risk: a stall nobody notices
  until the human asks.
- **B. Replace it with deterministic checks in the daemon (recommended):** wake someone only when
  idle *and* there's something to do: startable tasks in the queue with an idle manager (nudge the
  manager, not the orchestrator, as f5ww does), or a task stuck in a state too long. An empty
  queue with everyone idle wakes no one.
- **C. Keep it but rarer** (e.g. after an hour, or once a day as a health check).

Open: whether some failures can only be spotted by a model, and which; the human doubts it. Any
found should become deterministic checks (incidents) rather than a timed model wake.

## Decided: remove it (the human, 2026-10-03)

The human, verbatim (via the advisor):

> Yeah, my feeling is is that, well, first of all, it's 15 minutes after the last agent stops.
> That's kind of useless. So I think it should be removed. If it was 15 minutes always, then
> yeah, it's actually doing something, but that's super excessive. I guess it would catch a
> merge. Like if, if the manager ran into something and did merge or, I don't know, yeah, it
> just, it just seems kind of silly. But I see what you're saying, actually, it's really just a
> check after work quiets down that everything's right.
>
> When phrased that way, it doesn't concern me as much. OTOH The orchestrator can choose its own
> timeout. It doesn't have to set an hour and 55 minutes every time. So if there's a lot of work
> going on or something happening, the orchestrator could always set a shorter timeout and check
> on things. Um, yeah, I don't know. Write it up in the ticket. I'm gonna. I actually, yeah.
> Let's just let's just go ahead and remove it, and leave a decision up to the orchestrator. If it
> feels like a shorter timeout's needed for some particular case, that's still an option.

Decided:

1. **Remove the `all_idle` wake** (option A): the daemon no longer wakes the orchestrator when
   every agent has been idle 15 minutes. Remove it from `wake.rs`, the design doc
   (`orchestrator-supervision.md` section 5), `api.md`'s reason list and the orchestrator role's
   wake list.
2. **The orchestrator decides when to look.** With its own wait timeout (789x, up to 1 h 55 min),
   it picks a shorter one when a lot of work is running or something needs watching, as a
   check after work quiets down. Say so in the orchestrator role.
3. Option B's deterministic checks aren't asked for now; f5ww already nudges the manager on
   queue changes.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). The human decided to remove the all_idle wake; br-6c6a removed it.
