---
id: tgxy
title: A rule and role text make agents verify UI work in a browser on a throwaway site
kind: feature
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [vwqt]
tasks: []
---

## The ask

Section 3 and 4 of [[docs/design/dev-site|dev-site]]. Add `workflow/base/rules/verify-ui-in-a-browser.md` (text in the design), the one-line additions to `worker.md` and `manager.md`, a migration that appends `.agent-site/` to a project's `.gitignore` (idempotent; follows ticket xebc's automatic-by-default direction), and a "Verification" note in bridle's CLAUDE.md. track-web gets the rule only; nothing in its repo changes (rule existing-projects).

Verify: `bridle workflow update` plus `bridle sync` in a scratch project renders the rule; `bridle migrate --dry-run` reports the gitignore line and a second run reports nothing.
