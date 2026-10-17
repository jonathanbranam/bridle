+++
id = "br-tj7q"
title = "Change stages on every ticket: attachments, proposal/design/specs/tasks files, per-ticket stage reviews, specs become bridle specs"
kind = "epic"
state = "pending"
created_at = "2026-10-17T02:23:44.883Z"
updated_at = "2026-10-17T18:38:57.525013Z"
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

### note · external:advisor/tj7q · 2026-10-17T17:38:35.356Z
The human, 2026-10-17 ~12:15 PM ET, to advisor (tj7q), correcting advisor's "every ticket gets all four stages", verbatim:

> No, that's not what I said. I said we list the stages for each ticket on the ticket, and we list which stages require human review. That doesn't mean every ticket gets all four stages. It means that there's no rule. We don't need to make a rule for this ahead of time. When we create the ticket, we'll decide which stages it needs and which of those need a human approval.

advisor (tj7q): body corrected. No default-by-kind table and no all-four rule: the ticket's creator decides its stages and which need the human's approval. Open question 3 replaced.

### note · external:advisor/tj7q · 2026-10-17T17:45:29.030Z
The human, 2026-10-17 ~2:00 PM ET, to advisor (tj7q), verbatim:

> Yeah, I'm reading through some of your stuff here. I think that most of the time, the proposal should be written by the product manager. That means the skills, whatever we're doing with Bridle here, need to work in background or foreground, with agents, background or interactive agents. I don't know how that works. I don't know how to make that work, but that's the requirement we have.
>
> If I want to pull up an advisor and go through the ticket with an advisor live, then that should work, and it should work exactly the same as with the background agents. The way this works in OpenSpec is it's a skill. When the agent invokes the skill, the skill then compiles all the rules and then outputs the instructions, which include all the layered rules. Those layers should come from Bridle, and then it outputs the template as well, and the agent does the work.
>
> I don't care if it's a skill. What OpenSpec does, you should look at it. If you go up a couple levels in this folder, there's a summary, our analysis of how OpenSpec works. OpenSpec has a skill, and it says you need to create the next artifact. The way it works in OpenSpec is there's a schema, and the schema is our workflow. I think there's a corollary there, but every ticket gets its own schema. There are standard schemas, but you can assign those.
>
> Essentially, we're simplifying that for the moment and just saying the schema lives on the ticket. The schema tells you which artifacts to create in what order and which artifacts are dependent on other artifacts. For OpenSpec, there's actually a split here where the proposals come first, and then specs and design depend on the proposal. Approval that we're specifying on the ticket and metadata, and then, again, in the future, that gate could be other agents as well.
>
> We've talked all about this and how it's supposed to work somewhere, but essentially it should work the same as the human or the agent. The human, the gate, whoever owns the gate, will sign it as approved, and then the rest of the command can't go forward unless it's been signed.
>
> What else did you have in there that was wrong? There should definitely be a command for me to approve it, but I don't want to run a command. I do.

advisor (tj7q), facts looked up: OpenSpec (/Volumes/Data/work/bridle/OpenSpec) has schemas/spec-driven/schema.yaml: each artifact has id, generates, template, instruction and requires (proposal: none; specs: proposal; design: proposal; tasks: specs and design), plus apply (requires tasks, tracks tasks.md). Skill openspec-continue-change runs `openspec status --change <name> --json` (each artifact done/ready/blocked), picks the first ready one, runs `openspec instructions <artifact> --change <name> --json` (context, rules, template, instruction, output path, dependencies), writes ONE artifact and stops. Bridle's analysis: ../workflow/research/10 and 11. Signing: gates.md has no signing built; signed reviews are v2va. Body updated: who writes, the schema on the ticket, the next-artifact command for any agent, signed gates; open questions renumbered.

### note · external:advisor/tj7q · 2026-10-17T17:49:18.062Z
The human, 2026-10-17 ~2:30 PM ET, to advisor (tj7q), verbatim:

> I want to make something clear here: every ticket needs a properly written proposal, no matter what. A lot of our tickets explain the problem, just like they look like a bug report, and that's too wishy-washy. They list a bunch of problems, and then, I don't know, "fix it." That's not what I want at all.
>
> Any ticket that is going to be worked on needs a proposal at a minimum. I don't want to make rules, though. I don't want to make a bunch of rules right yet. I'm not quite ready to make all of these rules, but let's just start with that.
>
> When we create a ticket that's going to result in a change, we should be writing a proposal right then, or somebody needs to write a proposal. That should be on the ticket. I've opened up some of these tickets to see what was given to the agent to implement them. I don't know what the agent's going to do with it because they're just making guesses about what the actual problem is, how to approach it, what the scope of the change is, or something. They're just not clear enough.
>
> Call that what you want. I'm tempted to say that's not a proposal, but let me just say it's not a separate artifact. That's just how the ticket should be written. The ticket should be written using the proposal template, but a lot of our tickets are not necessarily changes. There is some mapping to tickets that are going to result in a change, but even a critical bug fix should have a properly written proposal for what the work is going to be.
>
> I'm still sort of not sure. Let's just make that the default for now. I think that's fine. Let's just say every ticket has a proposal, no matter what. That doesn't mean it needs a human review, but it just has to have a proposal. We said a minute ago that the proposal for a small ticket can just live inside of the body of the ticket. Again, I'm fine with that, but it needs to be a properly written proposal.
>
> We need to know where that starts and ends so that it can be validated, and it needs to follow the template. Again, I don't want to get too in the weeds on validating these templates at this point, but we need to build things so that whoever is writing this proposal gets the instructions. That's super important, and they get the template, and then they put it on there.

