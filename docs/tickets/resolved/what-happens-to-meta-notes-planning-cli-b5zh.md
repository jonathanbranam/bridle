---
id: b5zh
title: What happens to meta-notes' own planning CLI?
opened: 2026-09-27
repos: [bridle, meta-notes]
changes: []
specs: []
needs: []
see: [ajqa]
closed: 2026-09-30T05:12:44Z
---

## The question

From `docs/design.md` §15 @ c192bfc, item 6:

> **What happens to meta-notes' own planning CLI?** It's a planning tool too.
> It may be a client of bridle, a source of intake, or unrelated.

## Why it matters

meta-notes is one of the [[docs/context/projects|six projects]] bridle serves.

## Notes

## Resolution

Unrelated. The human, 2026-09-28:

> yes, close b5zh; the meta-notes CLI is part of the plugin, not related to bridle.

meta-notes' planning CLI is a product feature of the Vim plugin (personal planning:
ceremonies, calendar, time logs), not a tool for planning software work. Bridle's only
relationship to meta-notes is running its development, as one of the projects
([[onboarding-survey-meta-notes-ajqa|the meta-notes onboarding survey]]).

Resolved 2026-09-28.
