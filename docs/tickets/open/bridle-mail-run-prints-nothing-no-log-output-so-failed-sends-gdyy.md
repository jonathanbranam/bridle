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
tasks: []
---

## The ask

Found 2026-10-07 ~8:50 PM ET while the human tested the mail bridge on dalek. The orchestrator's "got it" reply (m-6721, `reply_to` m-6720) sat unread for ~15 minutes: every SES send was refused (`MessageRejected: Email address is not verified ... jonathan.branam@gmail.com`, the SES sandbox), yet the bridge's terminal showed nothing. The human: "sorry, no output in the bridle terminal I meant". The cause was found only by sending through SES by hand with the bridge's key.

Why: `bridle mail run` (`crates/bridle/src/commands/orchestrator.rs`, `mail_run`) installs no tracing subscriber, unlike `serve` (`serve.rs` ~182) and `gateway` (`gateway.rs` ~50). So the bridge's `tracing::warn!("mail outbound failed: ...")`, `"mail poll failed"`, dropped-mail warnings and `info!` lines (`reply relayed`, `digest mailed`) all go nowhere.

The ask: `bridle mail run` logs like `serve` and `gateway` (stderr, `RUST_LOG`-style filter, info by default), so a failed send, a dropped mail and a relayed reply show in its terminal, or its log file once it runs as a service.
