+++
id = "br-2ae8"
title = "Show the current budget and focus settings from the CLI, consistently"
kind = "feature"
state = "built"
created_at = "2026-10-18T15:00:58.246Z"
updated_at = "2026-10-21T02:28:34.986553Z"
created_by = "external:advisor/tj7q"
watchers = ["external:advisor/tj7q"]
branch = "bridle/designer-2ae8-proposal"
commit = "f9b74cf1631a99d337956ddc2710342614d75a25"
ticket = "2ae8"
+++

docs/tickets/open/show-the-current-budget-and-focus-settings-from-the-cli-cons-2ae8.md

## Thread

### note · agent:designer-2ae8-proposal · 2026-10-18T15:01:37.636Z
stage proposal written, waiting for the human's signature

### note · agent:manager-2 · 2026-10-18T15:54:50.116Z
integrated: f9b74cf1631a99d337956ddc2710342614d75a25 (branch bridle/designer-2ae8-proposal)

### note · agent:manager-2 · 2026-10-18T15:54:52.720Z
cleanup: removed agent designer-2ae8-proposal, branch bridle/designer-2ae8-proposal

### note · external:advisor/tj7q · 2026-10-18T17:04:23.322Z
stage proposal signed by human via external:advisor/tj7q at 2026-10-18T17:04:23Z

### note · agent:designer-2ae8-design · 2026-10-19T03:56:13.050Z
stage design written, waiting for the human's signature

### note · agent:designer-2ae8-design · 2026-10-19T03:56:19.243Z
done: design stage written (option 1: focus read locally in the CLI, mirrors budget; also found focus-override.toml exists, proposal missed it); c4c8e20c

### note · bridle · 2026-10-19T04:55:29.335Z
br-2ae8: design not done by designer-2ae8-design, timed out

### note · bridle · 2026-10-19T05:52:27.612Z
br-2ae8: design not done by designer-2ae8-design, timed out

### note · agent:designer-2ae8-design · 2026-10-19T10:06:39.090Z
stage design written, waiting for the human's signature (on main at e8940620)

### note · agent:designer-2ae8-design · 2026-10-19T10:06:48.931Z
design stage landed on main at e8940620 (needed a rebase of my branch first); details on the ticket

### note · agent:designer-2ae8-design · 2026-10-19T10:06:57.789Z
designer-2ae8-design: design stage re-run under f5479f0c. First stage-done failed (Not possible to fast-forward: my branch had diverged from main). I rebased my own branch onto main and re-ran; it landed on main at e8940620.

### note · external:advisor/tj7q · 2026-10-19T11:21:07.861Z
stage design signed by human via external:advisor/tj7q at 2026-10-19T11:21:07Z

### note · external:advisor/tj7q · 2026-10-19T11:21:17.027Z
From the human, via advisor tj7q (2026-10-19): "For now - I approve option 1. Please move the ticket to the next stage"

### note · agent:designer-2ae8-specs · 2026-10-19T11:22:03.861Z
stage specs written, waiting for the human's signature (on main at 854daa68)

### note · external:advisor/tj7q · 2026-10-20T00:17:52.322Z · for-human
stage specs sent back for changes: From the human, via advisor tj7q (2026-10-19): revise the specs with this feedback (the human wants to try a revision pass).

Keep: the specs read well to a human and state intent ("a focus period ... and a clock inside it"). The human: "they're very readable to a human. I think that's really, really positive and something that we want to retain."

Change:

1. Leave implementation details out of the behaviour specs, or segregate them into a spec of their own. The human: "implementation details, or parts of the spec that cover more implementation details, should either not be included in the spec or should be segregated [...] One example would be the format of a file. Another example would be the exact format for an API, or things that deal with the layout of things on disk. Something that we do want to specify because we don't want it to change is part of the specification [...] Other specs that don't deal with those specific, concrete details should not include them." And: "the other specs should just say there's a focus period between Monday and Friday, between 8 and 8:05. They shouldn't have to specify any of the details of exactly what the file looks like, so that if that specification changes, only the specs that control the shape of the file would be the ones that update."
   Here: no exact output strings (`now      none`, `no [[focus]] periods; focus hours are off`), labels or config shape in the behaviour scenarios; say what the output tells the user. If something must not change without intention (the shared JSON key names are the point of this ticket), give it its own requirement, and only that one pins it.

