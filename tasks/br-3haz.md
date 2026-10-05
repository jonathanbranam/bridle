+++
id = "br-3haz"
title = "Mail between daemons, slice 1: outbox, peer tokens, forwarding with acknowledgement and dedup (3haz P1, P4, P5)"
kind = "feature"
state = "integrated"
created_at = "2026-10-04T02:32:57.804Z"
updated_at = "2026-10-05T23:22:12.042755Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
branch = "bridle/mail-outbox"
commit = "29296901d9a9c6ab5d63790fb8bffcfad0dc0772"
summary = "Slice 1 of 3haz. `bridle send --project <other> <principal>` now goes to the sender's own daemon (POST /v1/outbox), which accepts at once (`queued o-0007`), stores it in a new `outbox` table and forwards it in the background to the destination (machine config, else registry) via POST /v1/forward with a peer token. Receiver: peer token only, acks with message ids, dedups by origin (machine, daemon, outbox id) in `forwarded_in`; per destination oldest first, a transient failure stops the flush (stays queued), a refusal for good marks failed and moves on. Forwarded `from` is qualified with the sender's machine (`agent:w1@nuc`) and believed only from a peer token; a peer token can call nothing else. `bridle token create --peer <machine>` mints `peer:<machine>` on the receiver; paste it under `[peer]` in the sender's credentials.toml keyed by the receiving project (one per sending machine per receiving daemon, not per daemon pair; decision). New PrincipalKind::Peer, wire types in types.rs, new crates/bridle-daemon/src/outbox.rs, tests in tests/outbox_test.rs (two in-process daemons). Docs: principals.md, storage.md, cli.md, api.md, CHANGELOG. The project-resolution test now expects `send --project y` to hit the own daemon. Deferred (later slices): retry loop and start-up ping (a queued message is retried only on the next send to the same destination), visitor mail home, status/message show, --task and @machine addressing across daemons. MIGRATION NOTE: schema SCHEMA_V20 (outbox, forwarded_in) applies by the daemon's normal migration on next start; peer tokens and the outbox are new and optional, so existing projects need no file changes."
+++

original id: 3haz
Ticket (read all of it first; the human decided Q1-Q4 and the wake-as-message design there): docs/tickets/open/daemons-deliver-mail-to-each-other-across-machines-store-and-3haz.md. Also docs/design/agent-host/principals.md, the k7mw design (projects on other machines), docs/design/storage.md. This is slice 1 of 6; the ticket's P8 order is the plan. Later slices (visitor mail home, retry and start-up ping, status/message show, wakes as messages, cross-project task watch) are separate tasks that depend on this one: do NOT build them here.

Goal: a message to a principal on another daemon goes to the sender's OWN daemon, which accepts it at once, stores it in an outbox table and delivers it to the remote daemon over HTTP; the CLI never writes mail to a remote daemon. Scope is every daemon pair, same machine included (Q1).
Build:
- Outbox table (schema migration per storage.md) and a delivery attempt on enqueue (one immediate try is enough; the retry loop is slice 3).
- Peer tokens (P5): one token per pair of daemons, `bridle token create --peer <machine>` (or the smallest equivalent), stored where visitor/per-machine tokens are; the forwarding daemon states the original sender and the receiver trusts that label only from a peer token.
- Forwarding endpoint on the receiving daemon, with acknowledgement and dedup by origin (machine, daemon, message id) so a retry never delivers twice; per peer, oldest first (P4).
- `bridle send` to a remote project/principal now enqueues on the local daemon (keep k7mw addressing, P7); the sender gets a message id at once.
Files likely: crates/bridle-daemon (store, server, new outbox module), crates/bridle-api/src/types.rs (wire change: update all clients together), crates/bridle/src (send, token commands), docs/design/agent-host/principals.md, cli.md, storage.md.
Migration: the schema change follows the daemon's normal DB migration; peer tokens and the outbox are new and optional, so existing projects need no file changes. State this in the done note.
Acceptance: just check passes; tests with two in-process daemons: send enqueues locally, delivers, an ack lost then retried does not duplicate, order is oldest first, a peer-token label is trusted and a visitor token's is not. Docs updated in the same change.
Model: Sonnet. Out of scope: the retry loop and start-up ping, forwarding visitor mail home, status lines, wake reasons as messages, task watch across projects.

## Thread

### question · external:advisor · 2026-10-04T02:32:57.831Z
Waiting for the human's review of the ticket's proposals P1-P8 and questions Q1-Q4 (docs/tickets/open/daemons-deliver-mail-to-each-other-across-machines-store-and-3haz.md). Don't plan or build until the human answers.

### answer · external:advisor/doc-review · 2026-10-04T15:01:38.282Z
From the human, via advisor (doc-review): "3haz is approved for work." The proposals and questions stand as written. Note: the three [!comment] threads in the ticket are the human's tests of document review (waiting on br-wjhp), not changes to the design. The document's agent handles them, so leave them alone.

### note · system · 2026-10-05T15:24:32.220Z
open 4h, never planned: back to pending. Ready it again once someone will plan it.

### note · agent:pm-1 · 2026-10-05T21:04:54.509Z
PM (pm-1): split the ticket into slices, this task is now slice 1 and is planned and first in the queue. Slices 2-4 are new tasks, pending until approved: br-n7cg (visitor mail forwarded home), br-fvkq (retry and start-up ping), br-cufw (visible state, message show, who-can-I-message). Approve them with `bridle task ready` (the human approved the ticket as a whole, 2026-10-04); I will plan them and add edges on this task (br-n7cg and br-fvkq on br-3haz; br-cufw on br-3haz and br-fvkq). Not yet filed, wait until slice 1 lands: every orchestrator wake as a message with one waiter per principal, and task watch across projects through the peer token. The ticket file has a small uncommitted edit (tasks list).

### note · agent:mail-outbox · 2026-10-05T21:47:25.729Z
done: slice 1 (outbox, peer tokens, POST /v1/forward with ack and dedup, send --project via own daemon); just check exit 0, 1256 tests passed, ran on e6cdf3c6; tip 0ac63a57 only adds a commit deleting a stray .bridle/check.log that I committed by mistake (no code change). Migration: SCHEMA_V20 by normal migration, new optional tables/tokens, no project file changes. Summary recorded.

### note · agent:manager-2 · 2026-10-05T23:19:30.524Z
integrated: 29296901d9a9c6ab5d63790fb8bffcfad0dc0772 (branch bridle/mail-outbox)

### note · agent:manager-2 · 2026-10-05T23:22:12.042Z
cleanup: removed agent mail-outbox, branch bridle/mail-outbox
