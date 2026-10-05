---
id: wu7r
title: "bridle-ui: a clear (X) button on the Document page's search box"
kind: feature
opened: 2026-10-05
repos: [bridle-ui]
changes: []
specs: []
needs: []
see: [jrm2, c2xn, g49c]
tasks: []
---

## The ask


The human, verbatim (2026-10-04 evening, to the bridle-ui aide, on their phone; dictated):

> Yeah, the mobile highlight looks okay at first glance. I think it works. I'll test it more later.
>
> Also, something I noticed: it's really frustrating when the document box has already been
> filled with a really long filename. To clear it out and search again, it needs to have a red X
> clear button next to it so that it's much easier to just empty that box.

## Context

- The box is the search input in bridle-ui `src/Document.tsx` (line 355 at c5c0a0a, placeholder
  "Search tickets and docs, or paste a ticket ID"), added by jrm2. Opening a document fills it with
  the document's path.
- The human uses it mostly on a phone. A native `type="search"` clear control isn't shown on every
  mobile browser (iOS Safari among them), so the button the human asked for needs to be the page's
  own, with an accessible label (see g49c's web-standards rule).
