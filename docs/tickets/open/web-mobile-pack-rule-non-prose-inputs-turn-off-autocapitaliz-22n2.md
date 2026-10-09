---
id: 22n2
title: "Web/mobile pack rule: non-prose inputs turn off autocapitalize, autocorrect and spellcheck"
kind: feature
opened: 2026-10-09
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [9xbk, g49c]
tasks: [br-22n2]
---

## The ask

The human, verbatim (2026-10-08 ~7:55 PM ET, relayed by the bridle-ui aide, m-7348), first:

> Is there a field hint or something you can put on the username field so that it doesn't auto capitalize the first character? Maybe call it an email or something? that behavior drives me nuts on webpages.

(bridle-ui ticket ybka fixed the login username.) Then:

> Do this always! Make it a bridle rule for the web design pack. And mobile if it applies there.

## The ask

A new rule, `web.input-no-autocapitalize` (severity **must**: the human said "always"), mirrored or cross-referenced in the mobile pack since only on-screen keyboards capitalize:

- Inputs whose values aren't prose (username, email, URL, IDs, codes, search terms for identifiers) set `autocapitalize="none"` (React `autoCapitalize`), `autocorrect="off"` and `spellcheck="false"`.
- `type=email` and `type=url` already skip capitalizing, but don't use `type=email` for a username that isn't an email.
- Prose boxes keep the defaults.

## Also (from the bridle-ui aide)

`mobile.viewport-meta` says to use `maximum-scale=1`, but bridle-ui's CLAUDE.md says the viewport meta must not set `maximum-scale` (it blocks zoom, failing WCAG 1.4.4), and 16px input text alone prevents focus zoom. Reconcile them; the research ticket filed alongside this one has the detail.
