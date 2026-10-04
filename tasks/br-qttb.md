+++
id = "br-qttb"
title = "x8jt: 'bridle review now <path> [--resend]' sends pending threads at once; sent marks in the file; gateway route"
kind = "feature"
state = "open"
created_at = "2026-10-04T02:25:31.949Z"
updated_at = "2026-10-04T02:25:39.389789Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "M"
+++

Approved by the human 2026-10-04, relayed verbatim by advisor doc-review (m-4238): "Add a bridle command to perform the review on a document immediately and a button in the UI also to request the review. Comments that have been sent for review should be marked as such and not resent if the button is pressed again, unless requested." Where the sent mark lives: "A" (in the file). Thread IDs deferred ("Comment threads probably need a UID as well but we could wait on that"). Spec: ticket x8jt, last section (commit a6a0a71).

bridle side. 'bridle review now <path> [--resend]' sends a document's pending threads to its agent at once, skipping the quiet period. Whenever bridle sends a batch (quiet period or now), it appends '· sent HH:MM' to the line of each thread's newest human entry; later sends skip entries already marked sent unless --resend. Add a gateway route the UI button calls (same semantics, resend flag). Update docs/design/agent-host/daemon.md, docs/design/cli.md and human-web-ui.md, and CHANGELOG.
