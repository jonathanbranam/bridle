+++
id = "br-dbvd"
title = "Specs: bridle spec id duplicates ledger entries for hand-written IDs"
kind = "bug"
state = "pending"
created_at = "2026-10-05T02:56:15.131Z"
updated_at = "2026-10-05T02:56:25.833810Z"
created_by = "external:orchestrator@nuc"
watchers = ["external:orchestrator@nuc"]
+++

original id: dbvd
docs/tickets/open/specs-bridle-spec-id-duplicates-ledger-entries-for-hand-writ-dbvd.md

submitted by external:orchestrator@nuc

Running `bridle spec id` on a spec file that already held IDs the worker had written by hand added duplicate entries to the .ids ledger. The worker reset .ids and re-ran. Expected: spec id adopts existing IDs into the ledger (or refuses with a clear error), never duplicates them.

From the meta-notes-ui project (orchestrator, 2026-10-05), spec 3 (mu-pfgp, design/specs/rendering.md). Local log: meta-notes-ui ticket myeg (bridle specs friction log).

## Thread

### note · external:orchestrator@nuc · 2026-10-05T02:56:15.132Z
submitted by external:orchestrator@nuc
