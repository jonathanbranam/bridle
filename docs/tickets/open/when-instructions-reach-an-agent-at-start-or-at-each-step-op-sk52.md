---
id: sk52
title: "When instructions reach an agent: at start, or at each step (OpenSpec-style phases)"
kind: question
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [vp9e, 34bw, 7r2c]
tasks: [br-b8ff]
---

## The ask


The human, verbatim (2026-10-02, via advisor workflow, by voice), continuing
[[are-roles-and-rules-the-same-thing-one-layered-kind-of-promp-vp9e|vp9e]]:

> Okay, so you pointed out something really good here, and that there is there is a difference.
> A, a role has models, prompts, or not not not text things that are not text. Um, but I think we
> just covered that. Um, a project can specify different hooks to modify that as well. I think we
> said that right. So it's um, so some of that configuration could come from a rule, um, like
> don't write to a certain folder. That's a rule. Uh, so is that really different than a role? I
> think conceptually we think of them differently. So I definitely understand that. Like
> conceptually, it feels different, but mechanically, maybe we can solve it with the same
> tooling. But there's one, there's one rub, there's one problem with this that I was kind of
> putting off bringing up, but I think needs to be brought up. A role. Um, th there's a. Sorry,
> let me back up for a second. There are there are different things like procedures that kind of
> make sense, like a the. workflow that a role follows, like a specific thing, like how how how to
> do this, how to do that. Those procedures are somewhat different than rules that say, you know,
> do this, do that, don't do this, don't do that. Um, but maybe we can handle them all with the
> same solution, is what I was going to say. But the, the rub here, the thing that we haven't
> really delved into, is that, from what I understand, we're lumping all of this together. and
> giving it to the agent at startup. And that's not necessarily what I want. Um, but I'm not sure
> how to, the best way to do this is. But the, I think I've said rules a lot, and I've been
> thinking of open spec rules. But open spec rules, have nothing to do with a role. They are
> literally inserted into the prompt of whatever agent is running. And they depend at, at the time
> the agent is asked to do something. So that's the thing here. Like an open spec um, prompt when
> you say It's time to write the proposal for this ticket. There's a couple things that happen,
> but OpenSpec reads the rules for writing a proposal. It has its own set of instructions that it
> gives for how to do that, that um, that are baked in. You, you can't, well, you can override
> them, actually. You can override them, um, but you have to replace them wholesale, the
> instructions. So it has a set of instructions, and then it has a set of rules which are
> independent and they are append only, although I think that's uh, debatable whether they should
> be for our system or not. I like being able to remove them in some cases. Um, and then they have
> a template because what OpenSpec does in each of its phases is create a document or a set of
> documents, a proposal, a design, a set of specs, and then a set of tasks, and then there's a
> separate thing, which is the implement uh, operation, which is handled somewhat differently. And
> originally, OpenSpec, you could not specify anything during implementation. That's the, it's
> called the apply step. But now, with the latest um, one, you can also have, they call it
> guidance. during the apply step, and there is also guidance that you can apply during the
> archive step when a change is finished. I'm not exactly sure why they called that guidance
> instead of rules. Um, I think mechanically, it's probably the same thing. So um, I don't know
> why they call it something different. I haven't looked at the code yet, so I'm not sure exactly
> how it works. But this is the big thing I'm getting at here. We've got system prompt text that
> comes up at the beginning, and then we've got prompt text that comes when an agent is asked to
> do something. And I think there's a little bit of magic in the way that OpenSpec does it that I
> think would be interesting for us. The text that comes in instructs the agent on what to do
> next. And you know, I don't. We don't start a new agent when we're writing the proposal, and
> then the design, and then the specs. We typically do it all in one session. Now you can you can
> start a new agent for every step, but it's it seems kind of unnecessary. Um, but uh, yeah,
> that's sort of the model I have in my mind. Um, but I'm not sure if that's. what we want here
> or not, like um, our workers are pretty much implementers. Uh, although actually that's
> probably not true. I, I haven't really looked closely at the detail they've been given to
> implement. I think they make a lot of decisions themselves. So that's something maybe to, to
> look into. But in OpenSpec, again, we've got, we write the proposal. That should be pretty
> similar to our tickets. And then we write, I typically write a design and um, go through the
> design questions and check them off. Um, in our workflow, I typically do all of that together
> with the agent. So first of all, that's something that's like completely missing because we've,
> I do that with the advisor or sometimes the orchestrator or whomever. And they don't have those
> prompts inserted into their thing. They haven't been given instructions on how to write a
> proposal or how to um, write a design. You know, they're not following a specific workflow at
> all. They don't have a template they're filling out. They're just writing words into a document
> based on their, you know, internal knowledge. So that's a, a, an issue that I've punted a few
> times. Um, And then, uh, you know, after the ticket's written, we do have the product manager
> reviewing it. Um, I think that's a good step. That's, you know, o- OpenSpec is a very linear
> system without agents operating independently. So um, I think that's a good system we have
> there. But um, yeah, I'm not sure how to handle that. Like, uh, exactly. You know, you don't
> need a separate. session for necessarily like each different thing. Um, you can do it with a
> prompt or a skill, which is, which is basically the same thing as a prompt. So um, yeah, I'm not
> sure um, exactly how to do this work here. Like, but I, I think there's something happening in
> our system that's not exactly the way I wanted it designed. Um, and uh, we need to think through
> that. Like, does it is is it fine to have the prompt contain everything at the beginning? I
> think I'm okay with it for now. Um, but I do want to, like, think about how how we work with
> agents and. how I work with the interactive ones, and then how the automatic agents in the
> background do their thing. Like, if I'm working on a ticket, um, should we have that those
> steps in there, right? Like, who, who writes the specs? Like, we have bridal specs, but who
> writes those specs? Um, I've brought this up before. I think the worker writes the specs. And so
> that means I never reviewed them. Um, but that's kind of an important thing for the human to
> take a look at. Um, I don't think it's a, something we need to get worked up about right now,
> but it's, uh, it's an interesting thing.

