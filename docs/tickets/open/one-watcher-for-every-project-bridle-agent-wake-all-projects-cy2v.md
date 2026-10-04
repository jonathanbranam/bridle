---
id: cy2v
title: "One watcher for every project: 'bridle agent wake --all-projects'"
kind: feature
opened: 2026-10-02
repos: [bridle]
changes: []
specs: []
needs: []
see: [interactive-sessions-context-for-all-daemon-decided-wakes-re-jttf, a-life-assistant-agent-on-the-notes-repo-phyy, daemons-deliver-mail-to-each-other-across-machines-store-and-3haz]
tasks: [br-1ddd]
---

## The ask

An orchestrator (or any principal) holding tokens for several projects needs one waiter per
daemon today: each project runs its own daemon (port, database, tokens), and
`bridle orchestrator wait-for-wake` is one long-poll against one daemon. Since 7b22c1e the
orchestrator role runs one waiter per project; miss one and that project's messages are
never seen.

The human (2026-10-01), on the choice between the CLI fanning out and daemons forwarding
wakes to each other: "Yes definitely agree - Recommended: the CLI watches every daemon at
once. bridle agent wake --all-projects".

## Decided

- **Client-side fan-out.** `bridle agent wake --all-projects` long-polls every project the
  caller holds a token for (its section in `~/.bridle/credentials.toml`, resolved through
  the registry as `--project` does) and returns on the first wake from any of them. No
  daemon-to-daemon forwarding.
- Each printed wake names its project, so the reader knows which daemon to act on.
- Wakes that arrive on other daemons while one returns stay queued there (the daemons already
  queue undelivered wakes), so exiting on the first one loses nothing.
- A project whose daemon is down or unreachable is reported once and skipped; it doesn't stop
  the others.

## Order

After `bridle agent wake` absorbs the orchestrator's wake reasons and replaces
`wait-for-wake` (jttf decision 2; parked for Saturday 2026-10-03 because it is the
orchestrator's own loop). Then the orchestrator role drops "one waiter per project" for this.

## Out of scope

Cross-machine projects (`[orchestrator.<machine>]` tokens) unless they come free; a
federated wake service.

## Added (2026-10-03, advisor)

- **Start-up, not just waits.** The human asked whether sessions read messages from other
  machines at start-up; they don't: an advisor checks only its own project's daemon, and `bridle
  inbox` has no all-projects form. Add `bridle inbox --all-projects` alongside the wake, and have
  the advisor and orchestrator roles run it at start-up.
- **Since rmpq the fan-out can lose messages** (a second daemon's reply marks them read, then the
  CLI drops it): see [[read-on-delivery-can-lose-messages-marked-read-before-the-re-k8jn|k8jn]].
  Settle that before building this.

## Likely superseded by 3haz (2026-10-04)

The human approved (2026-10-04, via the advisor) daemons forwarding each principal's messages to
its home daemon, with every wake reason sent as a message, so one waiter per principal is enough:
"Great yes. Cutting down waiters would be great." See
[[daemons-deliver-mail-to-each-other-across-machines-store-and-3haz|3haz]]. Whether to build
this fan-out meanwhile isn't decided: "Orch is special today. I'm not sure. . But other agents
only wake on messages." (the orchestrator is the only one with wakes that aren't messages).

Then (the human, 2026-10-04, verbatim):

> Leave all-projects for now. It'll be a NOOP after this lands. We can warn and remove later. Add
> a follow up ticket to remove it after this lands and rolls out so we don't forget

So br-1ddd stays as planned; removal after 3haz rolls out is
[[remove-bridle-agent-wake-all-projects-once-3haz-lands-and-ro-kuvh|kuvh]].
