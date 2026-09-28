# Agent harness / orchestrator name catalogue

**Unverified — no live web access.** WebSearch and WebFetch were both auto-denied in
the session that wrote this (headless, no approval surface available), so this
catalogue was written from the author's training-data knowledge only, not confirmed
against current sources. Names, descriptions, links and CLI commands may be stale,
wrong, or renamed since. Treat every entry as a starting point for a human spot-check
before using it to rule a candidate name in or out — do not treat an absence here as
proof a name is free.

Purpose: for `docs/questions/open/a-new-name-for-the-project-geem.md` — collect names
of existing agentic harnesses, orchestrators, and multi-agent coding tools so a new
bridle name can be checked against them.

| Name | What it is | Link | CLI command |
|---|---|---|---|
| Gas Town | Steve Yegge project referenced in the ticket as a multi-agent coding harness/orchestrator. **Not independently verified** — no confirmed public link or CLI found from memory; likely only described in a Yegge blog post or talk. | *(unconfirmed)* | *(unknown)* |
| Wheelhouse | Steve Yegge project referenced alongside Gas Town, same context. **Not independently verified**, same caveats as above. | *(unconfirmed)* | *(unknown)* |
| Claude Code | Anthropic's official agentic coding CLI/tool (the harness this repo's agents run under). | https://claude.com/claude-code | `claude` |
| Aider | Open-source AI pair-programming CLI that edits local git repos via chat. | https://aider.chat | `aider` |
| OpenAI Codex CLI | OpenAI's terminal-based coding agent, distinct from the older Codex model. | https://github.com/openai/codex | `codex` |
| Goose | Block (Square)'s open-source, extensible AI agent CLI for engineering tasks. | https://github.com/block/goose | `goose` |
| OpenHands (formerly OpenDevin) | Open-source platform for autonomous software-engineering agents (browser + terminal + code). | https://github.com/All-Hands-AI/OpenHands | *(runs as a service/Docker, no single top-level binary)* |
| Cline | VS Code extension (originally "Claude Dev") that runs an autonomous coding agent inside the editor. | https://github.com/cline/cline | *(editor extension, no standalone CLI)* |
| Devin | Cognition Labs' commercial "AI software engineer" agent product. | https://cognition.ai | *(product UI/API, no public CLI known)* |
| SWE-agent | Academic/open-source agent framework for autonomously resolving GitHub issues, from Princeton NLP. | https://github.com/SWE-agent/SWE-agent | `sweagent` |
| CrewAI | Open-source framework for orchestrating role-based multi-agent teams. | https://www.crewai.com | `crewai` |
| AutoGen | Microsoft's open-source framework for building multi-agent conversational/orchestration systems. | https://github.com/microsoft/autogen | *(Python library; no standalone CLI)* |
| MetaGPT | Open-source multi-agent framework that simulates a software company's roles (PM, architect, engineer, etc.) to build software. | https://github.com/FoundationAgents/MetaGPT | `metagpt` |
| Plandex | Terminal-based AI coding agent for large, multi-step tasks with its own plan/diff review workflow. | https://plandex.ai | `plandex` |
| Open Interpreter | Open-source CLI that lets an LLM run code locally in a REPL-like agent loop. | https://github.com/OpenInterpreter/open-interpreter | `interpreter` |
| Factory.ai (Droids) | Commercial agentic coding platform whose agents are branded "Droids"; has an orchestration/CLI layer. | https://factory.ai | `droid` (**unconfirmed** — recalled, not verified) |
| OpenCode | Open-source terminal coding agent positioned as an open alternative to Claude Code. | https://opencode.ai | `opencode` |
| Charm Crush | Charm's terminal-based AI coding agent (from the makers of `glow`/`gum`/`vhs`). | https://github.com/charmbracelet/crush | `crush` |

## Confidence notes

- Rows marked "unconfirmed"/"unknown" need a real web search pass before being relied on.
- Several multi-agent *frameworks* (AutoGen, CrewAI, MetaGPT) are libraries more than
  branded CLIs — included anyway since the ticket is about avoiding name collisions,
  not just CLI collisions.
- This list skews toward tools this author's training data covers reasonably well
  (mostly through the 2024–2025 period); anything newer in 2026 may be missing
  entirely and should be caught by a follow-up live search.
