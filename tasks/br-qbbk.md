+++
id = "br-qbbk"
title = "Browser tab titles name what you're viewing: a web pack rule, and bridle-ui follows it on every page"
kind = "feature"
state = "planned"
created_at = "2026-10-06T21:28:06.935Z"
updated_at = "2026-10-06T21:29:34.095375Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

original id: qbbk
Ticket (the human's words, verbatim): docs/tickets/open/browser-tab-titles-name-what-you-re-viewing-a-web-pack-rule-qbbk.md . This task is bridle's half only: the rule. bridle-ui's half (following it on every page) is filed in bridle-ui; do not touch it.
Goal: add ONE new web pack rule file, workflow/packs/web/rules/web.page-title.md, in the exact format of its neighbours (read web.input-clear-button.md first): front matter `id: web.page-title`, `severity: should`, `roles: [worker, reviewer]`; then a short rule body, one example, and a `Why:` paragraph quoting the human.
Rule content:
- The page title (`document.title`) names what the user is viewing, most specific first, so a tab is identifiable from its title alone among many open tabs.
- It updates on every route change and every selection change (opening a document, task or ticket), not only on page load; while the item is loading it falls back to the page name.
- Format: `<item name> - <Page> - <app>`; examples: "Browser tab titles - Document - bridle", "br-qbbk Browser tab titles - Task - bridle", "System - bridle". The item name is the document name or ticket title, or the task id and title.
- Example code: a short JS/TS snippet setting document.title in a route/selection effect (framework-neutral or Svelte/React-neutral).
- Why: the human, 2026-10-06: "The browser title should be updated when the user has selected something in a route to show the name of what they are viewing. I have 10 tabs that all say 'bridle' - I can't find my document or task or system tab." Keep ASCII only (rule ascii-in-editable-text): straight quotes, plain hyphens.
Check how pack rules are registered: if any list, index, test or doc enumerates the web pack's rules (grep for `web.input-clear-button` and `web.semantic-html` outside the rule files; only the ticket mentioned the clear button when I looked, so likely none), add the new one the same way. Do not change other rules.
Acceptance: just check passes (rule-loading tests, if any, load the new file); the file's front matter parses like its neighbours'.
Model: Haiku. Out of scope: bridle-ui changes, other rules, enforcement code.
