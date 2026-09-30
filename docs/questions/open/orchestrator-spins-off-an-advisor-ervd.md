---
id: ervd
title: The orchestrator spins a discussion off to a fresh (or existing) advisor
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: [session-names-per-machine-sfb3]
see: []
---

## The ask

The human, verbatim (2026-09-29, via the advisor):

> How possible would it be for bridle to start an advisor for me? Eg I'm chatting with orch
> (needs a shorter name btw) and I want to take an issue offline with a fresh advisor - I'd like
> to say "hey orch, you're a busy guy, that's a good question you asked me, have bridle start a
> new advisor for that."
>
> I guess there are a few questions here like where does the session live in TMUX or not what
> TTY is the session attached to.
>
> If I'm remote, it would show up as a new session, NBD, but locally I would want a tmux pane.
> Honestly it doesn't seem terribly hard to do but would need some configuration similar to the
> orchestrator tmux we set up. Something like "start new advisors in a new pane in this window"
> or something.
>
> Anyway, not urgent I can start them myself. The hand off from orch though would be handy to
> have in his wheelhouse - a prompt or skill that the human might spin off a discussion into a
> fresh advisor (or existing one) - and that multiple advisors may exist and include their name
> in messages.

## Two parts

**1. The handoff, now (role change only).** `workflow/base/roles/orchestrator.md`: the human may
spin a discussion off to an advisor, fresh or existing. When asked, the orchestrator writes a
short brief (the question, what's known, links to tickets and tasks) and either sends it to the
advisor (`bridle send external:advisor "For advisor <name>: ..."`, since advisors share one
inbox) or tells the human how to start one with it (`scripts/claude-advisor <name>`, plus the
brief). It also knows several advisors may be running, each signing its messages with its name
(sfb3).

**2. Bridle starts the advisor, later (not urgent).** Something like `bridle advisor start <name>
--brief <file>`:

- Runs `scripts/claude-advisor <name>` in a new tmux pane. Where is configuration, like the
  orchestrator's pane tag (`docs/design/agent-host/orchestrator-supervision.md`: the daemon
  already finds a tagged pane and types into it with `tmux send-keys`), e.g. "split the
  orchestrator's window" or "a new window in its session".
- The brief is a message, not a file (settled below).
- Remote: the session shows up in Claude mobile by its Remote Control name like any other; the
  pane still lives in tmux on that machine.
- The orchestrator can run it itself when the human asks.

## Also

The human finds `bridle-orch` too long for the orchestrator's session name; a shorter one (e.g.
`orch`, with the host per sfb3) belongs with sfb3.

## The brief is a message to the advisor by name (the human, 2026-09-29)

> It should be a message addressed to the advisors name. Orch and I would say "spin up advisor
> Fred and we'll talk about worker pooling" or whatever then the advisor knows he is Fred and the
> message is probably waiting for him when he starts.

So:

- The orchestrator sends the brief first, addressed to the name:
  `bridle send external:advisor "For advisor fred: <the topic, what's known, links>"`, then starts
  (or asks the human to start) `scripts/claude-advisor fred`.
- The advisor knows its name from the script (e.g. `BRIDLE_ADVISOR_NAME`). Its opening step reads
  the inbox for messages addressed to its name, and starts on the waiting brief.
- Advisors share one inbox (sfb3), so a message "For advisor <name>" is for that advisor only; the
  others leave it alone, and don't mark it read.
- If shared-inbox addressing gets messy, a later step is a real per-name recipient (e.g.
  `external:advisor:fred`); not now.
