---
id: file-tickets-on-other-projects
severity: should
roles: [advisor, aide, orchestrator, project-manager, manager, worker, designer, document-reviewer, prototyper]
---
To ask another project for work, file it on that project's daemon with `bridle ticket submit`,
not in a message.

- **The command**: `bridle ticket submit --project <project> -k <kind> "<title>" --body-file -`,
  with the body on stdin (`--body "<text>"` for a short one). `--project` picks the target's daemon
  from the registry; it defaults to `BRIDLE_PROJECT`. The kinds are the ones `bridle ticket submit
  --help` lists.
- **When to submit**: anything to be built or fixed in that project. Use a message instead for a
  critical bug that needs someone now, and for a question.
- **What happens next**: the submission files a pending task on that project's daemon (no ticket
  file yet). The target's project manager (else its orchestrator) gets one inbox message to triage
  it. A drop comes back to you with a reason.
- **Your own project** is not covered: file there with `bridle ticket new` (rule `tickets`).

Why: a message is lost in a busy inbox and has no thread to answer on. A submitted task is kept
on the target's daemon until its manager triages it, and a drop tells the submitter why.
