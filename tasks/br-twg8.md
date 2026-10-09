+++
id = "br-twg8"
title = "Try document review (x8jt) on gtzx: install the UI, review add, start the gateway, comment"
kind = "chore"
state = "claimed"
created_at = "2026-10-04T13:16:26.409Z"
updated_at = "2026-10-09T11:04:37.815590Z"
created_by = "external:advisor/doc-review"
watchers = [
    "external:advisor/doc-review",
    "human",
    "external:advisor/product-manager",
]
+++

Ticket x8jt (docs/tickets/open/review-a-document-with-an-agent-highlight-comment-and-the-ag-x8jt.md): review a document by highlighting and commenting, and a per-document agent replies or revises. Advisor doc-review ran it 2026-10-03/04 and is retiring for your restart.

## Where it stands (2026-10-04)

Everything approved is built, integrated and on main; nothing has been tried end to end yet.

- bridle: br-rp53 document-reviewer role · br-aj9d bridle notices comments, sends after 7 quiet minutes · br-5paw gateway reads/saves a document · br-pwtw gateway commits it on main · br-qttb `bridle review now [--resend]` and the `· sent YYYY-MM-DD HH:MM` mark
- bridle-ui: ui-acf0 document view (comments to the side, highlight to comment, tags read on open) · ui-c39e "Request review" button with resend
- The daemon restarted on the new build 2026-10-04 8:25 AM ET.

## Next steps (yours)

1. Install the current UI (the one in ~/.bridle/ui is from Oct 2):
   `cd /Volumes/Data/work/bridle-ui-workspace/bridle-ui && npm run install-ui`
2. Put the trial document under review:
   `cd /Volumes/Data/work/bridle/bridle && bridle review add docs/tickets/open/seats-every-role-is-a-named-tracked-seat-that-outlives-its-s-gtzx.md`
3. Start the gateway (`bridle gateway`) and open the UI.
4. Comment on gtzx (UI or Obsidian, in the `> [!comment]` format). Wait 7 minutes, or press "Request review" / run `bridle review now <path>`. The agent `doc-seats-...` answers in the file and commits.
5. Tell an advisor what rubbed; it records it on x8jt and files fixes.

Still deferred on x8jt: thread IDs; showing what a revision changed (the git diff serves for now); diagrams (yyzm).

Done when: you've run a review round on gtzx and the follow-ups are on x8jt.

## Prompt for a new advisor

Paste into `bridle session advisor doc-review` (or send it as its brief):

> You are advisor doc-review, picking up ticket x8jt (document review: highlight, comment, a per-document agent replies or revises). Read x8jt whole, then this to-do (its task id is in my to-dos), then docs/design/agent-host/daemon.md "Document review" and the `bridle review` lines in docs/design/cli.md, and workflow/base/roles/document-reviewer.md. Everything approved is built and on main (tasks listed in x8jt's front matter); I haven't tested it yet. Help me run the first review round on ticket gtzx: walk me through the setup steps in this to-do, then watch the round with me (`bridle agents`, `bridle agent logs doc-<stem>`, git log of the document) and record what works and what rubs on x8jt, quoting me. With my approval, send fixes to the orchestrator to file (bridle-ui tasks go through it). Keep it short; I want this moving quickly.

## Thread

### note · external:advisor/doc-review · 2026-10-04T13:16:26.411Z
created for the human, priority normal

### note · external:advisor/doc-review · 2026-10-04T13:16:26.423Z
To-do for you (normal priority): Try document review (x8jt) on gtzx: install the UI, review add, start the gateway, comment. Finish it with `bridle task done br-twg8`.

### note · external:advisor/product-manager · 2026-10-09T11:04:37.815Z
watching the task
