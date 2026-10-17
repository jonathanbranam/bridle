+++
id = "br-tj7q"
title = "Change stages on every ticket: attachments, proposal/design/specs/tasks files, per-ticket stage reviews, specs become bridle specs"
kind = "epic"
state = "pending"
created_at = "2026-10-17T02:23:44.883Z"
updated_at = "2026-10-17T11:28:11.641581Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
priority = "high"
priority_at = "2026-10-17T11:28:11.641581Z"
ticket = "tj7q"
+++

docs/tickets/open/change-stages-on-every-ticket-attachments-proposal-design-sp-tj7q.md

## Thread

### note · external:advisor/product-manager · 2026-10-17T02:25:23.652Z
The human, 2026-10-16 ~10:35 PM ET, to advisor (product-manager): "Yes" to the designer pass. Designer writes '## Design options' into this ticket (the four questions under 'First step', plus the stage list per ticket and which agent writes which stage), for the human's review. 10-minute hold first (the human's rule). No planning or build until the human approves the design.

### note · external:advisor/product-manager · 2026-10-17T02:25:29.262Z
Correction: the human's "Yes" was at ~10:25 PM ET, not 10:35. Hold ends ~10:35 PM ET.

### note · external:advisor/product-manager · 2026-10-17T02:26:04.135Z
The human, ~10:30 PM ET: "I want UI as part of this. I want attachments as tabs or document picker inside a ticket." Added to the ask (item 6, Done when). Designer: include the bridle-ui ticket page.

### note · external:advisor/product-manager · 2026-10-17T02:27:18.810Z
The human: ui-fwsx (one ticket page: task and ticket combined) is part of this epic: "Put that in this epic as well. It's several clicks every time now". Added as item 7; designer, include it.

### note · external:advisor/product-manager · 2026-10-17T02:48:11.728Z
advisor (product-manager): design presented to the human 10:44 PM ET; the human went to sleep before answering. Waiting on the human: the design's three open questions (1 default stages by kind, 2 who writes the proposal, 3 un-approve on edit) and approval of the recommended A+A+A+A. No planning until then.

### note · external:advisor/product-manager · 2026-10-17T02:49:18.315Z
The human, 2026-10-16 ~10:50 PM ET, to advisor (product-manager), on open question 1 (default stages by kind: all four for feature/arch-revision/epic, none for fix/docs/chore): "Sure. It doesn't matter that much we can change it later." Q1 settled; Q2 and Q3 and the overall A+A+A+A still open.

### note · external:advisor/product-manager · 2026-10-17T02:50:47.813Z
The human, ~10:52 PM ET, on open question 2 (who writes the proposal): "PM writes proposal". So: proposal = project manager; design, specs = designer; tasks = project manager. (Advisor had recommended the designer; the human chose the PM.)

### note · external:advisor/product-manager · 2026-10-17T02:51:30.765Z
The human, ~10:55 PM ET: "Pretty much a ticket should have a proposal worked out with human. A ticket without a proposal isn't a ticket ready to be worked". So: the proposal is written by the PM together with the human (not alone), and 'bridle ticket ready' requires an agreed proposal. Open: whether this applies to fix/docs/chore (Q1 said no stages for those) or the ask serves as their proposal. Q3 (un-approve on edit) not yet answered.

### note · external:advisor/product-manager · 2026-10-17T02:53:33.292Z
The human, ~10:58 PM ET: "The proposal can live in the ticket yes but it has to follow the template. Everyone of these must have a fixed template and then instructions on how to write them. Whatever agent does the work gets that template and instructions. And those should be built from bridle rules. This is how that system gets to an agent". So: (a) each stage (proposal, design, specs, tasks) has a fixed template and writing instructions; (b) the proposal may live in the ticket body (small kinds) but follows the proposal template; (c) templates and instructions are built from bridle rules (workflow/), and the agent writing a stage is given them; this is the path by which rules reach agents (see br-86c6, the workflow doesn't reach agents).

### note · external:advisor/product-manager · 2026-10-17T03:03:16.592Z
The human, ~11:00 PM ET: "Yes" to a second designer pass tonight. Revise '## Design options' for the answers on the thread (Q1 default stages by kind: yes; Q2 PM writes the proposal, with the human; proposal agreed before ready; proposal may live in the ticket body in the template) and ask items 8-9: a fixed template + writing instructions per stage, built from bridle rules in workflow/, given to the agent that writes the stage (relate to br-86c6). Q3 (un-approve on edit) stays open for the morning review. Design only; no planning or build.

### note · external:orchestrator@dalek · 2026-10-17T03:03:30.311Z
From orchestrator: the human via advisor (product-manager), 2026-10-16 ~11:03 PM ET: "Yes" to a second design pass on br-tj7q tonight. Revise '## Design options' for the human's answers on the thread, and add items 8-9 from the thread: stage templates come from bridle rules, and the proposal is agreed with the human before ready. Design only; ticket file only. Lower priority than j86p and the build queue. Note on the thread and tell me when done.

### note · agent:designer-1 · 2026-10-17T03:16:02.289Z
designer-1: '## Design options' revised (ce38c6ca): Q1 and Q2 settled per the thread (PM writes proposal; defaults by kind); new Q5 (proposal agreed before ready: A for stage kinds, template-shaped body for small kinds) and Q6 (one rule per stage holding template + instructions, delivered by the existing rules path; ticket check verifies headings). Open for the human: Q3 un-approve on edit, small kinds' proposal, rule delivery in prompt vs on demand.

### note · external:advisor/product-manager · 2026-10-17T11:28:11.641Z
priority: normal -> high
