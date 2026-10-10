---
id: scheduled-wakes
severity: should
roles: [project-manager, manager, worker, prototyper, designer]
---
When you need to wake at a time (before a meeting, at 3 AM, in 20 minutes), schedule a message
to yourself instead of looping short waits or sleeping.

- **Wait at the maximum timeout.** A message or task change ends a wait at once, so a long wait
  costs nothing. Short wait loops burn turns and tell you nothing on waking.
- **Schedule the wake**: `bridle schedule add --at "YYYY-MM-DD HH:MM" --message "<why you are waking>"`
  for once (RFC 3339 also works), or `--cron "<m h dom mon dow>"` for a repeat. `--to` defaults to
  you; times are in `--tz` (default America/New_York). The message is your reminder of the purpose,
  so write what to do on waking, not "wake up".
- **Clean up**: `bridle schedule list` shows yours, and `bridle schedule rm <id>` removes one that
  is no longer needed. Remove your repeating schedules when the work they serve is done.

Today the daemon refuses the external roles (orchestrator, advisor, aide) as schedule targets, so
they keep their own wait text.

Why: the human, 2026-10-09: "agents, instead of waking themselves up with shorter wake-up timers,
start using this functionality. They set their wake-up timer to the maximum and then rely on
scheduled message sends to wake themselves up, and that gives them a reminder of what the purpose
of the wake-up is."
