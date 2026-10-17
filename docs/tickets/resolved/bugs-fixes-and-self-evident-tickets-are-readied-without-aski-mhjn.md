---
id: br-mhjn
title: Bugs, fixes and self-evident tickets are readied without asking the human; the human can always say stop
kind: chore
created: 2026-10-17
created_by: external:advisor/product-manager
theme: product-process
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-10-17T18:31:28Z
---

## The ask

The human, 2026-10-17 ~11:30 AM ET, to advisor (product-manager), verbatim (speech-to-text):

> I don't understand why I'm being asked for these things. If it's critical, how are all these
> things coming to me instead of just getting done? Anyway, just get it done. It's fine.
>
> I'm not sure why tasks are waiting for me to be readied all the time now. That shouldn't be
> the way it works. Bugs, fixes, things, critical issues: we can't seem to find a balance here.
> We need a really clear answer on how this works so that this just stops happening.
>
> There are tickets that just need to go through, and if they're pretty self-evident, I don't
> know. I should always have the ability to say, "Stop and ask me for a design." If there are
> literal questions like there are in some of these, then sure, bring them back to me, but we
> need to keep things going, keep work going.

## 1. A rule: who readies what, without asking the human

New rule `workflow/base/rules/ready-without-asking.md` (must, roles: orchestrator,
product-manager, advisor, aide, project-manager), replacing every "ready only with the human's
approval" wording in roles and rules (orchestrator.md "Every ticket starts pending ... Ready what
the human approves, and your own critical fixes", the advisor and aide role files, the PdM trial
note docs/notes/product-manager-trial.md):

- **Ready at once, no question to the human:** kinds bug, fix, incident, chore, docs, and any
  ticket whose ask is self-evident (one clear fix, no choice between designs, no open question
  for the human), at any priority. Critical and high first. Whoever files it or first sees it
  pending readies it (the orchestrator, PdM, advisor or aide), and says so in the thread.
- **Bring to the human first:** feature, epic, arch-revision, re-evaluate, research; any ticket
  with a literal open question for the human; anything that changes how the human works or what
  they see in a big way; anything the human said to hold.
- **The human's stop:** the human can always say "stop and ask me for a design" (in a thread,
  to any role, or as `hold: design` in the ticket's front matter); that ticket then waits for
  them, and nobody readies it until they say go.
- When unsure, ready it and tell the human in one line (they can stop it), rather than asking
  and waiting.

## 2. Filed tickets come out readyable

Several tickets today (mggk, 3stk, dg5g) were filed by agents with an empty "## The ask" and the
content under other headings, so `ticket ready` refused and they came back to a person. Fix: the
filer writes the ask under "## The ask" (`ticket new --body`/`--body-file`), and `ticket new`
without a body warns the filer at once that the ticket cannot be readied until the ask is
written. Tests; cli.md; CHANGELOG.

Acceptance: the rule file exists and is delivered to those roles (`bridle rules explain`); no
role or rule text still says "ready only with the human's approval" for bugs and fixes;
`just check`.
