---
id: c2xn
title: "bridle-ui: highlight-to-comment doesn't trigger on mobile (only mouseup is listened for)"
kind: bug
opened: 2026-10-04
repos: [bridle-ui]
changes: []
specs: []
needs: []
see: [jrm2, x8jt, a3yd, g49c]
tasks: []
---

## The ask


The human, verbatim (2026-10-04 evening, to the bridle-ui aide, using the Document page on their
phone; dictated):

> Hmm... highlighting does work on mobile. The native highlight probably doesn't trigger the same
> as on desktop.

(The aide read "does work" as "doesn't work", given the second sentence. The human hasn't yet
confirmed that reading.)

## Context

- bridle-ui `src/Document.tsx` at 5ff4354 opens the comment box from a selection only in an
  `onMouseUp={select}` handler on the document body (line 385; `select` reads
  `window.getSelection()`, line 281). Nothing listens for `selectionchange`, `pointerup` or
  `touchend`. On a phone, the native selection (long-press, then drag the handles) doesn't
  necessarily fire `mouseup`.
- The human asked for mobile highlighting from the start: "When I highlight a section on mobile,
  or if the media width is too narrow, it should just cut the doc rendering in the center and open
  up the comment box directly below where I highlighted"
  ([[document-view-project-dropdown-ticket-search-by-id-comment-b-jrm2|jrm2]]), and "supporting
  both is essential".
- ui-pmkd (markdown rendering, [[bridle-ui-render-front-matter-and-auto-link-urls-file-paths-a3yd|a3yd]])
  is about to rework how the document body is rendered and split, so the two should be done
  together or in order.
