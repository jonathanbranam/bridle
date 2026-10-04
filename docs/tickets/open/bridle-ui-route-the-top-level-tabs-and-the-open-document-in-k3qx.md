---
id: k3qx
title: "bridle-ui: route the top-level tabs and the open document in the URL"
kind: feature
opened: 2026-10-04
repos: [bridle-ui]
changes: []
specs: []
needs: []
see: [x8jt, jrm2, essy]
tasks: []
---

## The ask


The human, verbatim (2026-10-04, to the bridle-ui aide):

> the ui should route to the top-level tabs and open documents in the URL. see track-web;
> probably use react route unles you have a stronger suggestion. I should be able to refresh and
> remain on the same tab and document.

## Context

- track-web (`/Volumes/Data/work/track-web-workspace/track-web`) uses `react-router-dom` ^7.2.0
  in each of its `client-*` apps.
- bridle-ui's CLAUDE.md says "no router until a second page needs one"; it now has several
  top-level tabs (to-dos, Time, Document).
- The gateway's UI fallback (`crates/bridle-gateway/src/ui.rs`, `serve_ui`) serves `index.html`
  for any path no API route or file claims, but only when the path has no extension: "A path
  with an extension is a file the page asked for; anything else is a client-side route". A
  document is identified by `project` + repo-relative `path` (`src/api/generated/Document.ts`),
  and those paths end in `.md`, so a URL whose last segment is the raw document path
  (`/doc/bridle/docs/tickets/open/foo-k3qx.md`) gets a 404 on refresh.
