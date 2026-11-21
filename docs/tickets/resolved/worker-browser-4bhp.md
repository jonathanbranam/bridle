---
id: br-4bhp
title: Workers get a browser tool (e.g. Playwright MCP) so they can verify UI work on a throwaway site
kind: feature
created: 2026-11-21
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-11-21T21:07:39Z
---

## The ask

Give workers (in projects that build UI) a browser they can drive headless, so they can verify UI work on a throwaway site without the human.

The human, 2026-11-21 ~1:50 PM ET, verbatim (to the bridle-ui aide, answering the orchestrator's question on ui-9xh2 "should agents get a browser tool (e.g. a Playwright MCP) so this stops needing you?"):

> I think agents should get a browser tool. Yes, I assume you mean workers.

Why: rule `verify-ui-in-a-browser` (must, br-tgxy) says UI work is clicked through on a throwaway site (`bridle dev site up --ui .`) before it lands, but no worker has a browser tool, so every UI ticket stalls waiting for the human. bridle-ui ui-9xh2 has been built and blocked since 2026-11-18 for this reason alone; ui-fwsx and ui-4a7d will hit it next. (Submitted by external:aide; approved by the human's yes.)

Acceptance (from the submission): a bridle-ui worker brings up a throwaway site, logs in, clicks through a page and takes a screenshot it can attach to its ticket, with no help from the human.

## Notes from pm-1 (check these first)

- `docs/design/dev-site.md` (~line 112) already names `playwright-cli` as the tool ("installed here; track-web's CLAUDE.md uses it"). So find out what actually stops a worker today: the tool not on the machine's PATH or not installed per project, the worker role's allowed tools (a Bash pattern for `playwright-cli *` missing), the sandbox/containment, or the browser binary download. The cheapest fix may be an allowed-tool entry plus an install step, not an MCP server. Prefer the `playwright-cli` path unless it cannot work; an MCP server is the alternative.
- The submission notes `--allow-tool` alone does not add a tool the role's `--tools` leaves out (found on ui-m2pz with WebSearch): see the researcher role in `.bridle/config.toml` for how that was solved.
- Agents may not widen their own permissions, and the human approves permission changes: this ticket is that approval for the worker role's browser tool only. Put it where the role is defined (`workflow/base/roles/worker.md`, `crates/bridle-daemon/src/config.rs` role defaults, the project's `.bridle/config.toml` for bridle-ui); the base worker should not get it everywhere unless it is harmless, so decide base vs per-project and say why.
- Migration: if a role default or config key changes, say how existing projects (bridle-ui, track-web, meta-notes) get it with no edit, or that they opt in with one config line. A screenshot to attach goes under the project's usual place for ticket attachments (check what exists; do not invent a store).
- Update rule `verify-ui-in-a-browser` and the dev-site doc to say a worker can do it now, and that a missing tool still follows `missing-tools`. Edit the existing rule, add no new one.
- Related: br-z8ru (options for stricter UI verification, research): this ticket is the capability it will assume. Do not wait for it.

## Verify

A real run on bridle-ui or `bridle dev site up` in bridle: a worker session opens the site, logs in, screenshots a page. `just check` passes. A browser download or install is not part of `just check`.

## Model

Sonnet. Out of scope: the style guide and gates (br-z8ru), fixing the meta-notes-ui checkbox.
