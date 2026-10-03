---
id: r9vh
title: Put background agents to sleep and wake them on demand, with a cap on how many are awake
kind: question
opened: 2026-10-02
repos: [bridle]
changes: []
specs: []
needs: []
see: [x8jt, yj38, pdmd]
tasks: [br-7973]
---

## The ask


The human, verbatim (2026-10-02, via the advisor), on x8jt's one-agent-per-document option:

> Yeah, a few things. I definitely like B in general. I think it's the right approach. Probably
> the we need a kind of an expiry for this agent. This kind of reminds me of another question I
> have about our agents, our background agents at least, is like, can we shut them down? Can we
> persist their context and shut them off? And if not, why not? Or could we ask them to summarize
> their context and then shut them off? Just in terms of like thinking, you know, what if I get
> really busy and add comments to a bunch of tickets? You know, we need a governor on this
> somehow. We don't want 23 agents spinning up all at once. But yeah, like, I know we're using
> Claude Code to run all of this stuff, but under the hood, it's an LLM with tool calls embedded
> in it. And that process doesn't need to stay in memory. Like, if I were using a different LLM,
> you know, I don't need to keep it all in memory. I could persist the context to disk anytime
> and just stop. So, let's, why don't you open a ticket on that topic? And I, you know, explain
> how we're handling background agents with the JSON streaming so that I can have a better sense
> of it and what would be involved in being able to put them to sleep for a period of time,
> which, to, to which I mean, you know, persisting their context either literally, like exactly
> what's in their context, or, you know, writing up some kind of summary of the context and then
> just just stopping the process so we could bring it back later. I think that would be an
> interesting capability to have.

## What it asks

1. **Put an agent to sleep:** stop its process and bring it back later, either with its exact
   context or from a summary it writes first.
2. **An expiry** for short-lived agents such as x8jt's per-document agent.
3. **A governor:** commenting on many tickets at once must not start 23 agents.

## How background agents work today (advisor, checked 2026-10-02)

From `docs/design/agent-host/agents.md` and spike 01 (`docs/spikes/01-stream-json-findings.md`):

- **Each agent is one `claude -p` process** in stream-json mode, owned by the daemon through its
  stdin and stdout. The daemon writes messages to stdin as JSON lines; Claude Code writes events
  (turn start, tool calls, text, the turn's `result` with cost) to stdout, which the daemon
  records. Between turns the process sits **idle, waiting on stdin**: it holds memory, but makes
  no model calls and costs nothing.
- **The context already lives on disk.** Claude Code writes every session's transcript to a
  JSONL file (`~/.claude/projects/...`) as it goes. The model holds nothing between calls: every
  turn sends the whole conversation again (most of it read from the prompt cache).
- **Sleep with the exact context exists already, by hand:** `bridle stop <agent>` closes stdin
  and the process exits (state `stopped`); `bridle resume <agent>` starts a new process with
  `claude --resume <session id>`, which reloads the conversation from the transcript (spike S7).
  Messages sent while it's stopped wait and are delivered on resume. Bridle already does this on
  daemon restarts (`resume_on_restart`) and when the budget governor pauses agents.
- **Sleep from a summary exists in part:** `bridle renew <agent>` replaces the process with a
  fresh session (same name, worktree and branch). The orchestrator has a handover note it writes
  before its relaunch (`bridle handover`); other roles have no summary step yet.
- **A governor exists for workers only:** `[budget] max_workers` (default 2) caps concurrent
  workers; managers, the PM and other roles aren't counted
  (`docs/design/usage-and-budget.md`).

## What's missing

- **Automatic sleep:** stop an agent after N minutes idle (an expiry), per role.
- **Automatic wake:** a message (or a new comment, x8jt) to a sleeping agent resumes it. Today
  it waits until someone runs `bridle resume`.
- **A cap on awake agents** of a role (e.g. at most 2 document agents; the rest queue or stay
  asleep until one sleeps), beyond `max_workers`.
- **Summary sleep for any role:** ask the agent to write a summary to its task, then renew from
  it, when the context is large.

## Cost, the real trade-off (advisor)

- Waking with the exact context re-reads the whole conversation. The prompt cache lasts 5 minutes
  (or an hour), so after a longer sleep the whole context is paid again on the first turn at the
  cache-write rate: e.g. a 100K context costs that once per wake. Cheap for a small document
  agent; costly for a long-lived one (pdmd measured the orchestrator's restart cost).
- Waking from a summary is cheap but loses detail.
- So a likely rule: sleep with the exact context while it's small; summarize and renew when
  it's large. An idle process costs only memory, so short idles needn't stop it at all.
