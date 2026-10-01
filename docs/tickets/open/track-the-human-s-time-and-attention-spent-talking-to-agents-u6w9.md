---
id: u6w9
title: Track the human's time and attention spent talking to agents
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: [cvaq]
---

## The ask


The human, 2026-10-01, verbatim:

> file a ticket for work that I'd like to get started today -
>
> tracking the human's time and attention in bridle
>
> I want a system (can be basic to start with) that tracks how much time the human spends
> interacting directly with bridle agents. Basic reporting is ok here - keep track of messages that
> arrive at interactive bridle agents (maybe they're in events already? IDK). the data collection
> part should be started immediately. the result should be that we can build a report of the
> human's time: daily, weekly, typical hours, stat/stop etc. things like that. I want to know how
> much of my time is spent talking to agents.
>
> We don't need to track things outside of bridle or things the system cannot reasonably find out.
> Let's see... so interactions with agents is the main and primary one; we could also track if the
> human uses the CLI to interact with tasks or in other ways; KISS here though, the main, big big
> thing is: how long do I spent sitting here talking to agents.

Priority: start today. Collection first, because history can't be recovered later; the reports
can follow.

## Today

- The human talks to agents mostly by typing into interactive sessions: the orchestrator and
  the advisors, started by `bridle session`. These sessions run outside bridle, so the human's
  prompts reach no bridle event or message.
- `bridle session` already gives both roles a `UserPromptSubmit` hook, `bridle focus gate`
  (cvaq, `crates/bridle/src/session.rs:22`). It runs on every prompt the human types in those
  sessions, so it is one place a timestamp per prompt could be recorded.
- What the human sends through the CLI (`bridle send`, `bridle task ...` as `human`) already
  reaches the daemon as messages and events.

## Wanted

- **Collection, now:** a record of each time the human interacts: when, which session or agent,
  which machine. Interactive sessions first; the human's CLI calls are optional.
- **Reports, later:** the human's time talking to agents, by day and by week; usual hours; when
  each day started and stopped.
- Out of scope: anything outside bridle, or anything bridle can't reasonably find out.
