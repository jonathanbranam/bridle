+++
id = "br-dxcw"
title = "vwqt: agents run a throwaway bridle site (daemon+gateway+UI) on a free port, log in and verify UI work"
kind = "feature"
state = "open"
created_at = "2026-10-06T00:07:44.945Z"
updated_at = "2026-10-06T00:07:45.297807Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

Human's ask, ticket vwqt: docs/tickets/open/agents-can-run-a-throwaway-bridle-site-on-a-free-port-log-in-vwqt.md. Modelled on track-web's docs/dev-second-instance.md. Bridle owns the design and the gateway/daemon side, plus carrying it over to every project; bridle-ui needs a matching dev-server piece (VITE port/API target), filed in bridle-ui once the split is designed. Design first, then build.
