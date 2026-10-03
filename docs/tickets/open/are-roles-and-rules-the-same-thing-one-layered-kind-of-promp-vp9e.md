---
id: vp9e
title: Are roles and rules the same thing? One layered kind of prompt text
kind: question
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [34bw, 5u9d, xebc]
tasks: [br-9889]
---

## The ask


The human, verbatim (2026-10-02, via advisor workflow), after the advisor proposed layering role
prompts (append by default, replace with a reason) in
[[the-workflow-doesn-t-reach-agents-resolved-rules-hooks-and-o-34bw|34bw]]:

> I'm good with the ideas here. I think the thing that's noodling around in my head still is
> whoa, how are roles and rules different? Why do we have a nice layering system on rules that
> get put into the system prompt but not roles? What's the difference between a rule and a role?
> Are, isn't all of this text that gets put into a prompt when the agent starts?

> I'm still not convinced here at all that roles and rules are different things. So please write
> this up, but don't send it to anyone. And if you have any further reasoning, just put it in the
> document, and we'll talk some more.

> […] I'd like the like there to be a layering system on, on pretty much everything.

**Not sent to the orchestrator.** Discussion only, until the human decides.

## Today

- **Rules**: `workflow/<layer>/rules/<id>.md`, one constraint each, with `id`, `severity` and
  `roles` tags. Layered by id (`override: replace|append|disable`, `locked`), explainable with
  `bridle rules explain|diff`. 20 base rules, 14 in packs; most are tagged for 4-6 roles, only
  two for one role. After 34bw step 1, a role's resolved rules go into its spawn prompt.
- **Role prompts**: `workflow/base/roles/<role>.md`, one document per role (2.6K-10.7K
  characters; 37K across six). Not layered: a project either uses the base file or points
  `system_prompt` at its own file, a wholesale fork. meta-notes has forked `worker.md` and
  `manager.md` and its copies have drifted from base (34bw). Only `prototyper` gets a project
  append, special-cased by name (`config.rs:2635`).
- **A role** is also config, not text: `[roles.<name>]` in `.bridle/config.toml` sets model,
  tools, allowed/disallowed tools, permission mode, budget, autostart, Stop hook. None of that is
  prompt text.

Why they differ today: history, not design. Role files came with the agent host (v1); the rules
engine came later (P2) and was never pointed at them.

## The advisor's first answer (2026-10-02)

Same thing, different shape. A rule is a small named constraint shared across roles, so "override
`kiss`" has an obvious unit. A role prompt is one document of procedure, with no obvious unit to
override. Proposal: treat each role prompt as one big rule with the same override vocabulary
(`append` by default, `replace` with a `reason`, shown by `rules explain`/`diff`).

## Further reasoning: the human may be right that they're one thing

Split "role" in two and the difference mostly disappears:

1. **The role as config** (model, tools, permissions, lifecycle hooks). This is real, it isn't
   text, and it stays in `[roles.*]` (or `workflow.toml` some day).
2. **The role's prompt text.** This is just text tagged for one role. A rule with
   `roles: [worker]` is already exactly that.

So the unified model: **there are no role prompts, only rules (call them fragments if "rule" is
the wrong word), and a role's prompt is every fragment tagged for it, resolved through the
layers.** `worker.md` becomes a handful of fragments: `worker.identity` ("you implement one task
on your own worktree"), `worker.procedure`, `worker.background-processes`, `worker.never`. Each
is overridable by id like any rule. A project adds a fragment, replaces `worker.procedure`, or
disables `worker.background-processes` with a reason, and keeps every base update to the rest.
That's layering on everything, with one mechanism and one tool (`rules explain`).

What would have to be true, i.e. the real differences to design for:

| Difference | Rules today | Role text today | What unifying needs |
|---|---|---|---|
| **Order** | unordered, rendered by id | a sequence: identity first, then procedure | an `order` (or `section`) field; identity first |
| **Severity** | must/should/may | none; procedure isn't optional | default `must`, or let severity be optional |
| **Placeholders** | none | `{{commands.check}}`, `{{branches.*}}` | substitute in every fragment (cheap: same function) |
| **Length** | a paragraph | the manager's "How you work" is ~90 lines | nothing technical; a fragment can be long. Finer ids only where a project needs to override part |
| **Delivery** | spawn prompt (after 34bw) and `bridle prime` | spawn prompt for hosted roles; `bridle prime` for orchestrator and advisor | one renderer for both paths: already the case for rules |
| **Audience** | many roles | one role | none: `roles: [worker]` |

None of these is a reason for two systems. They're fields.

What would be lost: a role prompt read top to bottom as one document in one file is easy to edit
and review. Fragments scatter it over files. Mitigations: `bridle prime <role>` (or `rules
explain --role worker`) prints the assembled prompt; or allow several fragments in one file, each
under a heading carrying its id.

Costs and risks:

- A rename of the concept ("rule" for a procedure reads oddly). Possibly "instructions" with
  `kind: rule|procedure`, or keep "rule" and accept it.
- Migration: split the six base role files; meta-notes' forks become project fragments
  (existing project, human's review, xebc migrations); `system_prompt` in `.bridle/config.toml`
  becomes a deprecated escape hatch.
- YAGNI check: is the finer granularity needed now? One project (meta-notes) forks today, and its
  fork changes the identity line, adds a "read first" list and drops bridle-specific text. All
  three are expressible as append + a project fragment, once bridle's own specifics move out of
  base. So the coarse version (one fragment per role file, append/replace) would cover today; the
  fine version is the same mechanism with more ids, so starting coarse doesn't block it.

## Options

1. **Two systems, role prompts get append/replace** (the advisor's first answer). Least work;
   keeps the split the human doubts.
2. **One system, coarse**: role prompts become rules in the rules engine, one per role to start
   (`worker.prompt`, `roles: [worker]`, `order: 0`). Same override vocabulary, same tools. Split
   into finer ids when a project needs it.
3. **One system, fine from the start**: split each role file into fragments now.

Advisor's lean: **2**. It answers the human's question ("they're the same thing") with one
mechanism, costs little more than 1 (the resolver already exists; add `order` and placeholders),
and leaves 3 as a later, incremental step. Not decided.

Depends on 34bw step 1 (rules into the spawn prompt), which is approved and being planned.