advisor (tj7q): body updated. Every ticket has a proposal (default, not yet a rule); it may live in the body, delimited so it can be validated later; the writer gets instructions + template. Open: this says the proposal "doesn't mean it needs a human review", while ~11:30 AM said "the proposal always needs a human approval" (new open question 3).

### note · external:advisor/tj7q · 2026-10-17T18:00:03.828Z
The human, 2026-10-17 ~2:40 PM ET, to advisor (tj7q), on open questions 3 (proposal review) and 3b (where the proposal lives), verbatim:

> Yeah, I think that's fine for the proposal delineation. Yes, proposal. The gate for proposal is the same as the others. Every ticket gets one, but it only needs a human review if we've added that. Agents can still file bugs in critical things, write a proper proposal, and get to work on them. If the orchestrator or somebody decides this is important, or if the product manager says this is important and needs to get fixed now and it's unambiguous, then that's fine.
>
> Yeah, I'm fine if the proposal lives in the ticket body. Let's just say that. I'm fine with the H2, but I'd like to add an HR before and after it. Three dashes is plenty, just to set it off clearly from the rest of the ticket.

Settled: the proposal's gate is like any other stage's (human review only when listed); the proposal lives in the ticket body under `## Proposal`, set off by a `---` line before and after.

### note · external:advisor/tj7q · 2026-10-17T18:02:08.275Z
The human, 2026-10-17 ~2:50 PM ET, to advisor (tj7q), on open question 2 (approving without typing a command), verbatim:

> Yes, that's right. I still want to trace on this, so I want to know where that came from, just so I can be like, "What happened?" I want to know that command can detect if it's being run from an agent. I believe we always know that.
>
> Just have the command detect if an agent ran it and include their principal, or whatever it's called, their name, and it should be timestamped, obviously. Then we should tell if I ran it from the command line or an agent ran it. The difference between whether I use the UI or the command line is unimportant, so that does not have to be differentiated.

advisor (tj7q), fact: yes, we always know. Every CLI call is authenticated as a principal (principals.md: an agent's token, or an external principal's from credentials.toml; the human token file is never read under $CLAUDECODE). So `ticket approve` records: the signer (the human), who ran it (the human directly, or the agent's principal, as "human via <agent>"), and the time. The UI and the CLI are both just "the human".

### note · external:advisor/tj7q · 2026-10-17T18:03:10.554Z
The human, 2026-10-17 ~2:55 PM ET, on open question 3 (schema shape: frontmatter `stages:` map, each stage with `requires` and `review`; `ticket new --stages ... --review ...` fills in OpenSpec dependencies; proposal always included): "Approve". Settled.

### note · external:advisor/tj7q · 2026-10-17T18:05:32.454Z
The human, 2026-10-17 ~3:05 PM ET, to advisor (tj7q), on open question 4 (how stages fit ticket states), verbatim:

> I'm not sure on this one. What does it mean for a ticket to be ready? This is something that's confusing and confounding me, because my first thought is that as soon as a ticket has a proper proposal, or as soon as the ticket has passed through these gates, it's ready. I don't know what that means.
>
> Does that mean it's prioritized and in a queue? That's a different question. Where this ticket fits in the priority and when it's actually going to be implemented is a different question than whether it's ready in my mind.
>
> Once it's been through these stages, if none of them require human review, the ticket should just plow through all the stages. Agents should fill out everything, and then the ticket's ready. The product manager, or someone similar, should decide when it gets implemented, which is a different question.

advisor (tj7q), fact: today `ticket ready` is pending -> open and means "approved for work" (mhjn decides who may give it); the PM then plans it, open -> planned, which puts it in the queue a worker claims from (storage.md, state branch). So today "ready" mixes approval with entry to the pipeline. Reworded open question 4 on the human's meaning: ready = prepared.

### note · external:advisor/tj7q · 2026-10-17T18:21:51.156Z
The human, 2026-10-17 ~3:10 PM ET, on open question 4 (ready = prepared: a new ticket is preparing, agents write each stage as it unblocks and stop only at a gate listed for the human; ready by itself when every stage is written and every gate signed; no separate ready approval; when it is built is the PM's call, outside this epic): "Approved". Settled. Note: this changes mhjn (who readies what): readiness is no longer a decision anyone makes; the states pending/open/planned map onto preparing/ready, to be designed.

### note · external:advisor/tj7q · 2026-10-17T18:38:57.525Z
The human, 2026-10-17 ~3:12 PM ET, on open question 5 (stage files in a folder beside the ticket on main, docs/tickets/open/<slug>-<id>/design.md, specs.md, tasks.md and other attachments; moves with the ticket on resolve): "Yes". Settled.
