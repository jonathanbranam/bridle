---
id: m9sd
title: "Agents don't know how to use the bridle CLI correctly: help, skills, shorter primes or guardrails?"
kind: research
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [9z2d, kae5, a67t, m7mp]
tasks: []
---

## The ask

The human, 2026-10-05 ~9:00 PM ET (verbatim): "File a different ticket for the fact that agents don't know how to use the CLI properly, and we need to figure out how to do this, how to get them to use it correctly." And, on incident 9z2d: "Do we need better instructions? Do we need better help? Do we need skills? What do we need here?"

## The problem

Agents, the orchestrator included, use bridle's CLI from habit and grep, not from knowing it. Seen on 2026-10-05 alone:

- The orchestrator didn't know `bridle advisor start` or `external:advisor/<name>` (incident 9z2d), though both were built and one was in its prime.
- `bridle --project X send` fails with a bare "error: unknown:" when BRIDLE_PROJECT names another project (br-ubdc). The bridle-ui aide was stuck on it and had no way to tell why.
- pm-1 filed br-avu7 with `bridle task new` and no ticket link (vk3y), because nothing in the CLI asked for one.

Today an agent learns the CLI from: a long prime read once (the orchestrator's is 42.8 KB), role and rule text, `--help` (which says what, rarely when or instead of what), and docs in the bridle repo that other projects' agents can't see.

## Options to weigh (a research ticket: compare, then propose)

1. **Better `--help`:** each command says when to use it, what not to confuse it with, and an example. Errors name the fix.
2. **`bridle help <topic>` / `bridle docs`:** task-shaped topics (start an advisor, message someone, file a ticket and its task) that ship with the binary (ticket kae5).
3. **Skills:** Claude Code skills per task ("start an advisor", "message a session"), loaded when relevant, kept in step with the CLI by tests.
4. **Shorter primes:** a prime short enough to be read whole, pointing at 2 and 3 for detail.
5. **Guardrails in the CLI:** refuse wrong-context use (no TTY, wrong project, missing ticket) with a message that says what to run instead.
6. **Check it:** a test or eval where an agent is given a task ("start advisor X with brief Y") and we check the command it runs.

Related: kae5 (CLI overview for agents), a67t (group the CLI), m7mp (interactive roles get their resolved rules), 9z2d.
