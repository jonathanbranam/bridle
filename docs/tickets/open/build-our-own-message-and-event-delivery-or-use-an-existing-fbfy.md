---
id: fbfy
title: Build our own message and event delivery, or use an existing system?
kind: question
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [xxxq, rmpq]
tasks: [br-7c7c]
---

## The ask


The human, verbatim (2026-10-03, via the advisor; full message in
[[a-message-an-agent-receives-is-read-no-separate-mark-read-st-rmpq|rmpq]]):

> Like, some messages are just like, like we're, we're creating kind of event, an event delivery
> system, right? So I want to talk about that too, and whether we should be building our own
> secure message delivery system or just like using one that exists or something. But, That's a,
> that's a separate question, but, you know, something to think about.

## The question

Bridle's messages are becoming its one notification system (task changes, xxxq; read on
delivery, rmpq): an event delivery system. Keep building it (SQLite in each daemon, HTTP and
long polls, per-project daemons, Tailscale between machines), or use an existing message broker
or event system? What would one give us (delivery guarantees, fan-out, cross-machine), and what
would it cost (another service to run on every machine, auth, the human's KISS rule)?

Not started. For a later discussion with the human.
