---
id: 2ax5
title: "Incident: br-3haz broke every cross-project message for ~22 h: the CLI needed an outbox no running daemon had, and peer tokens nobody had made"
kind: incident
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [3haz, t3vq, n63z, 95mu]
tasks: []
---

## The ask

The human, 2026-10-06 ~5:45 PM ET (verbatim): "Agree - this is an incident that warrants a post-mortem; critical changes like to messaging need a thorough design review that includes how to handle when the change will break running daemons; they should have a safe migration path."

## What happened (as known; the postmortem completes it)

- br-3haz slice 1 (29296901, landed 2026-10-05 7:02 PM ET; CLI installed 7:39 PM) changed `bridle send --project <other>`: it no longer sends straight to the other daemon. It POSTs to the sender's own daemon's `/v1/outbox`, which forwards with a peer token.
- When it landed, no running daemon had `/v1/outbox`: bridle's started 14 s before the landing (self-upgrade to 29296901 then deferred, "no quiet point"), and bridle-ui's and track-web's were from Oct 4. The endpoint returned 404 with an empty body; the CLI printed `error: unknown: `.
- No peer tokens existed (`[peer]` in credentials.toml), so even an upgraded daemon would have queued messages and never delivered them.
- Every cross-project send failed for about 22 h (from ~8 PM 10-05): aides relaying the human, the orchestrator. The bridle-ui aide reported it at ~8 PM; it was filed as a bug (br-ubdc) and not recognised as a regression until 5:30 PM 10-06.
- Same-project sends kept working.

## The postmortem must answer

- How did a breaking change to messaging pass planning, review and landing with no migration path? What did the brief say about compatibility with running daemons and missing tokens?
- Why does a new CLI talk to an old daemon at all, and why did a 404 become "unknown"? (t3vq covers the version-skew side.)
- Why was it filed as a bug and not an incident, and why did it take 22 h to find the cause?
- What gate would have stopped it: a design and proposal reviewed for risk and impact before building (ticket 95mu), a compatibility check in CI, a staged rollout (daemon first, then CLI), a fallback until the peers are configured.

Recovery in progress: restart the daemons into the installed build (has 3haz), the human mints peer tokens (br-y7ht), then n63z automates the tokens.

Related: 3haz, t3vq, n63z, br-ubdc, br-y7ht, br-2y3m.
