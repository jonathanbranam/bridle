# Spike 08 findings: what a spawned agent's first turn costs, and which flags trim it

Claude Code version: **2.1.284**   Date: 2026-09-29   Usage: Haiku (`claude-haiku-4-5-20251001`), 85
short runs, **$0.88** in total (`total_cost_usd` summed over the runs).

Ticket [[lean-starting-context-for-every-agent-ct8m|ct8m]] (Plan step 1).
Every run was a scratch dir outside the repo (`/tmp/lean-scratch/repo`, a `git archive HEAD` of this
repo, re-`git init`ed), built exactly as bridle builds a spawn (`crates/bridle-claude/src/command.rs`
`args()`, role permissions from `crates/bridle-daemon/src/config.rs`, the `--settings` JSON with the
worker's `Stop` hook) with a real `system-prompt.md` copied from a live worker, manager and
product-manager (6.1, 7.9 and 6.7 KB). `CLAUDE*` and `BRIDLE_TOKEN` were stripped from the env like
bridle does. One user message ("Reply with the single word OK"), then the control request
`get_context_usage` (the same probe the supervisor sends at turn end, kc4v) and read `totalTokens`
and `categories`. No live orchestrator or advisor session and not `~/.claude/settings.json` were
touched (`--setting-sources project` keeps it out).

## Verdict

A spawned agent's first turn is about **19-20K tokens** on 2.1.284, not the 48-60K `bridle agents`
shows, and the ticket's premise that every built-in tool's definition is in each agent's context is
**mostly wrong**: 11K of them are *deferred* (names only, about 130 tokens in the prompt) and
`totalTokens` doesn't count them. What does cost is the loaded set (Bash, Read, Edit, Write, Glob,
Grep, **Agent 2.1K**, Skill, ToolSearch) plus the **skill listing (~3K)**. **`--tools <whitelist>`
is the one flag that matters**: it takes a worker to 12.8K (-33%), a manager to 12.6K (-36%), and to
15.4K (-22%) if the manager keeps Agent. The bare-name denies and the settings keys save 1-2K
between them and add nothing once `--tools` is passed. `--strict-mcp-config` **does** block the
claude.ai connectors; `disableClaudeAiConnectors` is not needed for spawned agents.

## Starting context per role (total tokens at the first turn)

Variants: **A** today's flags. **B** A + `--tools`. **C** A + bare-name `--disallowedTools` for
EnterPlanMode ExitPlanMode DesignSync NotebookEdit SendMessage PushNotification RemoteTrigger
ReportFindings ScheduleWakeup AskUserQuestion CronCreate CronDelete CronList Workflow. **D** A +
`--settings` keys `disableBundledSkills disableWorkflows disableClaudeAiConnectors disableArtifact`.
**E** B + D. **F** E with `Agent` added to `--tools`. **G** C + D.

| role | A today | B `--tools` | C bare deny | D keys | E tools+keys | F E + Agent | G bare+keys |
|---|---|---|---|---|---|---|---|
| worker (`Bash,Read,Edit,Write,Glob,Grep`) | 19,140 | 12,800 | 18,516 | 17,541 | **12,800** | 15,534 | 16,917 |
| manager (`Bash,Read,Glob,Grep`) | 19,682 | 12,625 | 19,058 | 18,083 | **12,625** | 15,359 | 17,459 |
| product-manager (`Bash,Read,Glob,Grep`) | 19,384 | 12,327 | 18,760 | 17,785 | **12,327** | 15,061 | 17,161 |

Where the worker's 6,340 tokens (A to B) go, by `get_context_usage` category:

| category | A | B | change |
|---|---|---|---|
| System prompt | 5,007 | 4,726 | -281 |
| System tools (loaded) | 8,521 | 6,174 | -2,347 |
| Skills | 1,556 | 0 | -1,556 |
| Messages | 2,925 | 769 | -2,156 |
| Memory files (CLAUDE.md) | 1,131 | 1,131 | 0 |
| System tools (deferred, not in the total) | 11,343 | 0 | |

