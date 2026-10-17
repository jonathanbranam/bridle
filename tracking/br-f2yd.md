+++
id = "br-f2yd"
title = "Roles: tell every role that 'bridle ticket submit --project <p>' is how to file a ticket against another project"
kind = "docs"
state = "planned"
created_at = "2026-10-13T23:22:31.932Z"
updated_at = "2026-10-17T15:27:14.865934Z"
created_by = "external:aide@aide"
watchers = ["external:aide@aide"]
ticket = "f2yd"
+++

docs/tickets/open/roles-tell-every-role-that-bridle-ticket-submit-project-p-is-f2yd.md

submitted by external:aide@aide

Submitted by the meta-notes aide on vesta at the human's request, 2026-10-13 ~7:25 PM ET. The human's words, verbatim:

> Is this listed anywhere in your role instructions? If not, then use it immediately to file a ticket against Bridle, explaining that the aide and the orchestrator (probably every role, in particular the aide, orchestrator, product, and advisor roles) all need to be able to do this. They need to be informed that this is how you file a ticket against an external project.

## What's missing

`bridle ticket submit` (93xm, v0.7.0) lets any principal with a token file an open task on another project's daemon for its PM to triage. No role or rule mentions it. Checked in ~/.bridle/workflow/v0.7.1: workflow/base/roles/*.md (advisor, aide, designer, document-reviewer, manager, orchestrator, project-manager, prototyper, worker) and workflow/base/rules. It is documented only in CHANGELOG.md, docs/design/cli.md, principals.md, api.md and ticket 93xm.

The result: agents don't know it exists. Tonight, 2026-10-13, the meta-notes orchestrator found a bridle bug (the usage-wake repeat, now br-63nr). It said "I can't file there" and asked the human. The aide then relayed the bug by `bridle send` to bridle's aide instead of submitting it. The human had to remember the feature themselves.

## The ask

- Tell every role, in particular the aide, orchestrator, product manager and advisor roles, that this is how to file a ticket against an external project:
  `bridle ticket submit --project <project> -k <kind> "<title>" --body-file -`
  Say when to use it instead of messaging that project's aide or orchestrator: submit for anything to be built or fixed; message only for critical bugs that need someone now, or for questions.
- Say what happens next: the target's PM (else its orchestrator) gets one message, and a drop with a reason comes back to the submitter.
- Probably a short base rule, referenced from each role, so it's written once.

## Thread

### note · external:aide@aide · 2026-10-13T23:22:31.933Z
submitted by external:aide@aide

### note · agent:pm-1 · 2026-10-13T23:22:45.310Z
Accepted by pm-1 (triage of the aide's submission). Brief for when it is readied. Model: Sonnet. Size: S (docs only).

Goal: every role learns that `bridle ticket submit --project <project> -k <kind> "<title>" --body-file -` is how to file a ticket against another project, and when to use it instead of a message (anything to be built or fixed: submit; critical bugs needing someone now, and questions: message). Say what happens next: the target's PM (else its orchestrator) gets one message; a drop with a reason comes back to the submitter.

Files: a short base rule workflow/base/rules/file-tickets-on-other-projects.md (written once: the command, when to use it, what happens next, ASCII), and one line referencing it in each role in workflow/base/roles/ (advisor, aide, orchestrator, project-manager, manager, worker, designer, document-reviewer, prototyper; the aide, orchestrator, product manager and advisor roles most). Check the exact CLI flags against `bridle ticket submit --help` and docs/design/cli.md before writing, and CHANGELOG.md. Existing projects get it through `bridle workflow update` plus `bridle sync` (no migration, no project file edits; trial branches only, per rule existing-projects). Acceptance: `bridle workflow update` plus `bridle sync` in a scratch project renders the rule and the role lines; `just check` passes. Out of scope: changing the submit command itself.

### note · external:advisor/product-manager · 2026-10-17T15:26:12.947Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.
