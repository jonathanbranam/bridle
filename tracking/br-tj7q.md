+++
id = "br-tj7q"
title = "Change stages on every ticket: attachments, proposal/design/specs/tasks files, per-ticket stage reviews, specs become bridle specs"
kind = "epic"
state = "pending"
created_at = "2026-10-17T02:23:44.883Z"
updated_at = "2026-10-17T21:36:23.739194Z"
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

### note · external:advisor/tj7q · 2026-10-17T19:29:21.704Z
The human, 2026-10-17 ~3:25 PM ET, to advisor (tj7q), on open question 6 (who starts the background writers), verbatim:

> Okay, I'm not entirely sure, so let's talk through the flow here. I feel like there's a question that I'm not fully thinking through, and maybe a problem here. I had said at one point an interactive agent might be working on this, but the answer to this question is going to impact how that works and whether it's possible for that to happen or not.
>
> We could say the first path is assuming a background agent does it, or the PDM does it directly. I think we just want to have a decision about how this works and to support some flexibility here. Let's run down two paths.
>
> Let's say one path is that a human is not involved: no approval gates, or just no human interaction. Even if there is an approval gate, no human interaction. I don't think the product manager does anything. Bridle should do this. There's no agent involved. The whole point of this, to a large extent, is that the agent shouldn't be involved.
>
> If a ticket exists and it's ready to move forward, then the system, Bridle, should spin up a background agent, some sort of worker agent. I don't know if it's the same worker. It's probably not the same worker as what we have now. Maybe a designer or something like that.
>
> Anyway, it spins up an agent, and that agent gets instructions. It gets the following instructions:
> - Here's the ticket.
> - It's ready for stage blank, stage design.
> - Here are the instructions for writing the design stage.
> - Here's the template.
>
> Then the agent writes the design, puts it in there, and marks it done. Bridle will see that that's done and then figure out what to do next. Is a human approval needed? If so, send it to the human. Assign it to the human, right? The ticket should then be assigned to the human, I believe, for approval, like claimed by a human. When the human approves it, however they do that, it just kicks back to the system.
>
> Let's just make it simple. Let's do MVP. Let's just decide right now for the MVP: all the artifacts, all the stages are written by a background agent. That's just easy. That's MVP. We can work on the other approaches later.
>
> I think I agree there. The only possible exception is the proposal. Whatever agent is filing the ticket is allowed to immediately write the proposal. I think what this means, though, is that maybe it's easier than I was thinking. If a ticket is in the "needs a proposal" stage, whatever that means, wherever we delineate that, then it's either claimed or unclaimed. If it's claimed, then whatever agent that is is saying they're going to write the proposal, and it should just wait for that to happen.
>
> I think what we're saying is that an unclaimed ticket waiting for a proposal should go. The bridle should notice that transition and immediately send it to a background worker to get done.

advisor (tj7q): settled as stated; design Q3-A (daemon notifies the PM) is overturned in favour of the rejected Q3-B (bridle spawns the writer). Body updated; follow-up questions added (which role writes, how a stage is marked done, the filer's claim, revisions after the human asks for changes).

### note · external:advisor/tj7q · 2026-10-17T19:37:58.839Z
The human, 2026-10-17 ~3:45 PM ET, to advisor (tj7q), on open question 6 (which agent writes the stages), after naming options (preparer, shaper, tech-lead, refiner, analyst, change-author, designer), verbatim:

> Yeah, I think that's good because that's originally what I thought of as the designer: to write the tickets. Later, we'll have a UI designer. That's okay. That'll be separate, and I'm okay adding UI to the front of designer. It is very short and understandable, so I'm good with that.

Settled: the existing `designer` role (workflow/base/roles/designer.md) writes every stage in MVP; its role file widens from "proposal into the ticket" to all four stages, the stage rule supplying each stage's specifics; it still builds nothing. A future UI designer role will be `ui-designer`.

### note · external:advisor/tj7q · 2026-10-17T19:41:09.608Z
The human, 2026-10-17 ~3:55 PM ET, to advisor (tj7q), on open question 7 (marking a stage done), verbatim:

> I think there is a situation that is probably not important to capture in the MVP. If an agent feels that it cannot complete the stage, they should build a market as something other than done, incomplete, or failed, or something with some sort of reasoning behind it. The reasoning could be in a thread or a comment on the ticket.
>
> I think they should be able to write "needs more input. They should be able to do something like that and kick it out or send it back. That should be an explicit part of the instructions. If you encounter something that is just not solvable, is a problem for some reason, or the proposal as written is not something we can implement, or the implementation is very complex, the agent should be able to kick it back for probably a human review. That can definitely come later, but it's something to think about.
>
> Just send it straight back to the PDM if the agent doesn't finish it. No reason to try again. We'll have somebody investigate what happened.
>
> Something else: we need to have timers on this. Bridle needs to observe if that agent is done, marks it done, or is moving on to something else. This is somehow probably the easiest thing right now: just a timer. If it's not been done for an hour or 30 minutes, then set a message to the PDM or the orchestrator (I don't know which PDM, I guess), saying that that agent didn't work on it. What should we do? The PDM can decide. The PDM should be able to then have the authority to re-ready the ticket for that stage, whatever that means, so that it gets picked up again by Bridle.