`Messages` is mostly attachments: `skill_listing` 1,646, `agent_listing_delta` 469,
`deferred_tools_delta` 132. Dropping the `Skill` tool from `--tools` removes the skill listing;
dropping `Agent` removes the agent listing. The `system-prompt.md` bridle appends is in "System
prompt": with no tools at all (`--tools ""`) the whole first turn is **6,314** (system prompt 4,728,
CLAUDE.md 1,131, messages 455), so about 1.5K of the system prompt is bridle's role file.

Read this as a floor, not the whole story: this is a first turn that does nothing. Real agents show
48-60K after a turn that read the role docs, ran `bridle prime` and so on. This spike can't say how
much of the gap is real work and how much is something it didn't reproduce (Sonnet instead of
Haiku, the live repo state). Step 6 should log the turn-1 `get_context_usage` breakdown per agent.

### Each flag on its own (worker)

| flag | total | vs A | what happened |
|---|---|---|---|
| `disableBundledSkills` | 17,541 | -1,599 | init `skills` 18 to 2 (`design`, `doctor`); skill listing gone from Messages (the 1,556 moves into "System tools", the net saving is the listing) |
| `skillOverrides` (every skill `"off"`) | 17,669 | -1,471 | `Skills` 1,556 to 102; same idea, needs each name listed, so it goes stale as Claude Code adds skills |
| `disableWorkflows` | 19,086 | -54 | drops the `workflow-authoring` skill line; the Workflow tool is already denied by bridle |
| `disableClaudeAiConnectors` | 19,140 | 0 | nothing to remove: `--strict-mcp-config` already blocks them (below) |
| `disableArtifact` | 19,140 | 0 | there is no Artifact tool in a `-p` session |
| `disableRemoteControl` (reference) | 19,140 | 0 | nothing in `-p`; it matters only for the interactive orchestrator/advisor, which must keep Remote Control |

All six keys exist in 2.1.284: each name appears in the binary's strings
(`~/.local/share/claude/versions/2.1.284`; 11, 14, 19, 5, 28 and 13 occurrences of
`disableBundledSkills`, `disableWorkflows`, `disableClaudeAiConnectors`, `disableArtifact`,
`skillOverrides`, `disableRemoteControl`), and the first two and `skillOverrides` visibly change
the context above. `disableClaudeAiConnectors` was also seen working (below).

## Tool definition sizes

Each row is `--tools <name>` alone on a worker spawn, "System tools" tokens. Every single-tool run
carries roughly 0.4K of fixed overhead (the six worker tools sum to 8,694 alone but 6,174 together),
so subtract about 0.4K for the marginal cost. The ranking is what matters.

| tokens | tools |
|---|---|
| 6,792 | Workflow |
| 3,458 | Bash |
| 3,138 | DesignSync |
| 2,708 | Monitor |
| 2,646 | Agent (`Task` in the init list) |
| 2,050 | SendMessage |
| 1,759 / 1,681 | ScheduleWakeup / CronCreate |
| 1,599 | EnterWorktree |
| 1,561 | Grep |
| 1,555 | TaskUpdate |
| 1,548 | RemoteTrigger |
| 1,214 / 1,212 | ExitWorktree / TaskCreate |
| 1,192 | Read |
| 1,113 | ReportFindings |
| 1,096 / 1,027 | WebSearch / WebFetch |
| 980 / 962 | NotebookEdit / PushNotification |
| 944 | Edit |
| 909 | ToolSearch |
| 825 / 793 / 784 / 734 | TaskList / TaskGet / ListAgents / TaskStop |
| 781 / 758 | Write / Glob |
| 615 / 580 | CronDelete / CronList |

Not available in a `-p` session at all (absent from `init.tools` and `--tools` gave an empty
list): **AskUserQuestion, EnterPlanMode, ExitPlanMode, TaskOutput, Artifact**. Denying them in a
background agent is a no-op; the ticket's worry about AskUserQuestion is only for the interactive
sessions. `Skill` appears in `init.tools` but shows no size of its own (its cost is the skill
listing).