**Not sent to the orchestrator.** Discussion only. The human: everything-at-start is "okay with
it for now"; 34bw steps 1-3 go ahead as approved.

## What's in the repo already (checked 2026-10-02)

- **The design planned step-time delivery, as skills.** `docs/design/skills.md` replaces
  OpenSpec's ~10 skills with six: `bridle-manager`, `bridle-worker`, `bridle-plan` (the plan,
  spec edits, impact), `bridle-review`, `bridle-triage`, `bridle-conflict`. Only `manager` and
  `worker` exist (`workflow/base/skills/`), and the review (5u9d) found spawned agents can't use
  them (no `Skill` tool for the manager; `.claude/skills/` not in worktrees). There's no skill for
  writing a ticket, a design or specs, and no templates.
- **Interactive sessions follow no procedure for those documents.** The advisor and orchestrator
  write tickets by `docs/README.md`'s conventions and `bridle ticket new`'s frontmatter, nothing
  more: no template for the body, no design checklist.
- **No role says who writes specs.** `worker.md`, `manager.md` and `product-manager.md` don't
  mention spec authorship. `docs/design/spec-flow.md` covers importing and checking specs, not who
  writes or reviews them.

## The advisor's thinking (not decided)

1. **Two moments, one kind of text.** Text at start (the role: identity, standing rules) and text
   at a step (how to do this step, its rules, its template). OpenSpec's instructions, rules,
   templates and guidance all fit the second. Claude Code's skills are exactly that mechanism: the
   description is in the prompt from the start, the body loads when the step begins, in the same
   session. That matches the human's "you can do it with a prompt or a skill".
2. **Rules scoped by step, not only by role.** vp9e's unified rules get a second tag beside
   `roles:`, e.g. `steps: [ticket, design, specs, plan, implement, review]`. A rule tagged for a
   role goes in at start; one tagged for a step goes in when that step's skill runs (the skill
   calls `bridle prime --step design`, or sync renders the resolved rules into it). Same layering,
   same overrides (including `disable`, which the human wants and OpenSpec lacks).
3. **Per step: instructions (replaceable), rules (layered), a template.** The template is a file
   in the layer (`templates/design.md`), overridable like everything else.
