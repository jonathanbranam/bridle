# Agent harness / orchestrator name catalogue

Purpose: for `docs/questions/open/a-new-name-for-the-project-geem.md`. Collects the names
of existing agentic harnesses, coding-agent CLIs, multi-agent orchestrators and agent
frameworks, so a new name for bridle can be checked against them. It does not propose
names; choosing is the human's.

Checked against the web on 2026-09-28. "Verified" means one of:

- **yes 2026-09-28**: a search or fetch that day found the project's repo, docs or
  launch coverage, and it matched the row.
- **listed 2026-09-28**: seen only as an entry in
  [awesome-agent-orchestrators](https://github.com/andyrewlee/awesome-agent-orchestrators)
  (a curated list of 200+ orchestrators). The repo was not opened on its own.

This space moves weekly. A name missing from here is not proof it is free: search
GitHub, npm, crates.io, Homebrew and PyPI before settling on one.

**Direct collision:** `bridle` itself is already taken in this space by
[neiii/bridle](https://github.com/neiii/bridle), a Rust TUI/CLI that manages config for
agent harnesses (Amp, Claude Code, OpenCode, Goose, Copilot CLI, Crush, Droid). It ships
via Homebrew and Cargo.

## Steve Yegge's projects

| Name | What it is | Link | CLI command | Verified |
|---|---|---|---|---|
| Gas Town | Yegge's open-source multi-agent workspace manager (Go, tmux). Runs 20-30 parallel coding agents (Claude Code, Copilot and others) in named roles (Mayor, Polecats, Refinery, Witness, Deacon, Dogs, Crew) across project "rigs", with a merge queue. Launched Jan 2026 ("Welcome to Gas Town", Medium). | https://github.com/gastownhall/gastown (was steveyegge/gastown) | `gt` | yes 2026-09-28 |
| Wasteland | A federated work network for Gas Town. Linked "towns" post wanted items, claim each other's work and earn reputation "stamps", through DoltHub. | https://github.com/gastownhall/gastown | *(part of `gt`)* | yes 2026-09-28 |
| Gas City | "Orchestration-builder SDK" pulled out of Gas Town: formulas, orders, convoys of beads, rigs, runtime providers, and a declarative city config. | https://github.com/gastownhall/gascity | *(unconfirmed)* | yes 2026-09-28 |
| Wheelhouse | Yegge's **private, closed-source** orchestrator for his MMO Wyvern. Runs about 50-60 agents with 18 named "officer" roles; mostly bash plus elisp, built on Beads. Described in "The Shape of Things to Come, Part 1" (Aug 2026), where he gives up reusable harnesses because Gas Town "fell apart". | https://yegge.ai/essays/the-shape-of-things-to-come/ | *(none public)* | yes 2026-09-28 |
| Beads | Yegge's git-backed, dependency-aware graph issue tracker, used as memory for coding agents (now Dolt-backed; hash IDs like `bd-a1b2`). Gas Town, Gas City and Wheelhouse are all built on it. | https://github.com/gastownhall/beads (was steveyegge/beads) | `bd` | yes 2026-09-28 |
| beads_rust | Rust port of Beads by Jeffrey Emanuel (Dicklesworthstone): SQLite plus JSONL export, never runs git, has an MCP serve mode. | https://github.com/Dicklesworthstone/beads_rust | `br` | yes 2026-09-28 |

## Coding-agent CLIs and single-agent harnesses

| Name | What it is | Link | CLI command | Verified |
|---|---|---|---|---|
| Claude Code | Anthropic's agentic coding CLI (what bridle drives). | https://code.claude.com | `claude` | yes 2026-09-28 |
| Claude Code Agent Teams | Experimental Claude Code feature: a lead session coordinates teammate sessions through a shared task list and messaging (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`). | https://code.claude.com/docs/en/agent-teams | *(inside `claude`)* | yes 2026-09-28 |
| OpenAI Codex CLI | OpenAI's terminal coding agent; also the Codex cloud agent and `codex-action`. | https://github.com/openai/codex | `codex` | yes 2026-09-28 |
| Gemini CLI | Google's open-source terminal agent (ReAct loop, MCP). | https://github.com/google-gemini/gemini-cli | `gemini` | yes 2026-09-28 |
| Qwen Code | Alibaba's fork of Gemini CLI, tuned for Qwen3-Coder. | https://github.com/QwenLM/qwen-code | `qwen` | yes 2026-09-28 |
| GitHub Copilot CLI | Copilot's coding agent in the terminal (GA Feb 2026; Plan and Autopilot modes). | https://github.com/github/copilot-cli | `copilot` | yes 2026-09-28 |
| Cursor CLI | Cursor's agent in the terminal, including a headless mode for CI. | https://cursor.com/cli | `cursor-agent` (**command name not confirmed**) | yes 2026-09-28 |
| Kiro CLI | AWS's Kiro agents in the terminal (replaces the Amazon Q Developer CLI). | https://kiro.dev/cli/ | `kiro-cli` | yes 2026-09-28 |
| Amp | Sourcegraph's (now ampcode) coding agent, for the CLI and IDEs. | https://ampcode.com | `amp` | yes 2026-09-28 |
| Factory Droid | Factory's agent CLI, desktop app and cloud; "Missions" run many agents at once. | https://docs.factory.ai/cli/getting-started/overview | `droid` | yes 2026-09-28 |
| OpenCode | Open-source terminal coding agent from SST/Anomaly (build and plan agents). | https://github.com/anomalyco/opencode | `opencode` | yes 2026-09-28 |
| Crush | Charm's terminal coding agent (Go), by the original OpenCode author. | https://github.com/charmbracelet/crush | `crush` | yes 2026-09-28 |
| Cline | Autonomous coding agent: a VS Code extension plus CLI 2.0, with parallel agents and a `--kanban` board. | https://github.com/cline/cline | `cline` | yes 2026-09-28 |
| Roo Code | Cline fork for VS Code. **Shut down on 15 May 2026**; the team moved to "Roomote", a cloud agent that works from Slack. | https://docs.roocode.com/sunset | *(none)* | yes 2026-09-28 |
| Kilo Code | VS Code agent plus an open-source CLI. | https://kilo.ai/cli | `kilo` | yes 2026-09-28 |
| Goose | Open-source extensible agent from Block, donated to the Linux Foundation's Agentic AI Foundation (AAIF) in 2026. | https://github.com/aaif-goose/goose | `goose` | yes 2026-09-28 |
| Aider | AI pair programming in the terminal; every edit is a git commit. | https://github.com/Aider-AI/aider | `aider` | yes 2026-09-28 |
| Plandex | Terminal agent for large multi-step tasks. The managed cloud closed in Nov 2025; the self-hosted version is in maintenance mode. | https://github.com/plandex-ai/plandex | `plandex` | yes 2026-09-28 |
| Open Interpreter | A local terminal coding agent, rewritten in Rust in 2026; it can switch between harnesses. | https://www.openinterpreter.com | `interpreter` | yes 2026-09-28 |
| OpenHands | Open-source software-engineering agent platform (formerly OpenDevin): CLI, headless, web, and cloud. | https://github.com/OpenHands/OpenHands | `openhands` | yes 2026-09-28 |
| SWE-agent / mini-swe-agent | Princeton/Stanford agents that fix GitHub issues; mini-swe-agent is the recommended one now. | https://github.com/SWE-agent/mini-swe-agent | `sweagent`, `mini` | yes 2026-09-28 |
| Continue CLI | Continue's coding agent in the terminal. | https://docs.continue.dev/cli/overview | `cn` | yes 2026-09-28 |
| Codebuff | CLI agent that splits work across File Picker, Planner, Editor and Reviewer agents (the free tier is "Freebuff"). | https://github.com/CodebuffAI/codebuff | `codebuff` | yes 2026-09-28 |
| Pi | Mario Zechner's minimal coding agent and toolkit (pi-mono). | https://github.com/badlogic/pi-mono | `pi` | yes 2026-09-28 |
| Warp | Terminal turned "Agentic Development Environment", open-sourced in 2026. Its cloud agent orchestration platform is called **Oz**. | https://github.com/warpdotdev/warp | `warp` (app) | yes 2026-09-28 |
| Devin | Cognition's hosted, asynchronous "AI software engineer" that works through PRs. | https://cognition.ai | *(none known)* | yes 2026-09-28 |
| Jules | Google's asynchronous cloud coding agent; its CLI is "Jules Tools". | https://jules.google | `jules` | yes 2026-09-28 |

## Multi-agent orchestrators and harnesses for coding agents

| Name | What it is | Link | CLI command | Verified |
|---|---|---|---|---|
| Claude Squad | Go TUI that runs Claude Code, Codex, Gemini, Aider and others, each in a tmux session with its own git worktree. | https://github.com/smtg-ai/claude-squad | `cs` | yes 2026-09-28 |
| Ruflo (formerly Claude Flow) | ruvnet's swarm / "hive-mind" orchestration platform for Claude Code and Codex. Calls itself "the original agent harness". | https://github.com/ruvnet/ruflo | `npx claude-flow` (legacy), `ruflo` | yes 2026-09-28 |
| Conductor | Mac app from Melty Labs that runs parallel Claude Code or Codex agents, each in a worktree, with diff review. | https://conductor.build | *(desktop app)* | yes 2026-09-28 |
| Microsoft Conductor | Orchestrates Copilot and Claude from YAML workflows. | https://github.com/microsoft/conductor | *(CLI + web)* | yes 2026-09-28 |
| Code Conductor | GitHub-native CLI; agents claim tasks through issue labels. | https://github.com/ryanmac/code-conductor | *(CLI)* | yes 2026-09-28 |
| Sculptor | Imbue's desktop app for parallel coding agents, each in its own Docker container. | https://github.com/imbue-ai/sculptor | *(desktop app)* | yes 2026-09-28 |
| Crystal / Nimbalyst | Stravu's Electron app for parallel Claude Code and Codex sessions in worktrees. Crystal was deprecated in Feb 2026 in favour of Nimbalyst. | https://github.com/stravu/crystal, https://nimbalyst.com | *(desktop app)* | yes 2026-09-28 |
| Vibe Kanban | BloopAI's kanban board (Rust/React) for Claude Code, Codex, Gemini and others, each in its own worktree. Now sunsetting and community-maintained. | https://github.com/BloopAI/vibe-kanban | `npx vibe-kanban` | yes 2026-09-28 |
| uzi | Devflow's CLI for running many agents in parallel with worktrees, tmux and dev-server ports. | https://github.com/devflowinc/uzi | `uzi` | yes 2026-09-28 |
| Agent Orchestrator (Composio / Untrivial) | Plans tasks, spawns agents in worktrees and fixes CI failures and merge conflicts autonomously. | https://github.com/ComposioHQ/agent-orchestrator | `ao` (npm `@aoagents/ao`) | yes 2026-09-28 |
| CLI Agent Orchestrator (CAO) | AWS Labs: a supervisor agent with workers in tmux sessions (Claude Code, Kiro, Codex and others). | https://github.com/awslabs/cli-agent-orchestrator | `cao`, `cao-server` | yes 2026-09-28 |
| Symphony | OpenAI's open spec and Elixir reference for turning a Linear board into a control plane for Codex agents. | https://github.com/openai/symphony | *(service)* | yes 2026-09-28 |
| Superset | Editor or terminal for running many coding agents in parallel. | https://superset.sh | *(app)* | yes 2026-09-28 |
| Emdash | Electron "agentic development environment" with 30+ CLI providers, each running in parallel in its own worktree. | https://github.com/generalaction/emdash | *(desktop app)* | yes 2026-09-28 |
| ccmanager | TUI session manager for Claude Code, Codex, Gemini, Cursor, Copilot and others, with worktrees. | https://github.com/kbwo/ccmanager | `ccmanager` | yes 2026-09-28 |
| ccswarm | Rust multi-agent orchestration for Claude Code with worktrees and YAML flows (plan, then consensus, then implement, then review). | https://github.com/nwiizo/ccswarm | `ccswarm` | yes 2026-09-28 |
| claude-swarm (affaan-m) | Breaks tasks into parallel subtasks and shows the swarm in a TUI; built on the Claude Agent SDK. | https://github.com/affaan-m/claude-swarm | *(unconfirmed)* | yes 2026-09-28 |
| Claude Code Agent Farm | Runs 20-50 Claude Code agents in tmux, coordinated with locks. | https://github.com/Dicklesworthstone/claude_code_agent_farm | *(script)* | yes 2026-09-28 |
| Tmux-Orchestrator | Claude agents in tmux that schedule their own check-ins, with PM and engineer roles (many forks). | https://github.com/Jedward23/Tmux-Orchestrator | *(scripts)* | yes 2026-09-28 |
| Baton | Python daemon that polls GitHub Issues and sends Claude Code into worktrees. | https://github.com/mraza007/baton | `baton` | yes 2026-09-28 |
| Bernstein | Pipeline from planning to merge: deterministic scheduling, task graphs, checks before merge. | https://github.com/sipyourdrink-ltd/bernstein | `bernstein` | yes 2026-09-28 |
| Agent Kanban | VS Code extension with markdown-backed kanban lanes for Copilot Chat. | https://github.com/appsoftwareltd/vscode-agent-kanban | *(extension)* | yes 2026-09-28 |
| amux | Minimal TUI that spawns parallel agents in worktrees (the author also writes "harness engineering" guides). | https://github.com/andyrewlee/amux | `amux` | listed 2026-09-28 |
| agent-deck | A single TUI with live status across coding agents. | https://github.com/asheshgoplani/agent-deck | *(TUI)* | listed 2026-09-28 |
| Agent of Empires | TUI with a web view for phones. | https://github.com/agent-of-empires/agent-of-empires | *(TUI)* | listed 2026-09-28 |
| dmux | Multiplexer for dev agents, one worktree each. | https://github.com/standardagents/dmux | `dmux` | listed 2026-09-28 |
| cmux | Ghostty-based macOS terminal for agents. | https://github.com/manaflow-ai/cmux | `cmux` | listed 2026-09-28 |
| herdr | Background runtime that owns agent terminals and survives reboots. | https://github.com/herdrdev/herdr | `herdr` | listed 2026-09-28 |
| Vigil | macOS terminal where a manager agent spawns workers. | https://github.com/butterlatte-zhang/vigil | *(app)* | listed 2026-09-28 |
| YYLO | CLI orchestrator with task validation and merge queues. | https://github.com/yylo-dev/yylo | *(CLI)* | listed 2026-09-28 |
| Orca | Stably's desktop and mobile agentic development environment. | https://github.com/stablyai/orca | *(app)* | listed 2026-09-28 |
| Paseo | Self-hosted daemon with desktop, mobile, web and voice clients. | https://github.com/getpaseo/paseo | *(daemon)* | listed 2026-09-28 |
| Runner | macOS/Windows app that runs agents as a "crew". | https://github.com/yicheng47/runner | *(app)* | listed 2026-09-28 |
| t3code | Control surface for harnesses on web, mobile and desktop. | https://github.com/pingdotgg/t3code | *(app)* | listed 2026-09-28 |
| Berd | Block's open-source desktop app for AI agents. | https://github.com/block/berd | *(app)* | listed 2026-09-28 |
| OpenRig | An agent team defined in YAML and run in tmux. | https://github.com/mvschwarz/openrig | *(CLI)* | listed 2026-09-28 |
| CompanyHelm | Distributed agent orchestrator. | https://github.com/CompanyHelm/companyhelm | *(unconfirmed)* | listed 2026-09-28 |
| multi-agent-shogun | Agents run in a shogun hierarchy. | https://github.com/yohey-w/multi-agent-shogun | *(unconfirmed)* | listed 2026-09-28 |
| openswarm | "Mission control" for parallel agents. | https://github.com/openswarm-ai/openswarm | *(unconfirmed)* | listed 2026-09-28 |
| squad (bradygaster) | Human-led teams of Copilot agents. | https://github.com/bradygaster/squad | *(unconfirmed)* | listed 2026-09-28 |
| paperclip | Self-hosted platform where agents run on heartbeats. | https://github.com/paperclipai/paperclip | *(unconfirmed)* | listed 2026-09-28 |
| scion | Google Cloud's container-based orchestration testbed. | https://github.com/GoogleCloudPlatform/scion | *(unconfirmed)* | listed 2026-09-28 |
| ralph-orchestrator / ralph-tui | "Ralph loop" runners that drive an agent through a task list until it is done; ralph-tui has a beads-rust tracker plugin. | https://github.com/mikeyobrien/ralph-orchestrator, https://github.com/subsy/ralph-tui | `ralph` *(unconfirmed)* | listed 2026-09-28 |
| NEEDLE | Runs agents atomically against a shared bead queue. | https://github.com/jedarden/NEEDLE | *(unconfirmed)* | listed 2026-09-28 |
| sortie | Turns tickets into agent sessions (a single Go binary). | https://github.com/sortie-ai/sortie | `sortie` | listed 2026-09-28 |
| cyrus | Watches Linear, GitHub, GitLab and Slack issues and runs agents on them. | https://github.com/cyrusagents/cyrus | *(unconfirmed)* | listed 2026-09-28 |
| Archon | Builds harnesses for deterministic agent workflows. | https://github.com/coleam00/Archon | *(unconfirmed)* | listed 2026-09-28 |
| omnigent | Meta-harness that enforces policy. | https://github.com/omnigent-ai/omnigent | *(unconfirmed)* | listed 2026-09-28 |
| harness (revfactory) | Meta-skill that designs agent teams and writes their skills. | https://github.com/revfactory/harness | *(skill)* | yes 2026-09-28 |
| sandbox-agent | Rivet's daemon and API that run six different coding agents. | https://github.com/rivet-dev/sandbox-agent | *(daemon)* | listed 2026-09-28 |
| humanlayer | Human-in-the-loop control for agents (largely deprecated). | https://github.com/humanlayer/humanlayer | *(unconfirmed)* | listed 2026-09-28 |
| claude-code-action | Anthropic's GitHub Action that runs Claude Code. | https://github.com/anthropics/claude-code-action | *(action)* | listed 2026-09-28 |
| gh-aw | GitHub's agentic workflows, compiled from Markdown. | https://github.com/github/gh-aw | `gh aw` | listed 2026-09-28 |
| open-swe | LangChain's cloud coding agent, started from chat. | https://github.com/langchain-ai/open-swe | *(service)* | listed 2026-09-28 |

## Agent frameworks and SDKs

| Name | What it is | Link | CLI command | Verified |
|---|---|---|---|---|
| Claude Agent SDK | Anthropic's SDK for building agents on the Claude Code harness. | https://code.claude.com/docs/en/agent-sdk/overview | *(library)* | yes 2026-09-28 |
| OpenAI Agents SDK | Successor to OpenAI's experimental **Swarm**: agents, handoffs, guardrails, sessions, tracing. | https://github.com/openai/openai-agents-python | *(library)* | yes 2026-09-28 |
| LangGraph | LangChain's graph-based framework for stateful, durable agents. | https://github.com/langchain-ai/langgraph | `langgraph` (dev server) | yes 2026-09-28 |
| CrewAI | Framework for teams of role-playing agents. | https://github.com/crewAIInc/crewAI | `crewai` | yes 2026-09-28 |
| AutoGen | Microsoft's multi-agent conversation framework. In maintenance mode since late 2025. | https://github.com/microsoft/autogen | *(library)* | yes 2026-09-28 |
| Microsoft Agent Framework | The successor to AutoGen and Semantic Kernel: graph workflows, Python and .NET. | https://github.com/microsoft/agent-framework | *(library)* | yes 2026-09-28 |
| MetaGPT | Multi-agent "AI software company" (PM, architect, engineer roles); the commercial product is MGX. | https://github.com/FoundationAgents/MetaGPT | `metagpt` | yes 2026-09-28 |
| Open Multi-Agent | TypeScript multi-agent runtime with a DAG. | https://github.com/open-multi-agent/open-multi-agent | *(library)* | listed 2026-09-28 |

## Other agent platforms (personal assistants, "agent OS")

| Name | What it is | Link | CLI command | Verified |
|---|---|---|---|---|
| OpenClaw | Peter Steinberger's personal-assistant gateway (Node.js; 20+ chat channels). Has spawned many "*claw" projects (nanoclaw, zeroclaw, ironclaw, picoclaw, NemoClaw...). | https://github.com/openclaw/openclaw | `openclaw` *(unconfirmed)* | yes 2026-09-28 |
| Hermes Agent | Nous Research's self-improving personal agent: CLI, gateway and desktop app. | https://github.com/NousResearch/hermes-agent | `hermes` *(unconfirmed)* | yes 2026-09-28 |
| openfang | Open-source "agent operating system". | https://github.com/RightNow-AI/openfang | *(unconfirmed)* | listed 2026-09-28 |
| Hivekeep | A team of agents with persistent memory. | https://github.com/MarlBurroW/hivekeep | *(unconfirmed)* | listed 2026-09-28 |

## Name themes already crowded

- **Horse and tack.** *harness* is now the generic word for an agent runtime ("agent
  harness", "harness engineering", revfactory/harness, Ruflo's "original agent harness",
  Droid's "harness"), and guides spell out the tack metaphor (reins, saddle, bit).
  **bridle** is taken outright (neiii/bridle, a Rust harness-config CLI). No project was
  found named rein(s), saddle, halter or stirrup, but those words sit right next to a
  crowded metaphor.
- **Music and conducting.** Conductor (three separate projects: Melty's conductor.build,
  Microsoft, and Code Conductor), Symphony (OpenAI), Bernstein, Maestro (ai-maestro),
  orchestra/orchestrator (dozens), tutti, Contrabass.
- **Swarm, hive, colony, herd.** Swarm (OpenAI's old SDK), claude-swarm, ccswarm,
  openswarm, the_swarm, swarm-protocol, ruv-swarm, "hive-mind" (Ruflo), Hivekeep, herdr,
  Agent Farm, antfarm, ox ("team hivemind").
- **Squads, teams, crews, companies.** Claude Squad, squad, Agent Teams (Claude Code and
  777genius), CrewAI, Crewplane, Runner's "crew", ateam, CompanyHelm, "AI software
  company" (MetaGPT), shogun hierarchies.
- **Towns, places, industrial.** Gas Town, Gas City, Wasteland, rigs, Refinery,
  Wheelhouse, Factory, Foundry, OpenRig, Forge (NXTG-Forge).
- **Ships and nautical.** Wheelhouse, helm (CompanyHelm), Berd, dock (OtoDock), Orca,
  convoy (Gas City's term). Kubernetes-adjacent nautical words are crowded in general.
- **Tmux and multiplexer puns.** amux, cmux, dmux, tmux-ide, octomux, muxel, xum.
- **Kanban and boards.** Vibe Kanban, Agent Kanban, openkanban, AI4Kanban, kandev,
  `cline --kanban`.
- **Loops.** Ralph (ralph-tui, ralph-orchestrator, ralphex, ralphy), LoopTroop,
  MartinLoop.
- **Birds and animals.** Goose, Crush's Charm mascots, Orca, Prowl, Squid (agent-squid),
  lobster/claw (the whole OpenClaw family).
- **Crafts and tools.** Sculptor, Machinist, Forge, Archon, Codebuff, Fletch.
- **Beads and threads.** Beads (`bd`), beads_rust (`br`), NEEDLE, convoy/formula
  vocabulary; anything bead-, thread- or needle-flavoured reads as part of the Yegge
  ecosystem.

## Short CLI command names already taken

From verified or listed entries above (plus a few general ones worth avoiding):

`claude`, `codex`, `gemini`, `qwen`, `copilot`, `cursor-agent`, `kiro-cli`, `amp`,
`droid`, `opencode`, `crush`, `cline`, `kilo`, `goose`, `aider`, `plandex`,
`interpreter`, `openhands`, `sweagent`, `mini`, `cn`, `codebuff`, `pi`, `jules`, `gt`
(Gas Town; also used by Graphite), `bd` (Beads), `br` (beads_rust), `cs` (Claude Squad),
`ao` (Agent Orchestrator), `cao`, `uzi`, `ccmanager`, `ccswarm`, `baton`, `bernstein`,
`amux`, `cmux`, `dmux`, `herdr`, `sortie`, `metagpt`, `crewai`, `langgraph`, `ruflo`,
`gh aw`, `bridle` (neiii/bridle).
