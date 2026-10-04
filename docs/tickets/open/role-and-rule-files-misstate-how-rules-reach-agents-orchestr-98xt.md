---
id: 98xt
title: Role and rule files misstate how rules reach agents; orchestrator and advisor get none
kind: question
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [34bw, ntca]
tasks: [br-0473]
---

## The ask

The human, verbatim: "there should be a clear delineation between what we're planning to build
and what works today ... so that when the agents are reading, they can understand the difference
between what's been planned and what the vision is and what actually exists today."

A status check of `docs/design/agent-host/`, `cli.md`, `storage.md`, `CLAUDE.md`,
`docs/README.md` and `workflow/` against `main` at `5aca429` (2026-10-03). The docs in that set
were fixed in place. The `workflow/` role and rule files are prompts agents run on, and
[[the-workflow-doesn-t-reach-agents-resolved-rules-hooks-and-o-34bw|34bw]] is changing them, so
they weren't edited: their wrong claims are listed here. Related:
[[docs-design-much-of-not-built-is-built-but-unwired-and-claud-ntca|ntca]] (the same pattern
across the rest of `docs/design/`).

## What the code does today

- **Daemon-spawned agents get their resolved rules in the system prompt** since `9561950`
  (br-2242): `stable_system_prompt` appends `## Workflow rules` from `Config::role_rules_text`
  (`crates/bridle-daemon/src/config.rs:2648-2651`, `:2660`), called on spawn, resume and renew
  (`supervisor.rs:1044,2506,2706`). Base, packs and project layers; no components, facts or
  guides.
- **The orchestrator and advisor get none.** They aren't daemon-spawned: `bridle session` opens
  them with "Run `bridle prime orchestrator`" / "`bridle prime advisor`"
  (`crates/bridle/src/session.rs:44,47`), and those print the role file, the project addendum and
  (orchestrator) the handover, with no rules (`crates/bridle/src/commands/orchestrator.rs:83-133`).
- **Nothing runs `bridle workflow sync`** in bridle's repo: `CLAUDE.md` has no
  `bridle:managed` block, `.claude/settings.json` has no `hooks`.

## Wrong claims in `workflow/`

| File:line | Says | Today |
|---|---|---|
| `rules/{ask-blocking:17, kiss:31, docs-current:22, ticket-references:15, human-timezone:18, yagni:21, plan-discipline:19, out-of-scope:17, missing-tools:16, doc-links:18, record-decisions:15}.md` | "Until `bridle workflow sync` renders rules into agents, the role prompts in the `bridle` repo's `workflow/base/roles/` carry this." | Rules are rendered into the spawn prompt, not by `sync`; and br-899d (`5a0593f`) took the restatements out of the role prompts, so they no longer carry it. |
| `rules/cost-of-not-doing.md:19-21`, `rules/work-flow.md:22-24` | "Agents read the rule files themselves (`bridle sync` writes the CLAUDE.md pointer to them; rules are not rendered into the prompt), and the role prompts ... carry this too." | All three parts are wrong now (34bw step 4 rewrote these lines before step 1 landed). |
| `roles/worker.md:4-5` | "The workflow rules named below are files: see `CLAUDE.md`'s bridle block for where they live." | The rules are in the worker's system prompt; bridle's `CLAUDE.md` has no bridle block. |
| `roles/orchestrator.md:52,102,104-105,108,130`, `roles/advisor.md:90-94`, `roles/prototyper.md:44-47`, `roles/product-manager.md:36` | cite `workflow/base/rules/<id>.md` paths | Those paths exist only in bridle's repo. A project `bridle daemon init` vendored has them under `.bridle/workflow/base/rules/`; a project's own overrides are in `.bridle/rules/`. For the orchestrator and advisor these citations are their only route to the rules. |

Also stale, in code (not edited, code is out of scope): the CLAUDE.md block `sync` writes still
says "Read the rule files ... at the start of a session" (`crates/bridle-daemon/src/sync.rs:113-122`),
which is now redundant for spawned agents and leaves out packs.

## Recommendation

1. One chore: replace the 13 "Until `bridle workflow sync`..." / "Agents read the rule files
   themselves..." lines with nothing (the rule body is already in the prompt), and drop
   `worker.md:4-5`'s second sentence. Fold it into whatever 34bw task touches `workflow/` next.
2. Cite rules by id in the role files, not by `workflow/base/rules/` path (as `manager.md` and
   `worker.md` already do).
3. For the human: should `bridle prime orchestrator` and `prime advisor` print the role's
   resolved rules the way `prime worker` does? It's one call (`rules::rules_section`), and it
   would make an override in a project change what its orchestrator and advisor do, which today
   it can't. This is the same question as
   [[are-roles-and-rules-the-same-thing-one-layered-kind-of-promp-vp9e|vp9e]]'s role addendum,
   from the other side. Recommended: yes (rules as the one per-project mechanism).

## Next steps (advisor workflow, retiring, 2026-10-04)

Two parts. The stale "until bridle workflow sync renders rules" lines in 11 rule files, the dead CLAUDE.md-block pointer in worker.md and sync.rs's stale managed block are plain fixes, now that 34bw step 1 has landed; they need only a go to queue. The question (should prime orchestrator|advisor include resolved rules; recommended yes) waits on the human. Task br-0473 was dropped in pm-1's k7tm sort.
