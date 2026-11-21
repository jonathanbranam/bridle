+++
id = "br-4bhp"
title = "Workers get a browser tool (e.g. Playwright MCP) so they can verify UI work on a throwaway site"
kind = "feature"
state = "integrated"
created_at = "2026-11-21T18:55:13.132Z"
updated_at = "2026-11-21T21:07:39.168117Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:w4bhp",
]
branch = "bridle/w4bhp"
commit = "42358a5b0a4ff59605e4f40d402467166d19f49e"
summary = "Nothing stops a worker today: the base worker allows plain Bash (allowed_tools and --tools), which covers playwright-cli, so no role change and no MCP (base-wide is harmless; existing projects need no edit; a project narrowing to Bash(...) patterns adds Bash(playwright-cli *)). Verified from this worker session: bridle dev site up, playwright-cli open, screenshot to /tmp/4bhp-verify, close, site down. No bridle-ui checkout here so only the gateway (404 page, API 401) was reachable, not a UI login. Added a config test pinning Bash, a dev-site.md note, and a CHANGELOG line. Screenshots: name the path in the done message or a comment; no attachment store exists."
ticket = "4bhp"
+++

docs/tickets/open/worker-browser-4bhp.md

submitted by external:aide

The human, 2026-11-21 ~1:50 PM ET, verbatim (to the bridle-ui aide), answering the orchestrator's question on ui-9xh2 ("should agents get a browser tool (e.g. a Playwright MCP) so this stops needing you?"):

> I think agents should get a browser tool. Yes, I assume you mean workers.

Why: rule verify-ui-in-a-browser (must; br-tgxy) says UI work is clicked through on a throwaway site (`bridle dev site up --ui .`) before it lands, but no worker has a browser tool, so every UI ticket stalls waiting for the human. bridle-ui ui-9xh2 has been built and blocked since 2026-11-18 for this reason alone; ui-fwsx and ui-4a7d will hit it next.

Ask: give the worker role (in projects that build UI) a browser it can drive headless, e.g. a Playwright MCP server in the worker's tool set (note: `--allow-tool` alone doesn't add a tool the role's `--tools` leaves out, as found on ui-m2pz with WebSearch). Acceptance: a bridle-ui worker brings up a throwaway site, logs in, clicks through a page and takes a screenshot it can attach to its ticket, with no help from the human.

Approved by asking: the human said yes.

## Thread

### note · external:aide · 2026-11-21T18:55:13.133Z
submitted by external:aide

### note · external:orchestrator · 2026-11-21T18:55:48.827Z
readied by external:orchestrator

### note · external:orchestrator · 2026-11-21T18:55:49.135Z
readied by orchestrator on the human's go (via aide, 2026-11-21 ~1:50 PM ET): "I think agents should get a browser tool. Yes, I assume you mean workers."

### note · external:orchestrator · 2026-11-21T19:07:34.448Z
orchestrator: data point for planning. On 2026-11-21 bridle-ui worker picker-a already verified ui-4a7d in Chromium via Playwright on a throwaway site (375/768/1280/1920; its done note on ui-4a7d). Check what bridle-ui's workers already have before building; this ticket may reduce to making that available to every project's workers.

### note · external:aide · 2026-11-21T19:11:40.603Z
aide: the human asked aide to read playwright.dev/docs/intro and playwright.dev/agent-cli/introduction (2026-11-21). Findings: (1) docs/intro is Playwright Test, a per-project Node test runner (npm init playwright@latest, then npx playwright test); that is the only place a 'playwright' command exists. (2) The agent CLI is a separate global package, npm install -g \@playwright/cli, command playwright-cli, 'designed for coding agents': a persistent browser daemon, headless by default, each command prints URL, title and an accessibility snapshot with element refs (e21) for the next command; open/goto/click/fill/type/press/snapshot/screenshot/console/requests; named sessions with -s=<name>; installable agent skills; 'install-browser' for browsers. Its own comparison: the CLI is best for coding agents like Claude Code (lower token cost) and MCP for specialised agentic loops. So: workers should drive the UI with playwright-cli (installed on dalek, 0.1.25), not write Node scripts against playwright-core and not add an MCP. Relevant to this ticket's 'e.g. Playwright MCP': Playwright itself recommends the CLI over MCP for Claude Code.

### note · external:aide · 2026-11-21T19:22:21.141Z
From the human, via aide (2026-11-21), verbatim: "FYI I installed playwright-cli globally and chromium" (on dalek).

### note · agent:w4bhp · 2026-11-21T21:07:30.851Z
built: 6e9d9736 (branch bridle/w4bhp)

### note · agent:w4bhp · 2026-11-21T21:07:30.890Z
done: no role change needed (worker Bash already covers playwright-cli); test+docs; check exit 0, 2028 tests; 6e9d9736. Real run: dev site up, playwright-cli open+screenshot (404 gateway page, no UI checkout so no login click), site down.

### note · agent:manager-2 · 2026-11-21T21:07:39.005Z
integrated: 42358a5b0a4ff59605e4f40d402467166d19f49e (branch bridle/w4bhp)

### note · agent:manager-2 · 2026-11-21T21:07:39.168Z
ticket file moved to docs/tickets/resolved/worker-browser-4bhp.md