4. **Same skills for interactive and background agents.** The human writing a ticket with the
   advisor runs the ticket step; a background planner runs the same one. This fixes "they're not
   following a specific workflow at all" for interactive sessions.
5. **Who writes specs** belongs with the gates (`docs/design/gates.md`): if spec changes are part
   of the plan, the plan gate is where the human sees them. Separate question; the human: "not
   something we need to get worked up about right now".
6. **How much workers decide** is measurable: compare a sample of task briefs with their diffs
   and summaries. Worth doing before designing the steps, since it says where the design work
   actually happens today.

Depends on vp9e (one kind of layered text) and 34bw step 1.

## More from the human (2026-10-02)

> So I'm not terribly worried about what the workers are doing today. I think we, I think we have
> an idea. […] Yeah, and I agree they should be usable by anyone. OpenSpec uses skills, of course.
> The the thing that it does that's unique is when you invoke a skill, it reads the files in the
> repo, it reads the instructions, and it reads the template, and it reads um, the rules and
> layers those things together. and then provides it in a structured prompt. It also tells the
> agent, you know, um, it also defines a workflow, which steps come before which other steps, and
> then it provides context. So um, I think those are all things that we need.

So point 6 above (measure how much workers decide) is dropped. The product-manager naming point
from the same message is [[the-product-manager-role-is-really-a-project-manager-who-hel-7r2c|7r2c]].

What a step needs, per the human (all of it):

| Piece | What it is | Layered? |
|---|---|---|
| Instructions | how to do this step | yes (OpenSpec: replace only) |
| Rules | constraints for this step | yes, including `disable` |
| Template | the document the step produces | yes |
| Context | the repo files the step needs: the ticket, the specs it touches, the design | read at invocation |
| Workflow | which steps come before which (ticket → design → specs → tasks → implement → archive), and which are done | defined in the layers |

The skill is thin: it runs one bridle command (e.g. `bridle step design <ticket>`) that reads and
layers all five, and prints them as one structured prompt. The resolution engine already exists
(rules); this adds templates, step ordering and context gathering to it.

## Workflows per ticket, and who does which step (2026-10-02)

The human, verbatim, on the step command above: "I agree on like the bridal step design thing in
principle. Don't don't build it yet. But let's write that up in the ticket". Then:

> Something else to note here is that there's open spec, they call this a schema, but I prefer the
> term workflow. Every change in open spec can have a different schema. There's generally a
> default schema called spec driven, but you can create a change with a different schema, a
> different workflow. And I think that's super valuable. And there's a couple ways to do it, but
> basically you can customize that workflow and you can add and remove steps. And you can, you can
> kind of add and remove what context is applied. And when you do that, when you define your own
> steps, and you can change the instructions. So I still like our layering system better, but you
> can do that and you can then you can also with that change or separately from that, you can
> change the templates. So I think all that's really good. I had a better different point though.
> Oh, so if you're in a particular workflow or schema, you have to go through the steps that are
> listed in the order they're listed. Although it's not necessarily linear, they have they support
> some branching. And so this is this is true whether the agent does all of the steps itself or
> whether the human walks through the steps. And I, I love that because sometimes, you know,
> there's basically four steps before you apply a change. Proposal, write the specs, write the
> design, then write the tasks. And I can stop, I can go forwards and, and backwards in that. You
> know, generally you go forwards, but you can, you can go back a little bit. You can change the
> proposal after it's, after you've written the design and then update everything. Or you can
> take a few steps, with the human and then you can say, pass the rest off to the agent to do, but
> all the steps are always done. And, I think that's, that's super valuable. And for our system, I
> really want to be able to do, have that kind of flexibility to say, you know, for this ticket,
> for this, for us, I think it's maybe a task or a ticket. I don't know. That whole thing is
> confusing, but call it a ticket. For this ticket, or whatever, this is the workflow we're using.
> So we're going to follow it. But I can say, hey, have the agent do steps A, B, and C. Or I can
> say, oh, let's do A together. OK, that's done. Let's do B together. OK, that's done. All right.
> Agent, you take over all the rest of these steps. Just go and just send it off. Or I could say,
> you know, if we have, I could say agent, do steps A and B, and then come back to me for review.
> Don't go on to C until I look at A and B. So I think that's super, super valuable. There's times
> when. I just don't care, and I want the agent to go do it all and ship it. But I do want the
> workflow followed. but then the flexibility to say, for this particular use case, the workflow
> is different. And this is particularly true for, you know, like document changes or, yeah, I
> don't know what else, but. You know, a feature versus a bug, or like a hotfix, or like a
> vulnerability fix, a security fix. What else do we do? You know, we do like, like package
> version upgrades, right? There's a bunch of things we do that the workflow should just be like,
> upgrade all the packages, run CI, and ship it. It's still a workflow, but it's, it's a very
> simple workflow. Or like a bug fix, a bug fix, probably doesn't contain any changes to the specs
> because the specs were right in the first place. There's just a bug. So we don't need to update
> any specs, but they should all pass and we should still like, you know, maybe design the
> solution, maybe. But if it's a simple bug fix, you know, just go fix it. Yeah, or but for a new
> feature, depending on how complex the feature is or how critical it is, I might say that I want
> to review the design first. But in other cases, I, I don't. I just want the agent to finish the
> whole thing. Now, we do have the ability, and, I, and we need to retain this for the agent to
> say, like, hey, I wrote the design, and I think it needs a human review. So that, that should
> still be like a, a possible thing. Yeah, and I think that covers it really. Like, there are
> cases that we've talked about in, in other sessions about wanting a fresh agent with a fresh
> context to do like an adversarial review or security review. Those are still true and still
> things we can consider later. I've been calling those roles, and I think that's, that's kind of
> accurate because we don't want the same agent to write the code and do the security review. The
> agent's just too biased. All the stuff is, all the decisions it made are in its memory, in its
> context, and it doesn't challenge itself enough.

