---
id: ufrw
title: "Auto mode's classifier context per machine for bridle projects: who writes autoMode.environment, and how it stays current"
kind: question
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-ufrw]
closed: 2026-10-09T23:11:05Z
---

## The ask


Requested by the human 2026-10-04, relayed by the track-web session track-web-f5: "investigate how
to configure Claude Code auto-mode settings for bridle projects on a per-machine basis." File as an
investigation.

Background (from track-web-f5):

- Auto mode's classifier context is `autoMode.environment`, a list of plain-text lines (trusted repo,
  trusted domains, where secrets live, deploy targets).
- Claude Code reads `autoMode` only from user `~/.claude/settings.json`, managed settings, or a
  per-launch `--settings` file / the Agent SDK. Project `.claude/settings.json` and
  `.claude/settings.local.json` are ignored on purpose, so a repo can't mark itself trusted
  (https://code.claude.com/docs/en/auto-mode-config.md, "Where the classifier reads configuration").
- The human's `~/.claude/settings.json` block on dalek was generated from inside track-web and only
  describes track-web. Its "Trusted repo" line pointed at an old checkout
  (`/Volumes/Data/work/pi/track-web`), not the bridle workspace layout
  (`/Volumes/Data/work/track-web-workspace/track-web` plus `.bridle/state` worktrees). The human fixed
  that line by hand, but the block is global: in every other project (bridle, harness, otters, ...)
  the classifier is told track-web is the only trusted repo.
- Claude can't edit `~/.claude/settings.json`; auto mode blocks it as self-modification. A fix is
  applied by the human or by tooling the human runs.

Questions:

1. Should bridle generate or maintain the machine-level `autoMode.environment` block (say, a `bridle`
   command that writes it for the human to review), covering every bridle-managed workspace on the
   machine, workspace dirs and worktree paths included?
2. Or should bridle launch its sessions and agents with `--settings <generated file>` per project, so
   the context matches the project without touching the global file?
3. How are project facts (deploy on push to main, prod hosts, domains) kept apart from machine facts
   (trusted repos, remotes under github.com:jonathanbranam/)?
4. How does it keep from going stale when workspaces move (the cause of this incident)?

## Recommendation

Written 2026-10-04 by worker automode-study (br-ufrw). Docs only; no live run, and
`~/.claude/settings.json` was not touched.

### What the sources say

- **Where `autoMode` is read** (Claude Code docs, auto-mode-config, "Where the classifier reads
  configuration"): user `~/.claude/settings.json`, managed settings, and the `--settings` flag /
  Agent SDK as "per-invocation overrides for automation". Project `.claude/settings.json` and
  `.claude/settings.local.json` are ignored. Entries from the scopes are combined (additive).
- **`"$defaults"`**: an `environment` array without the literal `"$defaults"` replaces the built-in
  list for that section; with it, the defaults are spliced in. Sections are independent. `claude
  auto-mode config` prints the effective result; `claude auto-mode defaults` the built-ins.
- **`/auto-mode-setup`** drafts entries from the *current project* and recent sessions and writes
  to `~/.claude/settings.json`. That is how the global block came to describe only track-web.
- **Bridle already launches with `--settings`**: `crates/bridle-claude/src/command.rs` (`settings_json`,
  every spawn) and `crates/bridle/src/session.rs` (`orchestrator_settings`, `advisor_settings`,
  `with_layer_hooks`, `claude_args`). Layer hooks are already merged this way
  (`docs/design/workflow-layers.md`, "Layer hooks are live at spawn").
- **Finding that narrows the problem**: spawned agents never use auto mode as built. They run with
  `--permission-prompts none`, `--setting-sources project` (which excludes `~/.claude/settings.json`
  on purpose, command.rs:153) and `permission_mode` `acceptEdits` / `dontAsk` (config.rs:392, 425,
  458). So the wrong global block affects only the human's interactive sessions: `bridle session
  orchestrator|advisor|aide` and plain `claude` in a bridle workspace. Spike 01
  (`docs/spikes/01-stream-json-findings.md`) records `permissionMode":"dontAsk"` and no permission
  prompts in stream-json; it says nothing about auto mode or the classifier.

### Answers

**Q2 first, since it decides the rest: launch with a generated `--settings`.** For sessions bridle
starts, put `autoMode.environment` in the `--settings` JSON, built at launch from the project in
the current directory. The docs name this scope for exactly this use, nothing global is written,
and the block can't go stale: it is computed from where the workspace is *now*, every launch.

**Q1: do not write the global file.** Bridle should not maintain `~/.claude/settings.json`. Claude
can't edit it (auto mode blocks that), and a tool that rewrites a human-owned global file is the
riskier shape. Plain `claude` sessions outside `bridle session` are the only gap; cover it with a
print-only `bridle auto-mode print` (stdout JSON, same generator, all projects the machine knows)
that the human pastes if they want. Bridle never writes it. Because scopes are additive, the human
can also keep a small truly-global block (their GitHub org, `$defaults`) in the user file and let
bridle's per-launch block add the project's workspace paths.

**Q3: two layers, mirroring where the facts live.**
- *Machine facts* (trusted remotes such as `github.com:jonathanbranam/`, the workspace root, the
  human's domains): in `~/.bridle/config.toml` (the existing machine-level file) as
  `[auto_mode] environment = [...]`.
- *Project facts* (deploy on push to main, prod hosts, domains): in the project's
  `.bridle/config.toml` as `[auto_mode] environment = [...]`.
- *Derived facts* (never typed): the generator adds "Trusted repo: <clone path> and its worktrees
  under <workspace>/wt and .bridle/state" from the resolved project paths.
- Merge order: derived, machine, project; `"$defaults"` first. A project may only *add* lines.
  Caveat to weigh at review: a project-committed `.bridle/config.toml` feeding the classifier is the
  repo-marks-itself-trusted hole Claude Code closed on purpose. It is acceptable only because
  bridle is the human's own tool and the human reviews the file, but it is a real trade. The
  conservative variant: project lines may only *tighten* (sensitive targets, "prod hosts") and
  trust lines (repos, domains, buckets) come from the machine file only. I recommend the
  conservative variant.

**Q4: staleness.** Nothing is stored, so nothing goes stale: derived paths come from the project's
resolved clone and workspace at launch. The machine file holds no workspace paths, only remotes and
domains, which don't move with workspaces. The print command regenerates on demand; the human
re-pastes after a move, and `claude auto-mode config` shows what is in effect.

### Smallest first slice

1. A pure function in `crates/bridle-daemon` `config.rs`: `auto_mode_environment(machine, project,
   paths) -> Vec<String>` (leading `"$defaults"`, derived line, machine, project) plus the two
   `[auto_mode]` config structs. Unit-tested.
2. `crates/bridle/src/session.rs`: add `autoMode.environment` to `orchestrator_settings` and
   `advisor_settings` output (alongside `with_layer_hooks`), tests next to
   `settings_are_valid_json`.
3. Docs: `docs/design/cli.md` (config keys), `docs/design/workflow-layers.md` or the agent-host
   doc that describes session settings, `CHANGELOG`.

Deferred, with reason: `bridle auto-mode print` (only matters for plain `claude`; add if the human
still hits blocks there); putting `autoMode` in the daemon's spawn `--settings` (command.rs) —
agents don't run auto mode today, so it would be dead config until a role is switched to
`--permission-mode auto`; revisit then, via the same function.

### Rejected alternatives

- **Bridle writes `~/.claude/settings.json`**: global file the human owns, Claude can't be the
  actor, merge/ownership of hand-edited lines is fragile, and it goes stale on every move.
- **A per-machine generated block pasted once**: the incident; stale the moment a workspace moves.
- **Project `.claude/settings.json` / `settings.local.json`**: ignored by Claude Code by design.
- **Managed settings**: org-scope, needs system paths and admin rights, overkill for one human.
- **Run `/auto-mode-setup` per project**: writes to the global file, scans one project, and each
  run re-describes only that project, which is the original incident.

### Migration for existing projects

No existing project's files change by default. The new behaviour is a launch-time overlay inside
`bridle session`, and with no `[auto_mode]` section the only addition is `"$defaults"` plus the
derived workspace line. Projects that want project lines add `[auto_mode]` to their own
`.bridle/config.toml` themselves, after the human reviews it (rule `existing-projects`; trials such
as track-web get it on the trial branch only). The human's global block stays as is; the one thing
to review is whether to trim its track-web-specific lines once the per-launch block covers them.

### Could not verify (no live run)

- That a `--settings` `autoMode` block is honoured by an interactive `claude` launched with `--remote-control`
  and our other flags, and that it combines with (rather than replaces) the user file's entries as
  the docs' "Entries from each scope are combined" implies. Check with `claude auto-mode config`
  under the same `--settings`, if that subcommand accepts the flag (docs don't say).
- Whether `--setting-sources project` (used for spawned agents) also drops `--settings` `autoMode`;
  irrelevant to sessions (no such flag there) but matters for the deferred spawn case.
- The minimum Claude Code version for `"$defaults"` and the exact semantics of the "Trusted repo"
  entry when the working directory is a worktree under `.bridle/state` (the doc says working
  directory and configured remotes; worktrees of one repo share remotes, unconfirmed for the
  classifier).
- Whether the interactive sessions are actually in auto mode: `session.rs` passes no
  `--permission-mode`; the human's own default mode decides.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). See "Recommendation" above: per-launch --settings in `bridle session`, no global-file writes; built as br-fc9a (ticket fc9a).
