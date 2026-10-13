---
id: br-vwqt
title: Agents can run a throwaway bridle site on a free port, log in and click around to verify UI work
kind: feature
created: 2026-10-06
repos: [bridle, bridle-ui]
changes: []
specs: []
blocked_by: []
related: [br-nnj2]
---

## The ask

The human, verbatim (2026-10-05 ~8 PM ET, to the bridle-ui aide):

"We need to build a solution so that you can log into the site and verify things. There are a couple of options:
1. Build a solution like we have on TrackWeb, where you can launch the site in a separate fake folder structure for testing. That's probably the easiest thing and recommended. You can check with how TrackWeb sets that up and what the right approach is."

"You definitely need to be able to click around on the site. While building a UI, the agents need to be able to do this as well to verify that tasks are actually working. I think we need to be able to run a dev server and pick a port to use, typically, or assign a random port. If it's going to be run by multiple agents while they're working on tasks, then they should pick a free port. We have to be able to test this. It's just a critical thing for all software."

How track-web does it (docs/dev-second-instance.md there): a second, throwaway instance beside the developer's: its own port, its own SQLite file, its own login, under a gitignored directory; the Vite config reads VITE_DEV_PORT and VITE_API_TARGET so the client proxies to that instance, never the developer's. Delete the directory and it is gone.

For bridle (aide's reading; the design is open):
- One command starts a throwaway gateway on a free port, with its own config (its own [gateway] username and password, not ~/.bridle/config.toml's) and a fixture folder of fake projects (docs/tickets open and resolved, design/specs, documents), plus whatever daemon data /items, /tasks and /system need. Then a Vite dev server on another free port proxying /api to it. It prints the URL and login.
- Any number of agents can run one at once (free ports, separate directories); each stops only its own (by pid, never by name).
- Agents and the aide can drive it in a browser (log in, click, read the page) to verify a task before calling it done. The bridle-ui worker role/brief says to.
- The human said this is critical for all software: the approach should carry to other web projects (track-web already has its own), e.g. as a rule or pack.

## Again, 2026-10-12 ~9 PM ET (to advisor product-manager)

The human, verbatim: "there should be some sort of a dev setup here for the agents to inspect the results of things. ... Changes are getting shipped without an agent ever looking at the web page at all, let alone testing it, and the results are pretty subpar. I've had a lot better results when the agents can use Playwright and look at some pages, even click around and try out things. The results are far, far better. Probably even the worker agent should be able to do that, the one who's actually building it, but definitely before something's merged, somebody needs to be able to click around on the site. That means two things: 1. The site needs to be able to come up in some sort of test dev version that's got a free port that's available just for that agent. 2. Something like Playwright needs to be installed so they can click and view pages."

And: "Long term, you could make an epic for user interface testing or something similar for bridle, but we need a much, much better approach to this."

Example (advisor product-manager): ui-mxkj (per-agent usage chart) merged 2026-10-12 with "e2e unverified"; nobody had opened the page.

Status: designed in docs/design/dev-site.md (2026-10-06), build tickets vdu8, dgef, tgxy, none scheduled yet. `playwright-cli` is installed on dalek (track-web uses it). New since the design: the pre-merge click-through (see tgxy).
