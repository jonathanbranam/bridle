---
id: br-fve5
title: "Standing self-review: keep docs, architecture, tickets and postmortems true to the code without the human starting it"
kind: feature
created: 2026-11-21
created_by: external:advisor/agentic-dev
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-9x55, br-7pjv, br-feav]
---

From the human, 2026-11-21, to advisor agentic-dev (verbatim): "We need to pay attention to the incidents that we've hit as well. I want those learnings to be folded in somewhere, as well as what you're discovering here, which is that there's not a good self-review happening of documents, architecture, and tickets. These things are stale and out of date, and it is a significant problem that should be addressed in any new system or in this system."

## The problem (evidence from bridle main, 2026-11-21)

- 20 design docs carry "Status (checked 2026-10-03)" and none has been re-checked; docs/proposal/build-order.md, which CLAUDE.md calls "what's next", was last edited 10-03.
- One outside check today found ~26 doc drift items (br-9x55, br-feav) and 7 built-but-open tickets (br-7pjv). `bridle ticket check` flags 135 of 350 files in docs/tickets/open/ as integrated but still open, and exits 0. Older tickets with no row (hrcn, hw6c, q7rx, 22ab) are invisible to it.
- Postmortems are written and never closed: j7r4 is open though all its fixes landed; mvtz, q7mv, v3b7, ytqu, zcqv are pending; docs/context/incidents.md has no entry since 10-18.
- What exists catches none of this: rule docs-current works per change only; `ticket check` warns but nothing runs it or blocks on it; re-evaluate tasks (reevaluate.rs) fire only on arch-revision with a goals/architecture tree, which no project has; the document-reviewer role is a human-led comment loop on one document.
- Every drift fix so far was a one-off audit started by the human.

Full findings: the "Staleness and self-review" and "Incidents" sections sent to the notes project's agentic-dev advisor (messages o-0647, o-0648), and notes project/agentic-dev/Lessons.md.

## The ask

A standing self-review that keeps documents, architecture and tickets true to the code without the human starting it. At least:

1. Docs: re-verify each design doc's Status line and concrete claims (commands, routes, config keys, states) against the code on a schedule, and file the drift as a docs ticket.
2. Tickets: find open tickets whose work is built (and integrated-but-open files, rows with no file, files with no row), and resolve or comment on them.
3. Incidents: check each open postmortem against its fixes and close it, and keep the incident record current.
4. Make the existing check fail loudly: `bridle ticket check` warnings that go unread for weeks are not a check.

Open design questions for the human: a role (an auditor agent on a schedule), a scheduled check that files tickets, or both; how often; what budget; whether findings go straight to the PM's queue or to the human first.
