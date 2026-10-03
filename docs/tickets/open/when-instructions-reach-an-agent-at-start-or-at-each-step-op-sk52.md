---
id: sk52
title: "When instructions reach an agent: at start, or at each step (OpenSpec-style phases)"
kind: question
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [vp9e, 34bw]
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
