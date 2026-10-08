+++
id = "br-9xze"
title = "Scheduled messages, first slice: an agent schedules a message to itself (one-time or recurring), bridle schedule add/list/rm"
kind = "feature"
state = "open"
created_at = "2026-10-08T14:28:58.540Z"
updated_at = "2026-10-08T14:29:05.435849Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:orchestrator@nuc",
]
parent = "br-yfv5"
+++

First slice of yfv5 / hrcn: per-project scheduled messages that an agent sets for itself (the human may set them for any agent too). Tickets: docs/tickets/open/one-scheduler-for-timed-actions-scheduled-messages-hrcn-nigh-yfv5.md and docs/tickets/open/scheduled-messages-an-agent-or-the-human-schedules-a-message-hrcn.md.

Approval (the human, 2026-10-08, relayed verbatim by the meta-notes aide, m-7089): "Yeah, let's go ahead and schedule that. I think the primary thing there is just what you described: an agent wants to set itself a wake-up. This will really improve the wake-up handling and things, so that agents, instead of waking themselves up with shorter wake-up timers, start using this functionality. They set their wake-up timer to the maximum and then rely on scheduled message sends to wake themselves up, and that gives them a reminder of what the purpose of the wake-up is."

The proposal they approved: "approve just the first slice: per-project scheduled messages, one-time and recurring, that agents can set for themselves".

Scope (yfv5's shape, this slice only):
- A schedule record in the daemon DB (survives restarts): target, message body, one-time (at a time) or recurring (cron), the human's time zone with DST, and a rule for a missed firing (e.g. after the daemon was down or the machine slept).
- At the time, the daemon sends the message to the target as from the scheduler (or on behalf of the agent that set it), so it wakes the agent and says why.
- CLI: `bridle schedule add/list/rm`, usable by agents (for themselves) and the human.
- Role priming follow-up (part of this task or a child via --from): tell agents to wait at the maximum timeout and schedule a message for timed wake-ups, instead of short timers.

Not in this approval: nightly session restarts (br-cbbn) and maintenance windows (3nyk); machine-wide actions (cy2v).