Settled for MVP: `bridle ticket stage-done <id> <stage>`; an agent that ends without it, or a stage not done within a timeout, goes to the product manager (PdM) with no retry; the PdM can put the stage back for bridle to pick up again. Later (not MVP): the writer can mark a stage blocked / needs input with a reason on the thread, sending it back for a human review; this goes in the stage instructions.

### note · external:advisor/tj7q · 2026-10-17T19:47:25.682Z
The human, 2026-10-17 ~4:05 PM ET, on open question 2 (60-minute timer, one setting; not done or writer ended -> one message to the product manager; `bridle ticket restage <id> <stage>` puts it back): "Yes". Settled. (Placeholder ids are now br-oooo; br-xxxx trips the link hook, ticket 4mbd.)

### note · external:advisor/tj7q · 2026-10-17T19:50:17.962Z
The human, 2026-10-17 ~4:10 PM ET, on open question 3 (the filer's claim: `bridle ticket new --claim`, or a body that already holds the proposal, keeps the proposal with the filer; otherwise bridle starts a designer on it at once): "Yes on Q2 claim question". Settled.

### note · external:advisor/tj7q · 2026-10-17T19:50:41.012Z
The human, 2026-10-17 ~4:12 PM ET, on open question 4 (changes asked by the human: comment on the thread or the file and say "changes"; the stage goes back to unclaimed, bridle starts a fresh designer whose prompt adds the human's comments; it comes back for review): "Yes." Settled.

### note · external:advisor/tj7q · 2026-10-17T19:51:05.738Z
The human, 2026-10-17 ~4:14 PM ET, on open question 5 (`bridle ticket next <id>` prints the next unblocked stage, its layered instructions, template, output path and dependency files; bridle uses the same output as the designer's prompt; refuses on an unsigned gate): "Yes". Settled.

### note · external:advisor/tj7q · 2026-10-17T19:51:32.207Z
The human, 2026-10-17 ~4:16 PM ET, on open question 6 (signing in MVP: a thread entry with signer, who ran it and time, plus the ticket's signed list; while a gate is unsigned later stages do not start, `ticket next` refuses and the ticket cannot become ready; cryptographic signing stays with v2va): "Yes". Settled.

### note · external:advisor/tj7q · 2026-10-17T19:53:49.849Z
The human, 2026-10-17 ~4:20 PM ET, on open question 7 (specs.md is a draft in bridle spec syntax; the tasks template always starts with merging it into design/specs and running `bridle spec id` / `bridle spec check`; no new command), verbatim:

> Yes. For post-MVP if the final specs differ from the original specs, and the spec had a human or agent review gate, then the new specs should be sent back to the human or agent.
>
> That is far future concern but important to follow up. TBD if that is only an inform or an approval. BTW we should use RACI terms for these as well. Helpful to be clear

Settled for MVP. Recorded as later: spec drift re-review (inform or approval, TBD). RACI terms (Responsible, Accountable, Consulted, Informed) to be used for who does what at a stage and gate: added to the names question.

### note · external:advisor/tj7q · 2026-10-17T19:57:52.249Z
The human, 2026-10-17 ~4:24 PM ET, on open question 8 (one worker builds the whole ticket, following tasks.md as its checklist, like OpenSpec's apply; no child rows), verbatim:

> Yes, that's correct. We should liberally steal and copy the OpenSpec apply skill, or anything else from OpenSpec, for how this works. It doesn't have to be in the MVP, but in the future, there are rules for the apply step as well.
>
> OpenSpec doesn't really do this, and it kind of depends on how detailed those tasks are and how necessary it is. I think having rules injected right then is really helpful for certain very tight programming things where we want the code written in a specific way. That should be an expectation.

Settled for MVP. Later: copy OpenSpec's apply skill freely; the apply step (the worker building from tasks.md) gets its own layered rules injected at the moment of work, like the stages, for code that must be written a specific way.

### note · external:advisor/tj7q · 2026-10-17T20:02:04.255Z
The human, 2026-10-17 ~4:28 PM ET, on open question 9 (existing tickets: the workflow applies only to new tickets), verbatim:

> Yes, but then how do we do this? Are we saying that tickets are valid if they don't have stages, or are we making a cutoff to say that tickets created before a specific time are valid if they don't have stages? Maybe we should add a version tag to a ticket, and we could say version 1 tickets don't have stages. We could assume version 1, and then version 2 tickets require stages. That way, we can say new tickets must be version 2, but then the old tickets don't have to migrate.

advisor (tj7q), fact: no ticket has any version-like frontmatter field today. Proposed back: `format: 2` (absent = 1). Waiting on the human for the field name.

### note · external:advisor/tj7q · 2026-10-17T20:48:12.531Z
The human, 2026-10-17 ~4:32 PM ET, on open question 9 revised (advisor proposed `format: 2`): "Why format not version? I prefer version it's very common". Settled: frontmatter `version: 2`; absent = 1. `ticket new` writes 2; `ticket check` requires stages (at least the proposal section) on 2 and accepts 1 as is; old tickets never migrate; opting one in = set version 2 and its stages.

### note · external:advisor/tj7q · 2026-10-17T20:49:23.728Z
The human, 2026-10-17 ~4:36 PM ET, on open question 10 (names with RACI: Responsible writes, Accountable signs the gate, Consulted/Informed later; schema `responsible:`/`accountable:` instead of `review:`): "Every watcher is informed on every stage change". So Informed = the ticket's watchers (`bridle ticket watch`), told on every stage change (written, waiting for signature, signed, sent back, timed out). Rest of the RACI proposal awaiting the human.

### note · external:advisor/tj7q · 2026-10-17T20:51:43.182Z
The human, 2026-10-17 ~4:38 PM ET: "Oh let's not use RACI terms in the schema. That is too extra". Settled: the schema keeps `review: human` (as approved at 2:55 PM); RACI words are for explaining roles in docs and on tickets, not field names. Names: stage, file, gate, schema. Informed = watchers, on every stage change.

### note · external:advisor/tj7q · 2026-10-17T21:36:23.739Z
The human, 2026-10-17 ~4:45 PM ET, on open question 1 (epic split), verbatim:

> For V1, we should include that the ticket is like comments from the human get forwarded to a particular agent or agent role, a principal, I guess, whatever it's called. I should have the ability.
>
> Think about a PR: it should basically work like a PR review. I can comment on different sections of the ticket, ask for changes, and edit it, obviously, if I want to, from disk. I may just highlight and comment on it, and then I may add overall changes as well. I can say I should be able to say, "Approve with this feedback," or "I need another review." Those are two options.
>
> If I approve it with no comments, it just moves on. If I approve it with comments, those should go to the agent to fix. Which agent is what I'm trying to think through. I think it's the same agent, or the same role with the same instructions, that wrote it the first time, but the instructions are different, right?
>
> The agent needs to be told if it's a fresh agent with a fresh context. It needs to be told: "These are the rules that were used to write this ticket initially, and this is the ticket that was written. Now the human has added these comments. Please address them." That's the same in both situations: if I rejected it with comments, or if I accepted it with comments.
>
> The thing is, this is going to happen where I just want to change these three things, and then it's fine. I don't want to see it again, and I don't need to see it again. I just trust that the agent will do what I asked. There'll be times where I'm like, "No, this is all wrong. I want a bunch of changes, and I want to see it again when you're done." This is v1 or v2 stuff, I guess.
>
> The first one is just reject, approve, and reject goes back to the agent. The reason I got started on all this is that, for v1, I should be able to use comments. If v1 may just be accept, reject, or approve, that's fine. v2 could be accept with comments. If that's harder, I don't think it's harder, maybe, though. Anyway, it's just not that urgent of a thing.

advisor (tj7q): folded into the split proposal: V1 = review like a PR (highlight comments on sections plus overall comments; approve, or request changes; comments go to a fresh writer of the same role, whose prompt is the stage's original rules and template, the file as written, and the human's comments, "please address them"; it comes back for review). V2 = "approve with feedback" (the writer fixes, it moves on without a second review).
