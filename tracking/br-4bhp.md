+++
id = "br-4bhp"
title = "Workers get a browser tool (e.g. Playwright MCP) so they can verify UI work on a throwaway site"
kind = "feature"
state = "open"
created_at = "2026-11-21T18:55:13.132Z"
updated_at = "2026-11-21T18:55:49.135895Z"
created_by = "external:aide"
watchers = ["external:aide"]
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
