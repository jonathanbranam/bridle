+++
id = "br-ca8a"
title = "Life assistant on the notes repo via the meta-notes CLI: survey and shape (phyy)"
kind = "explore"
state = "planned"
created_at = "2026-09-30T04:13:56.493Z"
updated_at = "2026-09-30T04:18:57.720433Z"
size = "M"
+++

See docs/questions/open/a-life-assistant-agent-on-the-notes-repo-phyy.md. Survey the meta-notes CLI against capture/check-in verbs (tasks, reminders, events, remember, project and maintenance check-ins), propose the shape (pack vs project, machine, git), and bring the open questions to the human. Read-only on the notes repo.

## Thread

### note · agent:pm-1 · 2026-09-30T04:14:24.103Z
PM brief: read the ticket first, then survey the meta-notes CLI (bin/meta-notes, scripts/meta_notes/ in a scratch clone or a read-only look at the checkout) against the verbs the ticket lists. Frame OPTIONS only (pack vs project layer, which machine, git handling of the notes repo) with a recommendation each; write the findings as a short doc/ticket update in docs/questions/open/ and list the open questions for the human. Do NOT run the CLI against, write to, commit to or branch the human's real notes repo (workflow/base/rules/existing-projects.md); use a scratch copy for any experiment. Acceptance: just check passes; the notes repo is untouched. Model: Sonnet. Out of scope: building the agent, choosing answers to the open questions.

### note · external:advisor · 2026-09-30T04:18:57.720Z
From the human, via advisor (2026-09-30): use bridle with a different workflow (a life-admin pack in workflow/); the notes repo becomes a bridle project, and the agent sets it up and pushes it itself; meta-notes has most of what's needed, recurrence is missing. Recorded in phyy.
