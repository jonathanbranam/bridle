+++
id = "br-g49c"
title = "Rule packs for the web and for mobile: standard forms and ARIA, and no zoom on mobile (track-web's fix, written down once)"
kind = "feature"
state = "planned"
created_at = "2026-10-04T22:13:06.228Z"
updated_at = "2026-10-04T22:13:30.861254Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: g49c
Write the two rule packs from docs/tickets/open/rule-packs-for-the-web-and-for-mobile-standard-forms-and-ari-g49c.md (read it and the thread). Look first at workflow/packs/typescript/ (and python, vim) to copy the exact layout, rule file format, frontmatter and how packs are listed or tested. Create workflow/packs/web/ (standard form labels and autocomplete on inputs, ARIA where native elements do not do it, semantic elements; cite track-web's packages/auth/src/LoginPage.tsx as the reference form and ticket t4rf) and workflow/packs/mobile/ (the zoom fix from track-web commit 8f4793c: viewport meta width=device-width, initial-scale=1.0, maximum-scale=1, viewport-fit=cover; body touch-action: manipulation; input font-size at least 16px, because iOS Safari zooms smaller focused inputs; safe-area insets via env(safe-area-inset-*)). Each rule states the requirement and a short why, in the style of the existing rules. Include any index/doc the packs need (docs/README or the workflow docs list packs). Acceptance: just check passes, and any workflow/pack validation (bridle workflow check or similar, if it exists) is clean. Model: Haiku. Out of scope: enabling the packs in bridle-ui or track-web config and fixing bridle-ui index.html (follow-ups after this lands; track-web changes only on its trial branch).

## Thread

### note · external:orchestrator · 2026-10-04T22:13:10.442Z
Approval: the human, via aide (m-0158, 2026-10-04): "This needs to be a rule ... I spent hours fixing this once. I don't ever want to fix this again. This needs to be something encoded in a workflow system. So when I say make a web page that works on mobile, it always does this." Orchestrator's notes for planning: mostly markdown in workflow/packs (likely a web pack and a mobile pack), haiku-sized. Enabling the packs in bridle-ui's and track-web's .bridle/config.toml and fixing bridle-ui's index.html (viewport, touch-action, 16px inputs, safe-area) follow once the packs land; track-web changes go to its trial branch only.

### note · external:orchestrator · 2026-10-04T22:13:16.752Z
From orchestrator: br-g49c (ticket g49c, the human's ask, approved via aide): web and mobile rule packs. Notes on the thread. Mostly markdown, haiku; please plan it and place it after the red fix and p88z.
