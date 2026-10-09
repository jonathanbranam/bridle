+++
id = "br-srj5"
title = "Mail attachments: UTF-8 text saved garbled as Latin-1 (em dash becomes 'â€”')"
kind = "bug"
state = "planned"
created_at = "2026-10-08T00:56:39.666Z"
updated_at = "2026-10-09T09:57:12.901962Z"
created_by = "external:aide"
watchers = ["external:aide"]
summary = "Text attachments in bridle-mail parse.rs are now saved as the sender's transfer-decoded bytes (raw_contents re-parses the part's raw body as an unlabelled binary part), so UTF-8 labelled iso-8859-1 or unlabelled is byte-identical. Binary parts unchanged. Message body path untouched (not checked for the same bug). Tests: 8bit UTF-8 under three labels, base64 with iso-8859-1 label. CHANGELOG added."
ticket = "srj5"
+++

Ticket: docs/tickets/open/mail-attachments-utf-8-text-saved-garbled-as-latin-1-em-dash-srj5.md (read it: the evidence and the aide's guess).
Goal: a mail attachment is saved as the sender's raw bytes when those bytes are valid UTF-8, whatever charset the part declares.
Where: crates/bridle-mail/src/parse.rs saves `part.contents()`; for a text/* part the MIME parser has already decoded it by the declared charset, so a part labelled iso-8859-1 (or with no charset) whose bytes are really UTF-8 comes out as mojibake (em dash becomes 'â€”'). First reproduce with a test, to verify the guess. Then, for attachments, save the part's undecoded transfer-decoded bytes (base64 / quoted-printable decoded only), not the charset-decoded text; use the parser's raw-body accessor (check the mail-parser crate version in Cargo.lock for the exact method, e.g. contents()/raw bytes) and do not re-encode. Leave the message BODY text handling alone unless the same bug shows there (then say so in the done note and fix it the same way).
Tests (crates/bridle-mail tests): a UTF-8 .md part labelled charset=iso-8859-1 saves byte-identical; one with no charset saves byte-identical; a real Latin-1 body text still comes out readable if the body path is touched; binary attachments unchanged.
Serialize: br-gdyy and br-843g edit other files of bridle-mail; this task touches parse.rs only, so it may run alongside, but start after br-ezpj/br-843g if only one slot is free.
Docs: CHANGELOG. Migration: none. Out of scope: re-fetching the two garbled files already saved on dalek (S3 copies are gone).
Acceptance: just check passes. Model: Haiku is enough only if the fix is the one-liner the guess predicts; use Sonnet.

## Thread

### note · agent:srj5 · 2026-10-09T09:57:12.901Z
done: text attachments saved byte-identical (parse.rs raw_contents); just check exit 0, 1372 tests passed; 19b7cc418a9ee5f9d6e88ce005615b4a07809256
