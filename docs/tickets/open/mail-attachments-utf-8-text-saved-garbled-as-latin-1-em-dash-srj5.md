---
id: srj5
title: "Mail attachments: UTF-8 text saved garbled as Latin-1 (em dash becomes 'â€”')"
kind: bug
opened: 2026-10-08
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [rs7p, gdyy]
tasks: []
---

## The ask

Reported by the orchestrator, 2026-10-07 ~9:20 PM ET (m-6776), from the human's mail test on dalek: the same `.md` file mailed twice arrived different. From Gmail on the iPhone (SES d91o60r85gc72pbmuni8446bamq2jfk458e8tc01, 8:37 PM) the saved copy has UTF-8 punctuation garbled as Latin-1 (em dash `â€”`, en dash `â€“`); from `jonathan@branam.us` at 8:55 PM (0497fh2nav2lh8bt5k9ar6l0pgmr09nc1d7dpog1) it is clean. Both are under `/Volumes/Data/work/bridle/.bridle/inbox/mail/<SES id>/` on dalek (the S3 objects are gone, deleted on delivery).

Aide's guess, to verify: `crates/bridle-mail/src/parse.rs` saves `part.contents()`; for a `text/*` part the parser has already decoded it by the part's declared charset, so a part sent with a wrong or missing charset (iPhone Mail forwarding a `.md`) gets its UTF-8 bytes read as Latin-1 and re-encoded. Possibly the sender garbled it instead; compare with a test message built the same way.

The ask: an attachment is saved as the sender's bytes when those are valid UTF-8, whatever the declared charset; a test with a UTF-8 `.md` part labelled `iso-8859-1` and one with no charset. Small; can ride with br-gdyy if simpler.
