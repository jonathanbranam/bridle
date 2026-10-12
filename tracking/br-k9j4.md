+++
id = "br-k9j4"
title = "A grilling skill: interview the human to clarify a ticket or an epic, and check children against their epic's design"
kind = "feature"
state = "built"
created_at = "2026-10-12T04:16:45.877Z"
updated_at = "2026-10-12T10:07:46.845080Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wk9j4",
]
branch = "bridle/wk9j4"
commit = "c1200e3b"
summary = """
Design pass plus draft skill, no wiring. ## Design added to the ticket file (one skill with ticket/epic modes; rounds by default, one-at-a-time on request; write our own text, credit the idea; grilling feeds 95mu's change spec and is not a gate). Draft at workflow/base/skills/grill/SKILL.md (follows the manager/worker SKILL.md layout; marked DRAFT). Docs: none needed beyond the ticket.
Questions for the human: (1) rounds by default OK? (2) own text, credit idea only, OK? (3) "multi-part tickets (attachments)": attached files, or tickets made of several parts? (4) which role runs it first, PdM or designer? (5) may the skill edit the ticket body, or only comment?
Fetch note: github raw README reachable; skill body not read (not needed)."""
ticket = "k9j4"
+++

docs/tickets/open/a-grilling-skill-interview-the-human-to-clarify-a-ticket-or-k9j4.md

## Thread

### note · agent:pm-1 · 2026-10-12T04:17:19.230Z
Brief (pm-1): a designer pass first, per the PdM; no build until the human reviews. Model: Sonnet (designer role). After kt25's design in the queue.

Do: (1) Write `## Design` in this ticket's file answering the four "Open for the design" points (one skill or two; round-based or one question at a time; adopt grilling's text with credit, or write our own; how it fits the design-before-build gate 95mu and multi-part tickets), each with options, the rejected ones and a recommendation. (2) Draft the skill itself as a file under workflow/base/skills/ (check the existing skills there for format and naming; if the directory has no precedent, say so in the Design and put the draft in the Design section instead of creating a new layout). It is a draft for review, marked as such; it is not wired into any role yet. Keep the draft short: bridle's rules are brief (kiss). The skill never readies or builds, and records answers in the ticket thread with the human's words quoted (talk-on-the-task).
If it adopts any text from mattpocock/skills (MIT), credit it in the file.
Acceptance: Design section and draft skill present, ASCII only, `bridle ticket check` clean for this file; the done note lists the questions for the human. Migration: a new file under workflow/base/ reaches projects with the workflow update; no project migration. Out of scope: wiring the skill into the PdM or designer roles, the design-before-build gate itself (95mu), the OpenSpec pipeline.

### note · agent:wk9j4 · 2026-10-12T10:07:46.814Z
built: c1200e3b (branch bridle/wk9j4)

### note · agent:wk9j4 · 2026-10-12T10:07:46.845Z
done: grilling skill design + draft (workflow/base/skills/grill/SKILL.md); just check exit 0, 1590 tests, ticket check clean; c1200e3b (main merged; last merge only added a docs ticket). Questions for the human are in the summary.
