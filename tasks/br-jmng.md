+++
id = "br-jmng"
title = "Every task and ticket ID the human sees is a clickable link, without relying on agents to remember"
kind = "feature"
state = "planned"
created_at = "2026-10-06T00:45:44.090Z"
updated_at = "2026-10-06T00:53:41.215536Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

original id: jmng
docs/tickets/open/every-task-and-ticket-id-the-human-sees-is-a-clickable-link-jmng.md
Ticket (the ask with the human's words; read all of it, and the thread on this task):
This task is the DESIGN only, for the human's review. Build nothing; the human decides before any build, and no build tasks are made until then.
Goal: every task and ticket ID the human sees is a clickable link, without relying on agents to remember. Related, already built: br-yfjc (agents link IDs from a configured base URL), br-bnhn/br-a3yd (UI rendering and auto-linking). Read what they do before proposing more.
The design (docs/design/linkable-ids.md, status "planned", linked from docs/README.md; record the human's decision there when made) must cover, each with rejected alternatives:
1. Where links are made deterministically: the UI and gateway link every KNOWN id when they render text (messages, comment threads, task bodies, ticket documents); match only ids that exist (how it knows them, cost, false positives like words that look like ids, ids in other projects).
2. Terminal sessions (the human reads agent output in a terminal): how [id](url) renders there (what the terminal shows, OSC 8 hyperlinks vs plain markdown, what works in the human's setups), and the aide's idea of a Stop hook that sends the agent back when its reply has unlinked IDs (no hook can rewrite what the assistant wrote): feasibility, noise, loops, cost, and how it carries to projects (workflow pack and migration, ticket xebc direction).
3. bridle send and task comments: link at write time (the CLI/daemon rewrites ids in the text it stores) or at render time; what the recipient sees in each place.
4. How ticket vk3y fits (br-vk3y/br-avu7 are held; they add a real `ticket` task field and a required --ticket): say what stays, what overlaps and what the human should decide.
Deliverable: the design doc, plus feature tickets (`bridle ticket new ... --see jmng`) for each build piece, no tasks; `bridle ticket check` clean; a short summary of the open decisions on this task's thread for the human. No code.
Model: Sonnet. Out of scope: building any of it, changing vk3y's tasks.

## Thread

### note · external:aide · 2026-10-06T00:46:13.896Z
From the human, via aide (2026-10-05 ~9:10 PM ET), on ticket jmng (docs/tickets/open/every-task-and-ticket-id-the-human-sees-is-a-clickable-link-jmng.md, task br-jmng): "Work on the design". Earlier, verbatim: "Can a posthook tool alter the text of the message that the agent sends? Pretty much none of the agents are remembering to link tasks and tickets. Every task and ticket ID should be linked. The ID itself should be clickable." Please schedule a design for jmng (design only, for the human's review before any build). Aide's recommendation to the human, not yet decided: the UI/gateway links every known ID when it renders text (deterministic, covers messages and threads), plus a Stop hook for terminal sessions that sends the agent back when its reply has unlinked IDs (no hook can rewrite assistant text). The design should also cover bridle send / comments, the terminal's rendering of [id](url), and matching only IDs that exist. br-vk3y stays held meanwhile.

### note · external:aide · 2026-10-06T00:53:41.215Z
From the human, via aide (2026-10-05 ~9:35 PM ET), on br-jmng (design for auto-linking IDs): "Okay, I'm not planning to do JMnG. I don't see anything there that's worth implementing so far, so I'm not sure what that design will come up with. I think it could just be canceled." Please cancel br-jmng (don't start the design) and close ticket jmng as won't-do with that quote. br-vk3y stays held until the human says otherwise (asking them now).
