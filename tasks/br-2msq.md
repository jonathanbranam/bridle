+++
id = "br-2msq"
title = "bridle send --project to a local daemon needs a peer token, and --url can't be combined with --project"
kind = "bug"
state = "planned"
created_at = "2026-10-10T02:31:14.507Z"
updated_at = "2026-10-10T02:59:43.823766Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "2msq"
+++

Ticket: docs/tickets/open/bridle-send-project-to-a-local-daemon-needs-a-peer-token-and-2msq.md (read it). Goal: (1) bridle send --project <p> uses the caller's direct token for that project when it holds one on this machine (the way task ready/comment --project already do), and goes through the outbox/peer-token path only for a project it has no direct token for; (2) --url together with --project is accepted: --project names the token, --url picks the daemon. Files: crates/bridle/src/commands (send, project/token resolution; grep 'no peer token' and 'the daemon was found by URL'), docs/design/cli.md, docs/design/agent-host/principals.md (Mail between daemons: when send uses the outbox), CHANGELOG. Do not break slice 1 (br-3haz): a remote machine or a project with no direct token still enqueues on the own daemon. Acceptance: just check passes; tests: direct token present -> send goes direct, not outbox; absent -> outbox as before; --url + --project resolves the token. Migration: none (CLI behaviour). Model: Sonnet. Out of scope: visitor mail forwarding (br-n7cg), status lines (br-cufw).

## Thread

### note · external:advisor/product-manager · 2026-10-10T02:31:16.811Z
advisor (product-manager): placed: epic machine-setup (project setup across daemons), normal, after br-hdbj. Pending until the human approves.

### note · external:advisor/product-manager · 2026-10-10T02:59:22.334Z
advisor (product-manager): readied. The human, 2026-10-09 ~10:55 PM ET: "if the machine work finishes up, let's prioritize work that makes sending and receiving messages work better and more reliable, reducing waiter counts like the orc has 5 waiters; I think the scheduled message work is also an important epic to finish up soon". New epic messaging (theme agents-and-cli), ranked right after machine-setup. Order: br-2msq, br-n7cg, br-rhba, br-ysmu, br-cufw.
