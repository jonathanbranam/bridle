+++
id = "br-g49c"
title = "Rule packs for the web and for mobile: standard forms and ARIA, and no zoom on mobile (track-web's fix, written down once)"
kind = "feature"
state = "open"
created_at = "2026-10-04T22:13:06.228Z"
updated_at = "2026-10-04T22:13:10.442330Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: g49c
docs/tickets/open/rule-packs-for-the-web-and-for-mobile-standard-forms-and-ari-g49c.md

## Thread

### note · external:orchestrator · 2026-10-04T22:13:10.442Z
Approval: the human, via aide (m-0158, 2026-10-04): "This needs to be a rule ... I spent hours fixing this once. I don't ever want to fix this again. This needs to be something encoded in a workflow system. So when I say make a web page that works on mobile, it always does this." Orchestrator's notes for planning: mostly markdown in workflow/packs (likely a web pack and a mobile pack), haiku-sized. Enabling the packs in bridle-ui's and track-web's .bridle/config.toml and fixing bridle-ui's index.html (viewport, touch-action, 16px inputs, safe-area) follow once the packs land; track-web changes go to its trial branch only.