With today's flags the loaded set is the six core tools + Agent + Skill + ToolSearch (8,521); the
other 15 (DesignSync, Monitor, NotebookEdit, PushNotification, ReportFindings, EnterWorktree,
ExitWorktree, ListAgents, Task*, WebFetch, WebSearch) are deferred. Bridle's existing bare-name
denies (SendMessage, Workflow, ScheduleWakeup, Cron*, RemoteTrigger) already remove their
definitions: with none denied the deferred set is 15,317, with them 11,343. Variant C's extra
denies (DesignSync, NotebookEdit, PushNotification, ReportFindings) took another 3.6K out of the
*deferred* set and only 0.6K out of the total. A deferred tool costs its definition only when the
agent asks for it through ToolSearch (`select:Monitor` in the transcripts below).

## Do the claude.ai connectors get in?

| launch | `mcp_servers` in `init` | extra context |
|---|---|---|
| bridle today (`--strict-mcp-config`) | none | 0 |
| no `--strict-mcp-config` | `claude.ai Claude Docs`, `claude.ai Google Drive`, `claude.ai Gmail`, `claude.ai Google Calendar` | MCP instructions 538 + MCP tools (deferred) 1,726; the total rose 220 |
| no strict, `disableClaudeAiConnectors: true` | none | 0 |

**`--strict-mcp-config` blocks the connectors**, so a spawned agent already has none, and
`disableClaudeAiConnectors` is not needed there (harmless to add). It matters only for a launch
without `--strict-mcp-config`, i.e. the interactive orchestrator and advisor if step 4 doesn't
add the strict flag. The connectors' up-front cost is small (0.2K in the total, 1.7K deferred).

## Tool use in real transcripts

`.bridle/agents/*/transcript.jsonl` for all 225 agents with a recorded role, `tool_use` blocks,
grouped by the role in the agent's `system-prompt.md` (counts are calls, then agents using it).
Most are from before bridle's bare-name denies existed, so ScheduleWakeup and SendMessage use is
history, not a need.

| role (agents) | tool use |
|---|---|
| worker (222) | Bash 6,337 calls (all 222 agents; 487 calls were `bridle ...`), Read 1,613 (141), Edit 1,398 (126), Grep 390 (71), **Monitor 98 (45)**, ScheduleWakeup 76 (18), **ToolSearch 62 (45)**, Write 58 (35), Glob 34 (22), TaskStop 11 (8), SendMessage 5 (3), WebSearch 5 (2), Skill 3 (3), **Agent 3 (3)**, ListAgents 1, TaskGet 1 |
| manager (2) | Bash 3,116, Grep 196, Read 175, ScheduleWakeup 69, **Agent 13 (2)**, Glob 12, ToolSearch 9, ListAgents 4, Write 2, Edit 2, Monitor 1 |
| product-manager (1) | Bash 843, Read 234, Grep 168, ScheduleWakeup 25, **Agent 5**, Glob 26, Edit 1, ListAgents 1, ToolSearch 1 |

- **Monitor**: workers (20% of them) used it to wait on a background `just check`, always after a
  `ToolSearch select:Monitor` (the sampled ToolSearch calls were `select:Monitor` or `select:SendMessage`). 31 of the 98 calls
  errored with an `InputValidationError`. The same wait works without it: a worker spawn with only
  the six core tools ran `sleep 3; echo BGDONE` with `run_in_background`, read the completion, and
  ran `bridle --version` (run `func_E`; $0.017).
- **Agent**: 3 of 222 workers, but both managers and the product-manager use it for `Explore`
  subagents (13 and 5 calls).
- **ToolSearch**: only ever used to load Monitor, SendMessage or to search for a tool. With a
  `--tools` whitelist nothing is deferred, so it has no job.
- **Skill**: 3 workers loaded `bridle-worker` (from the human's `~/.claude`, which a
  `--setting-sources project` spawn no longer sees). This repo has no project skills, so `Skill`
  has nothing to run for a spawned agent.
