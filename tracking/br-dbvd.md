+++
id = "br-dbvd"
title = "Specs: bridle spec id duplicates ledger entries for hand-written IDs"
kind = "fix"
state = "integrated"
created_at = "2026-10-05T02:56:15.131Z"
updated_at = "2026-10-18T02:01:53.557343Z"
created_by = "external:orchestrator@nuc"
watchers = [
    "external:orchestrator@nuc",
    "external:advisor/product-manager",
    "agent:wdbvd",
]
branch = "bridle/wdbvd"
commit = "e6dc1eb5886c31dad5e3dfaafaa429e4a8ee00bf"
summary = "Not reproducible on main: specid::assign already adopts ids found in files into the .ids ledger (also with CRLF ledger lines) and never duplicates; running the built binary twice on a file with hand-written ids left the ledger unchanged. Added regression test hand_written_ids_are_adopted_once_across_runs in crates/bridle/src/specid.rs (mixed hand-written and missing ids, ledger with CRLF, two runs, ledger count unchanged). No code fix and no CHANGELOG line, since behaviour did not change. Possible original cause (unverified): spec id run with explicit paths and no --root/--ledger writes a new ledger at design/specs/.ids relative to cwd, not the project's ledger."
ticket = "dbvd"
+++

docs/tickets/open/specs-bridle-spec-id-duplicates-ledger-entries-for-hand-writ-dbvd.md

submitted by external:orchestrator@nuc

Running `bridle spec id` on a spec file that already held IDs the worker had written by hand added duplicate entries to the .ids ledger. The worker reset .ids and re-ran. Expected: spec id adopts existing IDs into the ledger (or refuses with a clear error), never duplicates them.

From the meta-notes-ui project (orchestrator, 2026-10-05), spec 3 (mu-pfgp, design/specs/rendering.md). Local log: meta-notes-ui ticket myeg (bridle specs friction log).

## Thread

### note · external:orchestrator@nuc · 2026-10-05T02:56:15.132Z
submitted by external:orchestrator@nuc

### note · agent:pm-1 · 2026-10-05T02:56:34.784Z
Triage (pm-1): accept. A real bug from meta-notes-ui. Ticket minted: docs/tickets/open/specs-bridle-spec-id-duplicates-ledger-entries-for-hand-writ-dbvd.md (uncommitted; needs committing on main). Stays pending: approve with `bridle task ready br-dbvd`, then I plan it (small; Sonnet, crates/bridle-spec; Haiku may do).

### note · external:advisor/product-manager · 2026-10-09T11:04:38.516Z
watching the task

### note · external:advisor/product-manager · 2026-10-11T07:22:56.703Z
kind: bug -> fix

### note · external:advisor/product-manager · 2026-10-17T15:26:06.084Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.

### note · agent:wdbvd · 2026-10-18T01:07:55.417Z
built: 17b63e85 (branch bridle/wdbvd)

### note · agent:wdbvd · 2026-10-18T01:07:57.604Z
done: bug not reproducible on main (spec id already adopts hand-written ids, no duplicates); added regression test, no code fix, no CHANGELOG line (no behaviour change). just check exit 0, 1835 tests, commit 17b63e85. Details in ticket summary.

### note · agent:manager-2 · 2026-10-18T02:01:53.557Z
integrated: e6dc1eb5886c31dad5e3dfaafaa429e4a8ee00bf (branch bridle/wdbvd)
