---
id: ready-without-asking
severity: must
roles: [orchestrator, product-manager, advisor, aide, project-manager]
---
Ready a ticket yourself when it is a fix or self-evident; bring the human only what needs them.
`bridle ticket ready <id>` needs a committed ticket with its ask under `## The ask`.

- **Ready at once, no question to the human:** kinds `bug`, `fix`, `incident`, `chore`, `docs`, and
  any ticket whose ask is self-evident (one clear fix, no choice between designs, no open question
  for the human), at any priority. Critical and high first. Whoever files it or first sees it
  `pending` readies it (orchestrator, product manager, advisor or aide) and says so in the
  ticket's thread.
- **Bring to the human first:** `feature`, `epic`, `arch-revision`, `re-evaluate`, `research`; any
  ticket with a literal open question for the human; anything that changes how the human works or
  what they see in a big way; anything the human said to hold.
- **The human's stop:** the human can always say "stop and ask me for a design" (in a thread, to
  any role, or as `hold: design` in the ticket's front matter). That ticket waits for them, and
  nobody readies it until they say go.
- **When unsure, ready it** and tell the human in one line (they can stop it), rather than asking
  and waiting.

Why: the human, 2026-10-17: "I'm not sure why tasks are waiting for me to be readied all the time
now. That shouldn't be the way it works. ... I should always have the ability to say, 'Stop and ask
me for a design.' ... we need to keep things going, keep work going." (ticket mhjn)
