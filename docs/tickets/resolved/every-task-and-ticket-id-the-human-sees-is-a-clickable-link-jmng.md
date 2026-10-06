---
id: jmng
title: Every task and ticket ID the human sees is a clickable link, without relying on agents to remember
kind: feature
opened: 2026-10-06
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: [yfjc, vk3y]
tasks: [br-jmng]
closed: 2026-10-06T00:53:58Z
---

## The ask

The human, 2026-10-05 ~9:00 PM ET, verbatim (to bridle's aide): "Can a posthook tool alter the text of the message that the agent sends? Pretty much none of the agents are remembering to link tasks and tickets. Every task and ticket ID should be linked. The ID itself should be clickable."

The rule from yfjc (link-ids-for-the-human) isn't followed reliably: agents forget. The human wants it to happen every time, and the ID itself to be the link (not a URL printed beside it).

Where agents' text reaches the human, and what can change it deterministically (aide's notes, to be checked by whoever designs this):
- Messages and task threads shown in the bridle UI: the UI (or gateway) can turn every known task and ticket ID into a link when it renders, whatever the agent wrote. No agent has to remember.
- Messages sent with `bridle send` / task comments: bridle could add the links when it stores or delivers them. A Claude Code PreToolUse hook can also rewrite a tool call's input before it runs, but that is more fragile than doing it in bridle.
- A session's own text in the terminal (aides, advisors, the orchestrator): no Claude Code hook rewrites the assistant's reply. A Stop hook can check the reply for unlinked IDs and send the agent back to correct it, which enforces the rule but adds a turn.

Matching should only link IDs that exist (a lookup), so four-character words aren't linked by mistake.

## Resolution

Won't do. The human, 2026-10-05 ~9:35 PM ET (via aide): "Okay, I'm not planning to do JMnG. I don't
see anything there that's worth implementing so far, so I'm not sure what that design will come up
with. I think it could just be canceled." Task br-jmng dropped before any design started.
