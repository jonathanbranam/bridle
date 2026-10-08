---
id: gdyy
title: "bridle mail run prints nothing: no log output, so failed sends are silent"
kind: bug
opened: 2026-10-08
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [rs7p, 843g]
tasks: [br-gdyy]
---

## The ask

Found 2026-10-07 ~8:50 PM ET while the human tested the mail bridge on dalek. The orchestrator's "got it" reply (m-6721, `reply_to` m-6720) sat unread for ~15 minutes: every SES send was refused (`MessageRejected: Email address is not verified ... jonathan.branam@gmail.com`, the SES sandbox), yet the bridge's terminal showed nothing. The human: "sorry, no output in the bridle terminal I meant". The cause was found only by sending through SES by hand with the bridge's key.

Why: `bridle mail run` (`crates/bridle/src/commands/orchestrator.rs`, `mail_run`) installs no tracing subscriber, unlike `serve` (`serve.rs` ~182) and `gateway` (`gateway.rs` ~50). So the bridge's `tracing::warn!("mail outbound failed: ...")`, `"mail poll failed"`, dropped-mail warnings and `info!` lines (`reply relayed`, `digest mailed`) all go nowhere.

The ask: `bridle mail run` logs like `serve` and `gateway` (stderr, `RUST_LOG`-style filter, info by default), so a failed send, a dropped mail and a relayed reply show in its terminal, or its log file once it runs as a service.

## Also: failures as events, without spam

The human, 2026-10-07 ~8:55 PM ET, via aide: "yes file for warnings; can they be events as well? will is spam though?"

Aide's reading, for the planner (not a decision): today only the daemon writes events (`docs/design/agent-host/api.md`, "Events"); the bridge only calls `POST /v1/messages`, so an event from it means either a new way for a client to record one or a message (a `system`-style note to the project's aide, ticket 843g). Against spam: record on a **change of state** only, not on every retry (the bridge retries every `poll_secs`, 30 s): one entry when a kind of failure starts (send refused, S3 unreachable, daemon down; keyed by the error kind and, for a refused send, the address), one when it clears, nothing in between. A dropped inbound mail (not allowlisted, no DMARC pass) is one entry per mail, which strangers can make many of, so those could stay log-only or be counted in the digest.
