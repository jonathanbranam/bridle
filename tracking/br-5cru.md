+++
id = "br-5cru"
title = "Commands that run bridle on another machine over ssh fail unless the user adds bridle to the non-interactive PATH"
kind = "bug"
state = "planned"
created_at = "2026-10-14T01:20:43.432Z"
updated_at = "2026-10-14T01:25:33.564407Z"
created_by = "external:advisor/wsl2"
watchers = ["external:advisor/wsl2"]
ticket = "5cru"
+++

docs/tickets/open/commands-that-run-bridle-on-another-machine-over-ssh-fail-un-5cru.md

## Thread

### note · external:advisor/product-manager · 2026-10-14T01:25:16.433Z
The human, 2026-10-13 ~9:25 PM ET (to advisor product-manager), verbatim: "another related issue is 5cru that bridle needs to be on the remote non-interactive path which requires changing /etc/zshenv. I'm not sure a real solution here, it can be researched and designed and then returned to me for review." Scope: research and a design (options, recommendation) written into this ticket, then back to the human for review. No build until the human approves.

### note · agent:pm-1 · 2026-10-14T01:25:33.195Z
Brief (pm-1). DESIGN ONLY, no build (the PdM, from the human: write options and a recommendation into the ticket, then it returns to the human for review). Model: Sonnet. Size: S. Role: designer.

Read the ticket (the findings and the three options are there), crates/bridle/src/token_pair.rs (~92-111), crates/bridle/src/preflight_remote.rs (~212), docs/design/cli.md (token pair, project preflight), docs/context/add-a-machine.md and docs/context/windows-wsl2-host.md. Write a "## Design" section on the ticket (via the ticket thread if you cannot edit the file; comment with the full text) covering: each option's behaviour on macOS, Ubuntu and WSL2; what each does to a machine set up by add-a-machine today with no PATH edit; failure reporting ("bridle not found on <host>", never "unreachable"); how the remote path is found or recorded and where that lives (and, if it adds config, how existing machines get a default without edits); what add-a-machine and the WSL2 doc must say meanwhile; rejected alternatives with reasons; the tickets a build would split into (sizes, which share files). End with ONE recommendation. Include the c2 wiki in any web search if you search at all (rule c2-wiki-in-searches), and say if it could not be reached. Do not change code. Acceptance: the design is on the ticket thread; mark it done and stop; the human reviews.
