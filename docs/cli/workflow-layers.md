# Workflow layers

The workflow (rules, role prompts, skills) is layered data, so a project can adopt bridle's
defaults and change only what it must.

```
L0  core       built into the binary: task states, edge types, command semantics
L1  base       workflow/base/                   shared by every project
L2  packs      workflow/packs/<name>/            opt-in (typescript, python, ...)
L3  project    <repo>/.bridle/                   this project's overrides and additions
L4  component  <repo>/.bridle/components/<n>/    scoped by task or spawn
```

Later layers win. A project lists its `packs` and points `workflow` at the directory
holding `base/` and `packs/` in `.bridle/config.toml`; when `workflow` is unset, a
project on an installed binary uses the copy `bridle init` vendored into
`.bridle/workflow/`, which changes only when the human runs `bridle workflow update`.

## What a layer contains

```
workflow.toml        layer settings (today only `layer = ...`; roles, gates, models planned)
rules/<id>.md        must/should/may statements with ids
guides/<id>.md       longer how-to prose
skills/<name>/       skill sources
roles/<role>.md      role prompts (a role with no system_prompt defaults to its file)
hooks/               hook scripts
facts.md             short operational facts
```

Not every layer has all of these yet.

## Overrides are explicit

A project overrides a rule by id with `override: replace|append|disable` and a `reason`;
the opt-out stays visible instead of drifting. A rule can be `locked`. See
`bridle docs priming-and-rules` for how rules reach agents.

```
bridle workflow rules explain <id>
bridle workflow update [--to <tag>]
```

Components (L4) resolve for `rules explain|diff --component`, but their rules don't reach
spawned agents yet. Rule proposals (`bridle rules propose`) and layer hooks at spawn are
planned.

## More

`docs/design/workflow-layers.md`, `docs/design/components.md`.
