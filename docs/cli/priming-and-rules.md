# Priming and rules

An agent learns how to behave from its role prompt plus the **rules** that apply to it.

## Rules

A rule is one markdown file with frontmatter: an `id`, a `severity` (`must`, `should`,
`may`) and `roles:` (who gets it). Rules live in the workflow layers (`rules/<id>.md`)
and in the project's own `.bridle/rules/`. A project overrides a rule by id with
`override: replace|append|disable` and a reason, and may lock one.

```
bridle rules explain <id>     # which layer won, and why
bridle rules diff             # what the project changed
```

## How they reach an agent

- **Spawned agents** (worker, manager, project-manager, ...): the daemon resolves the
  role's rules and puts them in the system prompt at every spawn, resume and renew. A
  rule changed in the workflow shows up at the agent's next one of those.
- **Interactive roles** (orchestrator, advisor, aide, prototyper, document-reviewer):
  `bridle prime <role>` prints the role prompt followed by its resolved rules;
  `bridle session <role>` runs it as the opening prompt.
- **Not wired in:** `bridle prime worker|planner` (facts, guide pointers, component
  rules) isn't run by spawned workers, so those don't reach them yet.

Role names in `roles:` are the tags the roles use: `orchestrator`, `advisor`, `aide`,
`manager`, `worker`, `project-manager`, `reviewer`, `prototyper`, `document-reviewer`.

## Project instructions

`CLAUDE.md` in the repo is read by Claude Code itself; bridle's block in it points at
the rule files. Claude Code's memory is off for every bridle agent: anything worth
keeping goes in the repository.

## More

`docs/design/workflow-layers.md`, `docs/design/agent-host/roles-and-config.md`.
