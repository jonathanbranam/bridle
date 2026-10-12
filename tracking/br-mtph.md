+++
id = "br-mtph"
title = "Grilling skill build: copy Matt Pocock's grilling (MIT) as bridle-grill, one or two questions at a time, bridle tickets as the record (k9j4 decisions)"
kind = "feature"
state = "integrated"
created_at = "2026-10-12T12:31:13.844Z"
updated_at = "2026-10-12T13:15:22.417556Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
    "agent:wk9j4",
    "agent:wmtph",
]
branch = "bridle/wmtph"
commit = "991a88179effb94008a57b42be3e3f0986e8aa4b"
summary = "Added workflow/base/skills/grill/SKILL.md (bridle-grill): Matt Pocock's grilling (MIT, notice + source URL + commit 95249b0b in the file) adapted: epic-or-ticket first, one or two questions with open count, bridle ticket thread/body as record, child checked against parent epic, never readies/plans/builds, ends on 'Shared understanding?'. Upstream emoji replaced with ASCII; upstream text taken from gh api, body reworded around it. Listed in docs/design/skills.md; CHANGELOG. Not wired into roles."
parent = "br-k9j4"
+++



## Thread

### note · agent:pm-1 · 2026-10-12T12:31:20.386Z
Brief (pm-1). Model: Sonnet. Size S. br-k9j4 already landed as the designer pass (1e51946d, a draft at workflow/base/skills/grill/SKILL.md written from our own text); the human then decided it is a small build. This task is that build; the authority is the "## Decisions (the human, 2026-10-12 ~8:30 AM ET)" section of br-k9j4's ticket file, items 1-6. Read it, and the draft.

Do: replace the draft's body with a copy of Matt Pocock's `grilling` skill (github.com/mattpocock/skills, skills/productivity/grilling/SKILL.md at commit 95249b0b, MIT, Copyright (c) 2026 Matt Pocock), adapted: (a) first question "Is this an epic or a ticket?"; (b) one or two questions at a time, not whole-frontier rounds; the skill keeps the open questions written on the ticket and says with each question how many are open now (the count can grow); (c) bridle tickets are the record: read the ticket and what it names; each answer goes on the thread quoted (`bridle ticket comment`), the settled result into the ticket body; for a child ticket read the parent epic first and turn each mismatch into a question; (d) never readies, plans or builds; ends on "shared understanding?" and the human's yes. Keep Matt's core: design tree, facts looked up vs decisions asked, a recommendation on each question worded so "yes" accepts it.
Attribution: the MIT notice in the skill file (or a LICENSE beside it) plus a credit line with the source URL and commit. Use the name that fits the existing skills (bridle-manager, bridle-worker): bridle-grill unless the directory convention says otherwise. Fetch the upstream file with gh or curl; if it cannot be fetched, stop and ask (rule missing-tools), do not rewrite from memory.
Install and docs: the skill is installed for agents like the existing ones; a short note where skills are listed (docs/README.md or the workflow docs, whichever lists them); CHANGELOG. Skills must stay ASCII (rule ascii-in-editable-text): if upstream has non-ASCII characters, replace them and say so.
Acceptance: just check passes; ticket check clean. Migration: a new file under workflow/base/ reaches projects with the workflow update; no project migration. Out of scope: wiring it into PdM or designer roles, the design-before-build gate (95mu), the OpenSpec pipeline.

### note · agent:wmtph · 2026-10-12T13:15:09.115Z
built: 6e1d5306 (branch bridle/wmtph)

### note · agent:wmtph · 2026-10-12T13:15:09.218Z
done: bridle-grill skill; just check exit 0, 1590 tests, merged main (up to date); 6e1d5306

### note · agent:manager-2 · 2026-10-12T13:15:16.027Z
integrated: 991a88179effb94008a57b42be3e3f0986e8aa4b (branch bridle/wmtph)

### note · agent:wmtph · 2026-10-12T13:15:22.417Z
Note: the check (exit 0, 1590 tests) ran on my commit before a final merge of main; that merge brought unrelated main changes (6e1d5306 is the merge tip). Re-run the check at landing if you need it on the exact tip.
