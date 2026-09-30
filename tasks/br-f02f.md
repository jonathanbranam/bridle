+++
id = "br-f02f"
title = "NUC B1: orchestrator and advisor scripts take a project, run on Linux (bash)"
kind = "chore"
state = "planned"
created_at = "2026-09-30T03:01:34.257Z"
updated_at = "2026-09-30T03:35:17.930125Z"
summary = "Added project support to scripts/claude-orchestrator and scripts/claude-advisor (--project flag, BRIDLE_PROJECT env var, default 'bridle'; project included in session names e.g. orch-meta-notes-nuc, advisor-alice-meta-notes-nuc). Converted both scripts to bash (#!/usr/bin/env bash) for Linux compatibility, no zsh-only syntax. Pass project to bridle for daemon/token selection. Tested syntax with bash -n, verified session name generation logic, verified command-line examples for two projects. Updated docs and CHANGELOG. just check: 788 tests passed."
+++

GOAL: scripts/claude-orchestrator and scripts/claude-advisor take a project (BRIDLE_PROJECT env or an arg; default bridle) and put it in the session name and remote-control name, e.g. orch-meta-notes-nuc, advisor-meta-notes-<name>-nuc (keep the host suffix, BRIDLE_SESSION_SUFFIX and the advisor [name] arg from br-47ba and br-df68). They must run on Linux: bash (#!/usr/bin/env bash), no zsh-only syntax (they use print -r), no macOS-only tools. They pass the project through so bridle picks that project's daemon and token entry (--project / BRIDLE_PROJECT; see discovery in docs/design/agent-host/daemon.md and the credentials entry naming in principals.md). Test with shellcheck if available and by running each with a stub 'claude' on PATH, showing the resulting command lines for two projects. Docs (docs/context/nuc-host.md or the scripts' header comment), CHANGELOG. Acceptance: just check passes. Model: Haiku. Out of scope: the role prompt content (next task), NUC setup.
