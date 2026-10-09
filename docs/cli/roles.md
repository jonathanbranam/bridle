# Roles

A role says what an agent may decide. Bridle is mechanism; the roles hold the judgement.

| Role | Hosted by | Decides | Never |
|---|---|---|---|
| Human | n/a | priorities, product questions, acceptance | |
| Orchestrator | the human's own agent (an `external:orchestrator` principal; bridle can also run it as a session) | whatever the human delegates: relaying, summarising, steering | act as the human |
| Advisor / aide | interactive sessions (`bridle session advisor\|aide`) | talk with the human about the work or the running system | |
| Manager | bridle, long-lived | decomposition, plans, ordering, what to ask the human | write bulk code; accept work |
| Project manager | bridle, if the project defines it | plans `open` tasks and owns the queue | |
| Worker | bridle, one per task | how to implement a planned task | change design silently; accept; talk to the human directly |
| Integrator | not an agent: `bridle task land` | merge gate, conflict probes | resolve a semantic conflict |

Other roles exist as prompts (`prototyper`, `designer`, `document-reviewer`). A **reviewer** role is
planned (no role file or config yet).

## What a worker does

1. Reads the task (`bridle task show <id>`) and the project's docs, then builds it in its
   own worktree, committing on its branch.
2. Runs the project's checks, merges the local `main` into its branch.
3. Writes a summary (`bridle task summary <id> --file ...`) and reports to its manager
   (`bridle send <manager> --task <id> "done: ..."`). Being blocked: `bridle send
   <manager> --question "..."`.

Finishing is not landing: only the manager's or human's merge accepts work.

## Where roles are configured

`[roles.*]` in `<repo>/.bridle/config.toml` sets model, autostart and the system prompt;
a role with no `system_prompt` defaults to `<workflow>/base/roles/<role>.md`. Setting
roles and models in `workflow.toml` is planned.

## More

`docs/design/roles-and-lifecycle.md`, `docs/design/agent-host/roles-and-config.md`,
`docs/design/agent-host/principals.md`.
