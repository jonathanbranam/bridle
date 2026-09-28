# Changelog

All notable changes to bridle are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.3.0] - 2026-09-28

- Per-spawn environment variables: `bridle spawn --env KEY=VALUE` for secrets that don't appear on the process or in the daemon log
- Per-spawn tool overrides: `bridle spawn --allow-tool <tool>` to allow additional Claude Code tools per spawn
- Workflow rules layer resolution engine: `bridle rules explain` and `bridle rules diff` to inspect effective rules
- Role-based message routing: `bridle send role:<name>` fans out to every live agent with that role
- Session handover: `bridle prime orchestrator` to bootstrap a new session
- Worker background process limits: document bounded/cleaned-up lifecycle for long-running jobs
- `bridle task note` to attach notes to existing tasks

## [0.2.0] - 2026-09-28

- Task management: `bridle task create`, `bridle task list`, `bridle task read`, `bridle task note`, full task record storage in state branch, and import from existing task files
- Context size governance: `bridle renew` to apply new context limits and persist them across sessions
- Agent tool allowed by default; SendMessage, Workflow, and remote-trigger tools denied by default
- `bridle logs` shows latest lines by default, not oldest
- `bridle rm` refuses a worktree with open files unless `--force` is used
- Human-readable token and cost formatting in CLI output
- Improved daemon shutdown sequence to fix Linux-only race

## [0.1.0] - 2026-09-27

- Initial release
- Daemon and CLI for spawning and supervising headless Claude Code agents
- Worktree management and agent lifecycle control
- Basic message routing and inbox support
- Configuration and containment (process groups, system limits)
