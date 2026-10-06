+++
id = "br-jmng"
title = "Every task and ticket ID the human sees is a clickable link, without relying on agents to remember"
kind = "feature"
state = "pending"
created_at = "2026-10-06T00:45:44.090Z"
updated_at = "2026-10-06T00:46:13.896231Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

original id: jmng
docs/tickets/open/every-task-and-ticket-id-the-human-sees-is-a-clickable-link-jmng.md

## Thread

### note · external:aide · 2026-10-06T00:46:13.896Z
From the human, via aide (2026-10-05 ~9:10 PM ET), on ticket jmng (docs/tickets/open/every-task-and-ticket-id-the-human-sees-is-a-clickable-link-jmng.md, task br-jmng): "Work on the design". Earlier, verbatim: "Can a posthook tool alter the text of the message that the agent sends? Pretty much none of the agents are remembering to link tasks and tickets. Every task and ticket ID should be linked. The ID itself should be clickable." Please schedule a design for jmng (design only, for the human's review before any build). Aide's recommendation to the human, not yet decided: the UI/gateway links every known ID when it renders text (deterministic, covers messages and threads), plus a Stop hook for terminal sessions that sends the agent back when its reply has unlinked IDs (no hook can rewrite assistant text). The design should also cover bridle send / comments, the terminal's rendering of [id](url), and matching only IDs that exist. br-vk3y stays held meanwhile.
