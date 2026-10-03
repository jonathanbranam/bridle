+++
id = "br-c4f4"
title = "Advisor wake waits about 90 minutes, not 5"
kind = "chore"
state = "planned"
created_at = "2026-10-03T00:00:32.650Z"
updated_at = "2026-10-03T00:01:21.790019Z"
+++

original id: 789x
Ticket: docs/tickets/open/advisor-wake-waits-about-90-minutes-not-5-789x.md (read it; human's words verbatim). Files: workflow/base/roles/advisor.md ('Waiting for messages'), docs/design/agent-host/api.md and the wake code only to READ the cap.
Goal, ROLE PROMPT ONLY, NO DAEMON OR CLI CHANGE (the wake cap stays 25 minutes; raising it is a separate later daemon change, held until the human is back): change the advisor wait to about 90 minutes using the current cap. The prompt gives one background command that re-waits in a shell loop: run 'bridle agent wake external:advisor[/$BRIDLE_ADVISOR_NAME] --timeout 1500' up to 4 times, stopping the loop as soon as it exits with anything other than the timeout code (exit 4: check docs/design/agent-host/cli.md / the CLI for the exact codes), so a message ends the wait at once and a timeout only re-waits. Provide both the named and unnamed forms. Say the timeout is only a fallback (a message ends the wait immediately), that the whole thing is one background task of about 100 minutes (under Claude Code's 2-hour limit), and keep the existing rules (mark inbox read, error handling, mail waiter). Keep it short. Tests: if role text is tested for key phrases, update; otherwise none. Docs: CHANGELOG. Acceptance: just check passes. Model: Haiku. Migration: none (workflow base file; projects inherit). Out of scope: the daemon/CLI cap, other roles.
