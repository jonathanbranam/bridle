+++
id = "br-efs2"
title = "Review and comment on kuw2, the machine daemon design (docs/tickets/open/a-machine-daemon-...-kuw2.md), with document review"
kind = "chore"
state = "claimed"
created_at = "2026-10-04T15:40:03.732Z"
updated_at = "2026-10-09T11:04:37.885934Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "human",
    "external:advisor/product-manager",
]
+++

Review and comment on ticket **kuw2**, the machine daemon design:
`docs/tickets/open/a-machine-daemon-one-per-machine-doing-machine-wide-work-onc-kuw2.md`
("A machine daemon: one per machine, doing machine-wide work once, starting, stopping and moving projects").

You asked for this 2026-10-04: "I want to review it in some detail before anything is scheduled.
This should address message delivery as well." Nothing is scheduled until you've reviewed it.

Using document review (x8jt), steps as in br-twg8:
1. If not done yet: `cd /Volumes/Data/work/bridle-ui-workspace/bridle-ui && npm run install-ui`
2. `cd /Volumes/Data/work/bridle/bridle && bridle review add docs/tickets/open/a-machine-daemon-one-per-machine-doing-machine-wide-work-onc-kuw2.md`
3. Start the gateway (`bridle gateway`) and open the UI.
4. Comment on kuw2 (UI or Obsidian, `> [!comment]`). Wait 7 minutes, or press "Request review" /
   `bridle review now <path>`; the document's agent answers in the file and commits.

Read with it: 3haz (mail across machines) and gtzx (seats), both waiting on your review, may change
shape if a machine daemon exists. What rubs about the review tool goes on x8jt.

Done when: you've commented on kuw2 and are happy with the design, or have said what to change.
Finish with `bridle task done <this id>`.

## Thread

### note · external:aide · 2026-10-04T15:40:03.735Z
created for the human, priority normal

### note · external:aide · 2026-10-04T15:40:03.737Z
To-do for you (normal priority): Review and comment on kuw2, the machine daemon design (docs/tickets/open/a-machine-daemon-...-kuw2.md), with document review. Finish it with `bridle task done br-efs2`.

### note · external:advisor/product-manager · 2026-10-09T11:04:37.885Z
watching the task