- **WebSearch**: 5 calls in 2 workers, not in any role's allowed list, so denied anyway.
- **Permissions still apply under `--tools`**: with the six-tool whitelist and a `Bash(uname *)`
  deny, `uname -s` was refused ("Permission ... has been denied", run `func_deny`), so
  `--tools` narrows what exists and `--allowedTools`/`--disallowedTools` still gate the calls.

## Recommended tools per role

| role | `--tools` | why | first turn |
|---|---|---|---|
| worker | `Bash,Read,Edit,Write,Glob,Grep` | the only tools with real use. Drop Agent (3 of 222 workers, 2.1K), Monitor (use `run_in_background` on Bash), ToolSearch, Skill | 12.8K (from 19.1K) |
| manager | `Bash,Read,Glob,Grep,Agent` | Agent for `Explore` subagents (13 calls); no Edit/Write (its permission mode wouldn't allow them anyway), no Monitor (1 call) | 15.4K (from 19.7K); 12.6K without Agent |
| product-manager | `Bash,Read,Glob,Grep,Agent` | same as the manager; Agent for research subagents | 15.1K (from 19.4K); 12.3K without Agent |

Also: the settings keys are redundant once `--tools` is passed (E equals B, since `Skill` is already
gone). Add `disableBundledSkills` only for a role that keeps `Skill`. The existing bare-name denies
stay as belt and braces (they are cheap and also cover a role configured with a wider `tools`
list), but they are no longer where the saving comes from. Adding `Agent` back costs 2.7K in total
(F minus E: tools 2,142, Messages 446, system prompt 146).

The orchestrator and advisor need a decision at step 4: they legitimately use Agent, ToolSearch
(Monitor for waits until fx7x's wait command replaces it), Cron/ScheduleWakeup (heartbeat) and web
tools, so they are not covered by the lists above.

## Not measured

- **The interactive orchestrator and advisor launch shape.** A `-p` run can't reproduce it: the TUI
  session has tools `-p` doesn't (AskUserQuestion, plan mode) and loads the human's
  `~/.claude/settings.json`, plugins, hooks and user CLAUDE.md, which this spike must not read or
  run. A scratch tmux session (like spike 07) would still load the human's user settings. The ticket's
  own figure is 50K at the first turn (9K of it `bridle prime`). Measure it at step 4 with an
  orchestrator that has just been restarted, by `/context` in the live pane, by the human or the
  orchestrator.
- Sonnet (the roles' model) instead of Haiku. Tool and prompt tokens are the same text; the
  tokenizer counts may differ slightly, and the tool descriptions do not depend on the model in
  what was observed.
- Whether deferred tools cost anything at the moment ToolSearch loads them (not needed if
  `--tools` leaves nothing deferred).
- The gap between 19K here and 48-60K in `bridle agents`: explained in [the gap section](#the-gap-19k-first-turn-vs-48-60k-live-plan-step-3) below.

## Fixtures

Not kept. The harness was a 60-line Python script in `/tmp/lean-scratch` (spawns `claude` with the
argv above, sends one message, then `get_context_usage`); the per-run JSON (categories, init tools,
skills, mcp servers) is reproducible in a few cents per run.

## The gap: 19K first turn vs 48-60K live (plan step 3)

Task br-5c6c. Docs only, no live run: the daemon already records everything needed. Source:
`.bridle/agents/*/transcript.jsonl` (each has the spawn line, the stream-json events with per-call
`usage`, and every tool result) and `system-prompt.md`. The 77 most recent workers with more than 5
turns (mean 27 turns), plus the two long-lived sessions `manager-2` (`a-qx8ud`) and `pm-1`
(`a-fs6oq`). Context at a call is `input + cache_creation + cache_read` tokens of the call's
`usage`, which is what `bridle agents` shows as `context_tokens` (last turn). Tokens below are
`bytes / 2.7`, the ratio that reconciles one worker's summed content (215 KB) with its measured growth
(80K); the split is an estimate, the totals are not.

### Verdict

**There is no unexplained fixed cost.** The 48-60K is the ordinary context of a worker part-way
through its task; only about 17K of it is the fixed start.

- A real Sonnet worker's first call is **17.5K** (mean over 77; 17.1K for most, 19.7K for the few
  that also carried a longer first message). That agrees with spike 08's 19K for Haiku with the
  same flags (`get_context_usage` and the API's `usage` differ by a couple of K; not chased).
- It then grows **~1.4K per turn** and ends at a median **53K** (mean 55K) after ~27 turns. That
  is the "48-60K". Nothing else is added at start: the first call's `usage` already contains the
  system prompt, tools, skills, CLAUDE.md and the first message, and the later calls only add what
  the agent itself produced or read.
- `manager-2` (545 turns) and `pm-1` (224) show 88K and 103K: the same growth over more turns, not
  a bigger start.

### One real fixed-cost finding: `manager-2` and `pm-1` started at 32K, not 17K

Their first call is **32.7K and 32.1K**. Their spawn lines have no `--setting-sources project`,
`--tools` or `--disallowedTools`: they predate `c8f5baa` (2026-09-28 01:47 -04; manager-2 started
09-27 23:45, pm-1 01:05). Their `init` shows the human's setup leaking in: 4 plugins
(skill-creator, rust-analyzer-lsp), **31 skills** (`anthropic-skills:*`, `deep-research`, ...), 67
slash commands, 28 tools including `LSP`. That is the missing ~15K, and it goes away on a
restart of those roles with today's `command.rs`. Every worker in the sample from 09-28 on is at
17K.

### Where a worker's 55K goes (mean of 77 workers, at the final call)

| part | tokens | notes |
|---|---|---|
| fixed start (first call) | 17.5K | system prompt 5.0K (bridle's role file ~1.5K of it), tools 8.5K, skill/agent listings ~2.1K, CLAUDE.md 1.1K, first message ~0.7K |
| the agent's own tool calls (`tool_use` input) | ~8.4K | heredocs, `sed -n` ranges, commit messages, task summaries |
| thinking kept in context | ~6.2K | over-counted: the transcript stores the signature too |
| source read through Bash (`cat`, `sed -n`) | ~6.0K | |
| docs read through Bash | ~5.1K | |
| `Read` of source / docs / other | ~2.7K / 1.1K / 1.3K | |
| `just check` / cargo output | ~1.9K | |
| git output | ~1.7K | |
| CLAUDE.md, role file or rules re-read | ~1.1K | |
| `bridle task show` | ~0.8K | one call in most tasks, the brief |
| the agent's text, other Bash, `bridle ...`, grep | ~2.4K | |

Sum ~56K against a measured mean of 55.3K. Reading (source, docs, output) is ~19K, what the
agent writes and thinks ~15K, the fixed start 17.5K. `bridle prime` is never run by a worker
(0 calls); its 12.5K output for `worker` (`bridle prime worker` today) exists but is not in their
context. The ticket's 9K of `bridle prime` is an orchestrator figure.

### Which files cost most (workers, whole-file reads, mean per read)

| file | reads | bytes/read | note |
|---|---|---|---|
| `crates/bridle/src/cli.rs` | 21 | 13 KB | whole file when the worker needed one subcommand |
| `CHANGELOG.md` | 11 | 15 KB | whole file read to add one entry |
| `docs/design/cli.md` | 7 | 18 KB | |
| `crates/bridle/src/commands.rs` | 20 | 6 KB | |
| `docs/design/agent-host/*.md` (several `cat` in one command) | many | 17-27 KB per command | |

Big multi-file `cat` commands (15-27 KB, ~6-10K tokens each) are the single largest items: one such
command costs more than the whole role file and CLAUDE.md together.

### Repeated text between preamble, role file, CLAUDE.md

Small. Checked `system-prompt.md` (6.1 KB, worker) against `CLAUDE.md` (3.8 KB): repeated is the
`just check` / `just check-affected` line, "no memory", and the pointer to docs; about 250 tokens
in total. `Don't commit unless asked` in CLAUDE.md even contradicts the worker prompt, which says to
commit; it costs nothing but is a trap. Not worth a task on its own.

### Ranked recommendations (tokens saved per agent, worker unless said)

1. **Restart `manager-2`/`pm-1` (and any other long-lived role) under today's flags:** ~15K each at
   start (32K to 17K). No code change; takes effect at their next spawn. Interactive orchestrator
   and advisor stay separate (step 4).
2. **`--tools` whitelist (spike section above):** ~4.7K per worker/manager start (19.1K to 12.8K
   measured, and the worker's first call today is 17.5K). Already the plan's step 2.
3. **Tell workers to read by range, not whole file:** `CHANGELOG.md` (`head -30`, entries go on
   top), `sed -n` for `cli.rs`/`commands.rs`, one design doc rather than the folder. Whole-file
   reads of these four files alone are ~4-6K tokens per worker that touches them; a line in the
   worker role file or a `docs/` index that says which doc covers what saves the `cat docs/design/*`
   shotgun (~5K per worker). Estimated ~3-5K.
4. **Keep the brief short and a pointer:** `bridle task show` is only 0.8K, so nothing to gain here;
   the cost is the ticket and docs the brief points at. A brief that names the sections
   (`file:lines`) instead of the file saves what item 3 saves.
5. **Cap chatty output:** `just check` and cargo output ~1.9K, git ~1.7K per worker; `| tail -n 30`
   in the worker role's check instruction saves ~1K.
6. **Long-lived roles (manager, pm): compact or restart on a schedule.** They grow ~1.4K per turn
   with no ceiling (88K and 103K); nothing at start explains it. Out of scope for a start-context
   trim, but it is the largest single number in `bridle agents`. Related: the step-6 logging.
7. **De-duplicate CLAUDE.md against the role prompt (~250 tokens) and drop the "Don't commit unless
   asked" line for agents:** cosmetic, last.

The system prompt itself (5.0K, ~1.5K of it bridle's) is not worth trimming: cutting the role
file in half saves ~0.8K, less than any of items 1-3.

## Step 4: the orchestrator and advisor launchers (task br-72da)

`scripts/claude-orchestrator` and `scripts/claude-advisor` now pass `--strict-mcp-config` and one
`--settings` object with `disableBundledSkills`, `disableWorkflows`, `disableClaudeAiConnectors`,
`disableArtifact` and `permissions.deny` for EnterPlanMode, ExitPlanMode, DesignSync, NotebookEdit,
PushNotification, ReportFindings, RemoteTrigger, Artifact and Workflow. The orchestrator's
object also carries the `SessionStart` hook (a second `--settings` would replace it, so it is one
object). Kept on purpose: AskUserQuestion (the human's decision), Agent, SendMessage (~2K; the human,
2026-09-29: both roles run subagents and talk to them), ToolSearch, WebSearch/WebFetch,
Monitor and Bash background runs, the Cron tools and ScheduleWakeup (heartbeat), and
`--remote-control` (no `disableRemoteControl`). Not `--tools`: these roles use too much of the set.

Scratch check, Haiku, `-p`, the harness above, `--setting-sources project`, unrestricted tools, the
manager prompt, one "OK" turn, $0.055 in all. Before is the old launch flags (no strict, no keys);
after is the new set. It is a `-p` approximation: the live TUI also has AskUserQuestion and loads the
human's user settings, so the live number is higher; `/context` after a restart gives the real one.

| | before | after |
|---|---|---|
| total tokens at first turn | 22,777 | **18,838** (-3,939, -17%) |
| System tools (loaded) | 11,397 | 10,830 |
| System tools (deferred, not in total) | 15,317 | 9,139 |
| Skills / skill listing | 1,556 | 0 (skills 18 to 2) |
| MCP instructions + deferred MCP tools | 538 + 1,726 | 0 (4 claude.ai connectors gone) |
| Messages | 2,606 | 1,328 |

Not measured here: `bridle prime orchestrator` (~9K, step 5's state-file shrink) is not in this number.
