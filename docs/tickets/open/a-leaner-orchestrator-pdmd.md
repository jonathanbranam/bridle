---
id: pdmd
title: A leaner orchestrator: fewer wakes, less conversation, cheaper restarts
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [the-orchestrator-stays-running-fx7x, bridle-watches-ci-c8qw]
---

## The ask

The human, verbatim (2026-09-29, via the advisor):

> what is orchestrator's starting context? I think it's over 50k yeah? that's my concern; that
> 50k is not cached; so, we're spending 50k tokens for every restart; that is going to add up in
> cost very quickly; so - we either need to find a way to cut that down significantly, or delay
> restarts until orchestrotor hits > 200k.
>
> Also - I think orchestrator has too many responsibilities to juggle which is why the starting
> context prompt is so high and why we hit a ceiling so rapidly. We need to share the research
> and burden here with other agents. I want the advisor to handle more of the Q&A. And/or I think
> we should have a different agent handle the mechanical aspects of monitoring work. Another
> thing that eats context is the heartbeat - it wakes orch up and orch does some work every time;
> is there a way to offload or automate any of that work?

After the advisor's findings (below):

> Yes, let's get started on this work. I will direct more of my questions to the advisor which
> isn't interrupted or dealing with system events and wakes; also I think (2) will be a big
> benefit and keep orch leaner.

## Findings (advisor, from the orchestrator's transcripts)

Sessions read: today's 8d0a517c (1 h), d19c4df3 (47 min), b2005ca8 (running), and last night's
fc26fe40 (8 h), all in `~/.claude/projects/-Volumes-Data-work-bridle-bridle/`. Context is
input + cache writes + cache reads per API call.

**Starting context: 42–50K on the first call, 55–71K after the startup turn.**
- ~25K is Claude Code's system prompt and tools, the same every session: a cache *read*
  (0.1× input price), not a write.
- ~17–24K is written new: the prime prompt, CLAUDE.md and the rest of the setup.
- The startup turn adds 13–20K over 18–39 tool calls.
- The real cost of a restart is the whole startup turn. For b2005ca8 that was ~70K cache writes,
  1.15M cache reads and 12K output: **~10% of a one-hour session**.

**Cost is mostly cache reads, so it grows with context size.** Every call re-reads the whole
context. b2005ca8 so far: 197 calls, 17.6M cache reads, 214K cache writes, 96K output. At the
usual price ratios (read 0.1×, write 1.25–2×, output 5× input), the reads are about two-thirds of
the cost. **A call at 200K costs twice a call at 100K, so restarting later costs more, not
less.** A rough model with these numbers (about 460 tokens of growth per call, a restart ≈ 10%
of a session) puts the cheapest restart at **~100–150K**. The model leaves out the cost of
losing continuity, which is real but not measured.

**What fills the context:**

| Session | From the human's conversation | From watcher wakes |
|---|---|---|
| 8d0a517c (1 h) | 106K (10 turns) | 54K (17 wakes) |
| b2005ca8 (running) | 92K (8 turns) | 40K (8 wakes) |
| fc26fe40 (8 h) | 91K (12 turns) | 219K (106 wakes) |

- Today, the human's conversation is 60–70% of the growth.
- Over a long session the wakes dominate, and they are mechanical. In fc26fe40, 67 of 83
  classified wakes were `MAIN MOVED`, and the session started dozens of its own background
  "Watch CI for the X merge" tasks. Each wake costs ~2K of context and ~6 calls.

## Decided (the human, 2026-09-29)

1. **Q&A goes to the advisor.** The human sends questions to the advisor, which isn't
   interrupted by system events and wakes. The orchestrator hears from the human about direction
   and decisions.
2. **Mechanical monitoring is automated in bridle, not done by the orchestrator** ("a big
   benefit"). The daemon already holds the wake conditions (`bridle wait-for-wake`, slice 1b,
   `docs/design/agent-host/orchestrator-supervision.md` §5). It should wake the orchestrator only
   for what needs judgment. Concretely:
   - Drop the **main moved** wake. A merge the manager reports comes as a message. A red main
     comes as the CI-failure wake.
   - Remove the role's "when `main` moves, `gh run watch` in the background"
     (`workflow/base/roles/orchestrator.md`, "Verify every merge by its CI run"). c8qw is resolved
     and bridle watches CI, so a failed run on main is already a wake.
   - Check the rest of §5 the same way: each wake needs a decision from the orchestrator, or it
     goes.

### Done for item 2 (br-9e71)

The main-moved wake is gone (daemon, §5, the role's `gh run watch` line). Review of the wakes
that remain, for the human to say which to drop (none dropped):

- **Keep**: a question to the human, a message to the orchestrator, a failed CI run on main,
  context/uptime notes (the orchestrator must act on each).
- **Candidates to drop** (the manager or the daemon can handle them without the orchestrator's
  judgment):
  - `agent_exited` / `agent_crashed` / `agent_stalled`: the manager supervises workers; the
    orchestrator only needs these for the manager itself, or once the manager has failed to deal
    with them.
  - `budget_hold` begins: the governor already holds spawns automatically; nothing to decide.
  - `usage` (five_hour >= 93% / seven_day >= 85%): informational unless the orchestrator's
    role has a step to take at that level.
  - `all_idle` (15 min): needs a look, but often only to read the manager's last note; could go
    to the manager first.

## Open

- **Thresholds** (decided 2026-09-29): 150K / 180K / 200K, now also the defaults (by23). The
  findings suggest earlier may be cheaper still (~100–150K); worth another look once 1 and 2 cut
  the growth rate.
- **Trimming startup** (prime prompt, startup calls): worth doing, least urgent (~10% of a
  session, less once restarts are rarer).
- **Thinking tokens**: output tokens (96K in b2005ca8) far exceed the visible text and tool
  inputs (~16K tokens). Most output is thinking, which isn't in the transcript. Not examined
  further.