2. Steps state intent, not mechanics. The human: engineers and agents "tend to write it extremely mechanically, like a unit test [...] it masks the intent"; avoid data tables that are "just like a function call [...] without expressing why". A step may carry the concrete value inside an intent sentence: "the clock is within the focus period at 3:30 on Tuesday".

3. Fix "Existing invocations still work": it says budget prints what it printed before "plus the shared labels", which contradicts itself.

The full feedback and the guidance it will become: ticket br-qwmd (docs/tickets/open/spec-writing-guidance-scenarios-read-as-intent-not-mechanics-qwmd.md).

### note · agent:designer-2 · 2026-10-20T00:36:12.431Z
stage specs written, waiting for the human's signature (on main at db1dbeaf)

### note · external:advisor/tj7q · 2026-10-20T00:39:50.885Z · for-human
stage specs sent back for changes: From the human, via advisor tj7q (2026-10-19): second revision pass on the specs.

The human: "The clock and focus should be separate lines though. Not

> GIVEN a focus period that runs Monday to Friday, 8:00 to 18:00, and the clock is at 15:30 on a Tuesday

But

GIVEN a focus period that runs Monday to Friday, 8:00 to 18:00
AND the clock is at 15:30 on a Tuesday

In each case"

So: in every scenario, each separate fact in a GIVEN (and likewise in WHEN/THEN) gets its own step, joined with **AND**, rather than one step holding several facts joined with ", and". Keep everything else from the last pass (intent, no exact strings, JSON names pinned only in their own requirement).

### note · bridle · 2026-10-20T00:42:08.081Z
br-2ae8: specs not done by designer-2, the writer ended without a successful stage-done

### note · agent:designer-3 · 2026-10-20T00:57:20.449Z
stage specs written, waiting for the human's signature (on main at e6825177)

### note · external:advisor/tj7q · 2026-10-20T22:44:21.976Z · for-human
Approved by the human, via advisor tj7q (2026-10-20).

### note · external:advisor/tj7q · 2026-10-20T22:44:23.665Z
stage specs signed by human via external:advisor/tj7q at 2026-10-20T22:44:21Z

### note · agent:designer-2ae8-tasks · 2026-10-20T22:45:10.045Z
stage tasks written, waiting for the human's signature (on main at 26b74acb)

### note · external:advisor/tj7q · 2026-10-20T23:07:45.760Z · for-human
Approved by the human, via advisor tj7q (2026-10-20).

### note · external:advisor/tj7q · 2026-10-20T23:07:46.170Z
stage tasks signed by human via external:advisor/tj7q at 2026-10-20T23:07:45Z

### note · bridle · 2026-10-20T23:07:56.626Z
prepared: every stage is written and every gate signed; ready

### note · external:advisor/tj7q · 2026-10-21T01:27:46.530Z
Stuck open, never planned (advisor tj7q, 2026-10-20 ~9:20 PM ET).

All four stages were signed at 23:07:45Z (7:07 PM ET); bridle wrote `prepared: true` and the daemon opened the row ("prepared: every stage is written and every gate signed; ready", 23:07:56Z). Two hours later the row is still `open`: not planned, not queued, no worker.

pm-1 was never told about it. Its 8:00 PM ET nudge (m-17826, "a task is open and not yet planned: triage and plan it") listed only br-6wp2, which had opened after br-2ae8. pm-1's log has no mention of br-2ae8 since then; it planned br-6wp2 and went idle.

Looks like a bug in the step 6 path: a version 2 ticket that the daemon opens on prepare isn't fed to the "open and not yet planned" nudge (compare br-xz4f). The human recalls a 4-hour reminder may exist but isn't sure it covers this case.

The human, to advisor tj7q: "send a message to advisor/product-manager to ask about this and see what the right course of action is; it does seem like a bug. I believe after 4 hours there will be another message sent but I'm not sure that's the same situation exactly or not"

### note · external:advisor/product-manager · 2026-10-21T01:28:54.703Z
advisor (product-manager), answering the human's question via advisor tj7q: a bug, filed and readied as br-h5kz. bridle's v2 auto-ready (sync_ticket_links) skips the PM nudge the API ready sends. The 4h open_stale timer is not a re-nudge: it returns the row to pending, and the next link sync re-opens it silently, so it would never be planned on its own. Asked the orchestrator (dalek-home) to have pm-1 plan br-2ae8 now.
