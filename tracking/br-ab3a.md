+++
id = "br-ab3a"
title = "Make the orchestrator non-interactive (headless)"
kind = "question"
state = "reopened"
created_at = "2026-10-03T11:04:16.326Z"
updated_at = "2026-10-18T23:24:06.626255Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
ticket = "z485"
+++

docs/tickets/open/make-the-orchestrator-non-interactive-headless-z485.md

## Thread

### note · external:advisor · 2026-10-03T11:05:27.036Z
The human, 2026-10-03: do not build. A raw idea that needs refinement and specification with the human first (their to-do to expand on it is linked in the ticket). Don't plan or start it.

### note · agent:pm-1 · 2026-10-03T12:42:33.517Z
dropped: k7tm sort: the human said don't build; the ticket stays

### note · external:advisor/product-manager · 2026-10-18T13:20:43.328Z
From advisor (product-manager): readied for a DESIGN PASS ONLY (designer-1), as for gb97: the human reviews the design before any build. The human, 2026-10-18 ~8:45 AM ET: 'I don't want more interactive sessions on RC. Let's move the orch to background before pursuing the admin. Sequence that work.' and 'I think we would be ok to move the orch to background as a first step.' Brief: the ticket's Proposal (how the human talks to a headless orchestrator without RC, see gcvj's findings; supervisor/handover/restart path; permissions and auto mode as a spawned agent; still one per machine in <host>-home; cut-over and fallback). Write '## Design options' into the ticket. First step of epic p7jf; gb97 and j6d3 are blocked_by this. Row br-ab3a was dropped 10-03 and is reopened (kind still question on the row; can't change while reopened).

### note · external:orchestrator · 2026-10-18T13:21:07.098Z
From orchestrator: next design pass for you, ticket br-z485 (orchestrator to the background; row br-ab3a), DESIGN PASS ONLY: the brief and the human's words (2026-10-18 ~8:45 AM) are on the thread (bridle ticket show br-z485). The human reviews before any build; gb97's build waits on this. Post the result on the ticket and tell pm-1 and external:advisor/product-manager when done.

### note · agent:designer-1 · 2026-10-18T13:22:28.958Z
designer-1: '## Design options' written (013111e2): run the existing orchestrator role as a daemon agent in <host>-home with one is_orchestrator() helper and a routing alias; human keeps talking to aide; session path kept as fallback for a release. Four options, five questions for the human. Design only.

### note · external:advisor/product-manager · 2026-10-18T13:32:43.044Z
From advisor (product-manager): related, approved 2026-10-18 and being planned: br-7ytg (the home project is the orchestrator's only project; refusals elsewhere). Option A fits it; the build must keep 7ytg's refusals letting the orchestrator agent run in the home.

### note · external:advisor/product-manager · 2026-10-18T14:40:36.029Z
From advisor (product-manager): for the build: br-9966 (orchestrator hold/release, being built; the human 10-18: disable auto-start, re-enable, auto-started after re-enable) is written against the session supervisor. In agent mode the same hold must stop the daemon autostarting the orchestrator agent, and release must start it.

### note · external:advisor/product-manager · 2026-10-18T23:24:06.626Z
From advisor (product-manager): the human, 2026-10-18 ~7:20 PM ET: 'fine, yes, i'm worried background agent will just take up time for little benefit. let's do next steps on project transfer'. Design review deferred; the human doubts the benefit is worth the time. Nothing is blocked today except gb97/j6d3 (sequenced after this).
