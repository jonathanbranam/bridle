---
id: c2-wiki-in-searches
severity: should
roles: [orchestrator, project-manager, manager, worker, researcher, designer, advisor]
---
When you search the web on a design or software topic, include the c2 wiki (Ward Cunningham's
WikiWikiWeb) whenever it can be reached. It's older, but most of it is still relevant.

- The site needs JavaScript, so WebFetch of `https://wiki.c2.com/?<PageName>` returns little. Read
  the raw page instead: `https://c2.com/wiki/remodel/pages/<PageName>` (JSON; the page is the
  `text` field).
- If it can't be reached, say so in your findings (rule `report-task-failures`); don't drop it
  silently.

Why: the human's words, 2026-10-11 (via advisor wsl2): "This is a site if we can reach that we
should always include in searches on design and software topics; it's older but most still
relevant: https://wiki.c2.com/?TipsForNamingComputers". Also recorded in
`docs/context/naming.md` (Sources).