**Don't build yet.** Design notes only.

### What the human wants, distilled

1. **Workflows are named and layered.** A workflow (OpenSpec's "schema") is an ordered set of
   steps, with some branching. Each step has instructions, rules, a template and context (the
   table above). A layer can add or remove steps, change context, instructions and templates,
   through bridle's layering, not OpenSpec's replace-wholesale.
2. **Each ticket picks its workflow,** usually by kind, overridable per ticket. Examples from the
   human:

   | Ticket | Workflow |
   |---|---|
   | new feature | ticket → specs → design → tasks → implement → review |
   | bug fix | maybe a design; no spec changes (the specs were right); every spec still passes |
   | package upgrades | upgrade, run CI, ship |
   | hotfix, security fix, docs change | their own short workflows |

3. **Every step is always done, in order.** Going back is allowed (edit the proposal after the
   design, then update what follows); skipping isn't. This holds whether the human or an agent
   does the step.
4. **Who does each step is chosen per ticket, and can change midway.** Some patterns:
   - The agent does everything and ships it.
   - Do A together, then B together, then the agent takes the rest.
   - The agent does A and B, then stops for the human's review before C.
5. **An agent can always ask for review:** "I wrote the design, and I think it needs a human
   review" stays possible in any workflow.
6. **Some steps need a fresh agent:** an adversarial or security review by a separate role with
   a clean context, never the agent that wrote the code. Later.

### How it might map onto bridle (advisor, not decided)

- A workflow is a file in the layers, e.g. `workflows/feature.toml`: steps, order, branches, and
  per step its instructions, template, context and rule tag (`steps:` on rules, above).
  Projects override by id like rules. Kinds pick defaults (`feature` → `feature`).
- The ticket records its workflow and each step's state (done, who did it, when). `bridle step
  <step> <ticket>` prints the layered prompt for the next step and refuses one whose earlier
  steps aren't done. Going back reopens the later steps.
- The handoff is a per-ticket field: `agent through: design`, then human review; or a gate on a
  step (`review: human`). This is where `docs/design/gates.md` (plan, merge, accept gates) folds
  in: a gate becomes "the human does or reviews this step".
- Review roles (adversarial, security) are steps whose `role` is a fresh agent: the daemon
  spawns it, so it can't be the implementer. Ticket `br-4034` (independent code and security
  review) is the existing question.
- Open: ticket vs task as the thing that carries the workflow (k7tm's question, `br-3724`). The
  human says "call it a ticket" for now.
