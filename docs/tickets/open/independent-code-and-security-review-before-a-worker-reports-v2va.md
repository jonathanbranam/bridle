---
id: v2va
title: Independent code and security review before a worker reports done
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: [hvxk, 2bzw]
---

## The ask


The human, 2026-10-01, verbatim (via the advisor):

> I think a step we need to consider adding to workflow (main bridle provides this, consumers opt
> in or out):
>
> - code review by independent agent
> - security review by independent agent
>
> Don't rush these but note that it should be something available. These should pass before a
> worker reports back to manager that the work is done.

The human, 2026-10-01, verbatim (follow-up):

> A task can require certain reviews and a workflow can require certain reviews. Eg a workflow
> might dictate: every change to module/component/entire project needs an independent security
> review. Or it could be optional and left to the discretion of the PM or whoever creates the
> ticket / task could indicate that a specific review is necessary.

The human, 2026-10-01, verbatim (second follow-up):

> Independent reviews should be signed and added to the task including the commit sha and the
> task body - particularly important for security reviews.
>
> If a human review is required (same options apply) that also should be signed which the humans
> token in the same way.

## What's there now (at 0c741f6)

- `docs/design/roles-and-lifecycle.md` (future work) has a **Reviewer** role: "strong model,
  never the task's implementer", checking "whether a diff matches its plan and specs", and an
  `in_review` state between `claimed` and `integrated`. Nothing implements it. There is no
  security review anywhere.
- `docs/design/gates.md` (future work) has an accept gate `default = "reviewer+tests"`.
- How consumers opt in or out: `docs/design/workflow-layers.md`. Base rules apply by default,
  and a project opts out with `override: disable` and a `reason`; packs (L2) are opt-in.
- Today a worker runs `just check`, hands off, and the manager lands. No review step between.
- Components (`docs/design/components.md`, `bridle task new --component`) already scope a task
  to part of a repo.
- Signing: today the daemon records each call's `actor` from its bearer token, and the store
  keeps only token hashes. `docs/design/agent-host/principals.md` ("What this is and isn't")
  calls that "attribution that honest agents can't get wrong by accident, not a security
  boundary". How strong provenance should be is open in
  [[how-strong-agent-provenance-should-be-2bzw|2bzw]].

## Decided (2026-10-01)

The human, verbatim: "Right so the tokens aren't secure and that will be hard. But let's assume
the tokens are secure and then use them for cryptographic signing. This isn't a truly secure hard
system but we are experimenting with important functionality."

So reviews (agent and human) are cryptographically signed using the reviewer's bridle token,
on the working assumption that tokens are secure. Hardening token storage isn't part of this
ticket (2bzw).

## Decided: bridle runs reviews, on the task (2026-10-01)

pm-1 asked (via the orchestrator): "Should reviews run before the merge, not before the worker
reports done? Workers can't spawn agents, so the manager would start two read-only reviewers, one
for code and one for security, after the worker hands off and before the merge. Projects opt out
by rule. Reviews are skipped for changes under about 30 lines, docs-only changes, and when the
budget is winding down." It recommended yes, with the manager running the gate.

The human, verbatim:

> bridle should spawn the agents, not the manager; IDK how things are written today, but when the
> worker is done with his part, he should mark the task as "ready for review" or whatever and then
> bridle evaluates some metadata on the task, looks at the workflow rules, and starts any required
> review agents. they should do their job, sending reports to the worker, they can communicate,
> but the review agent must approve the work, then sign the task, when reviews are all signed,
> then bridle messages the manager;
>
> The system should be enforcing this not an agent. If I set a policy of security review on every
> task or I set a task as "requires security review" then I should be able to see that it was done
> correctly.
>
> Also - sorry I misspoke earlier; really the agents should not message each other directly, they
> should communicate using the task itself - adding comments to the task and updating the task
> status when they're done; the security review agents interactions with the worker should be
> permanent record on the task so that we can audit and review what happened - I want to learn
> from these interactions so that the worker guidelines and reviewer guidelines can be improved.
>
> E.g. if the security reviewer is too lenient, we want to know; if it is too strict, same; if we
> spend $10 on code review between two agents, I want to have a record of their interactions and
> fix the instructions for them so we spend a reasonable amount on this.

So, in short:

- The daemon, not the manager or any agent, enforces reviews. The worker moves the task to a
  "ready for review" state. Bridle reads the task's metadata and the workflow rules, spawns the
  required reviewers, and tells the manager only when every required review has signed.
- Reviewers and the worker talk only on the task: comments and status changes, no direct
  messages. That's the permanent, auditable record, kept so reviewer and worker guidelines and
  their cost can be tuned.
- A reviewer must approve and sign before the task moves on.

## Decided: review thresholds (2026-10-01)

On pm-1's proposed skips (under about 30 lines, docs-only, budget winding down), the human,
verbatim:

> I didn't rule on those. sorry the answer is: the workflow should define that; I am fine with
> those guidelines as defaults EXCEPT budget running low; if budget is low then the work shouldn't
> merge. a required review should not be skipped for that reason. Human is authority here, so they
> can override that if they want to. general guideline though is that if the review is listed as
> required, it should be done.
>
> The review thresholds should be part of the workflow and enforced and tracked by the system.

So:

- The workflow defines the review thresholds, and the system enforces and tracks them.
- Defaults: no review for changes under about 30 lines or docs-only changes.
- A low budget never skips a required review. The work waits and doesn't merge.
- A required review is always done. Only the human can override that.
