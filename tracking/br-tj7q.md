+++
id = "br-tj7q"
title = "Change stages on every ticket: attachments, proposal/design/specs/tasks files, per-ticket stage reviews, specs become bridle specs"
kind = "epic"
state = "pending"
created_at = "2026-10-17T02:23:44.883Z"
updated_at = "2026-10-17T15:28:57.597466Z"
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

### note · external:advisor/product-manager · 2026-10-17T12:22:23.631Z
The human, 2026-10-17 ~8:05 AM ET (to advisor product-manager), on Q3 'should editing an approved stage withdraw its approval?': "No".

### note · external:advisor/tj7q · 2026-10-17T15:28:57.597Z
The human, 2026-10-17 ~11:30 AM ET, to advisor (tj7q), verbatim:

> Okay, let's get started on this. I'm not sure exactly where it stands. I looked through the ticket. I need to read it in more depth, but I'll give you a basic overview.
>
> What we're trying to do is build the workflow. The main purpose of Bridle is that when we have work to do, there should be a workflow assigned to a ticket. That workflow governs how the ticket is prepared to be built, investigated, researched, planned, designed, and implemented. It does not deal with this: this workflow is not about scheduling or prioritization. It's about designing the ticket properly and getting approvals that are needed.
>
> We need to set a clear scope between epics, and we should give these a name. We should sequence the work, give it a name, and figure out which tasks, which tickets, and which functionality need to go in each section.
>
> In my view, we haven't delivered almost anything on this, so we need to get this moving. We haven't delivered. We've delivered almost nothing. For an MVP, my thinking is that we need something very simple and straightforward: a ticket list instead of steps or gates or whatever we want to call them. We're just going to copy OpenSpec more or less for now. We'll change this later, but don't worry about that. We'll just copy what OpenSpec does.
>
> Every ticket has to have a proposal, a design, specs, and tasks before it gets handed to a worker. Then, optionally, we indicate which of those need a human review. To begin with, the proposal always needs a human approval, and the other three are just optional. When we create a ticket, we'll set which ones need approval. If that's all, if we just ship that, we're ready.
>
> I think that our MVP is to create a ticket and specify the stages on the ticket. We'll just have a standard set, but we'll just list them on the ticket because every ticket could have different stages in the future: different gates, different steps, and different steps in the workflow.
>
> Each step in the workflow has a gate on it. For now, that gate is "artifact created." OpenSpec calls them artifacts. We call them files. I don't care. We just call them a file or an attachment, whatever. It's all the same thing.
>
> If it needs human approval, then we're going to have other types of approvals too in the future: other types of reviews, not just human. If it needs human approval, then an agent writes the file and says, "It's ready for your review." I review it and say "approved," or we fix it or we change it or whatever, and then I say "approved." It goes on, and everything's automated and done by agents unless there's a human review required.
>
> What else is important? There's a template for these files, these attachments, artifacts. All have a template, and that template comes with Bridle. Everything has instructions and then a template, right? Instructions are layered rules from Bridle, and the template is also from Bridle. All of this is given to the agent exactly when they're doing the work, not in their Claude.md.
>
> No. When we tell an agent it is time to write the proposal, we give it all the instructions for writing a proposal and the template right there, every single time. Same with all of the steps here. Which agent gets this is something we need to work out later, but we could just start up by saying a background agent starts up and does this, and it gets the instructions. That should be okay.
>
> Take all that down, clear it, read it, clarify, and fold that into the document as needed. Then bring back all the open questions for clarification so that we're sure that we have MVP and that the first epic is clear on the scope and requirements.

advisor (tj7q): folded into the body as '## Captured intent (2026-10-17)'. Note two shifts from the night's design: every ticket gets all four stages (Q1's "none for fix/docs/chore" is reopened), and instructions + template are given at the moment of writing, not in the role's prompt (the designer's Q6-A is overturned; this is Q6-B / OpenSpec's `openspec instructions`).
