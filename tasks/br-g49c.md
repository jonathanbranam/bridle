+++
id = "br-g49c"
title = "Rule packs for the web and for mobile: standard forms and ARIA, and no zoom on mobile (track-web's fix, written down once)"
kind = "feature"
state = "planned"
created_at = "2026-10-04T22:13:06.228Z"
updated_at = "2026-10-04T23:32:05.367283Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:aide",
]
summary = """
Created two rule packs for web and mobile development:

**workflow/packs/web/**: Four rules for web standards — form labels (must), input autocomplete (should), ARIA labels for inaccessible controls (should), and semantic HTML over ARIA roles (should). These guide projects building web forms toward accessibility and usability.

**workflow/packs/mobile/**: Four rules for mobile-friendly pages — viewport meta tag with no-zoom (must), touch-action: manipulation body style (should), input font-size >= 16px to prevent iOS Safari auto-zoom (must), and safe-area insets for notched devices (should). Codifies track-web's mobile zoom fix (commit 8f4793c) so projects don't rediscover it.

Both packs follow the existing layout (workflow.toml, rules/*.md) and style (must/should severity, "Why:" explanations, code examples).

Note on pack organization: The human mentioned wanting a "mobile-web" pack in the thread, but projects can enable both web and mobile packs together in config (e.g., `packs = ["typescript", "web", "mobile"]`), giving them all the rules they need. Separate packs allow finer control: a pure web service uses only "web"; a backend service needs neither."""
+++

original id: g49c
Write the two rule packs from docs/tickets/open/rule-packs-for-the-web-and-for-mobile-standard-forms-and-ari-g49c.md (read it and the thread). Look first at workflow/packs/typescript/ (and python, vim) to copy the exact layout, rule file format, frontmatter and how packs are listed or tested. Create workflow/packs/web/ (standard form labels and autocomplete on inputs, ARIA where native elements do not do it, semantic elements; cite track-web's packages/auth/src/LoginPage.tsx as the reference form and ticket t4rf) and workflow/packs/mobile/ (the zoom fix from track-web commit 8f4793c: viewport meta width=device-width, initial-scale=1.0, maximum-scale=1, viewport-fit=cover; body touch-action: manipulation; input font-size at least 16px, because iOS Safari zooms smaller focused inputs; safe-area insets via env(safe-area-inset-*)). Each rule states the requirement and a short why, in the style of the existing rules. Include any index/doc the packs need (docs/README or the workflow docs list packs). Acceptance: just check passes, and any workflow/pack validation (bridle workflow check or similar, if it exists) is clean. Model: Haiku. Out of scope: enabling the packs in bridle-ui or track-web config and fixing bridle-ui index.html (follow-ups after this lands; track-web changes only on its trial branch).

## Thread

### note · external:orchestrator · 2026-10-04T22:13:10.442Z
Approval: the human, via aide (m-0158, 2026-10-04): "This needs to be a rule ... I spent hours fixing this once. I don't ever want to fix this again. This needs to be something encoded in a workflow system. So when I say make a web page that works on mobile, it always does this." Orchestrator's notes for planning: mostly markdown in workflow/packs (likely a web pack and a mobile pack), haiku-sized. Enabling the packs in bridle-ui's and track-web's .bridle/config.toml and fixing bridle-ui's index.html (viewport, touch-action, 16px inputs, safe-area) follow once the packs land; track-web changes go to its trial branch only.

### note · external:orchestrator · 2026-10-04T22:13:16.752Z
From orchestrator: br-g49c (ticket g49c, the human's ask, approved via aide): web and mobile rule packs. Notes on the thread. Mostly markdown, haiku; please plan it and place it after the red fix and p88z.

### note · external:aide · 2026-10-04T22:14:03.972Z
watching the task

### note · external:aide · 2026-10-04T22:14:03.994Z
The human, 2026-10-04, via bridle-ui's aide (verbatim): "this rule needs to exist and be used in Bridal UI, but it should also be part of a rule pack that exists on Bridal. ... We want to be sending these kinds of things to the aid at Bridal, not the orchestrator. ... there should be like a pack that's about mobile web development. And this rule should be a part of that. The packs that I've heard about so far were about languages, which is fine, but a little confusing to me because my rules are definitely like specific to the thing I'm building, not just the programming language that I'm using." Bridle's aide is tracking this and tells bridle-ui's aide when the pack lands. Note: 'packs' is a list in config (bridle-ui has ["typescript"]), and vim is already a non-language pack, so a 'mobile-web' pack enabled next to typescript fits as built.
