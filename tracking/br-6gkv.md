+++
id = "br-6gkv"
title = "Mail bridge ignores mail where the project address is only in BCC"
kind = "bug"
state = "integrated"
created_at = "2026-10-15T13:55:17.133Z"
updated_at = "2026-10-15T15:07:46.626980Z"
created_by = "external:orchestrator@vesta"
watchers = [
    "external:orchestrator@vesta",
    "agent:6gkv",
]
branch = "bridle/6gkv"
commit = "459a09a9e735ad2b2982d40458d312e143e7d87d"
summary = "Mail bridge now also reads the 'for <addr>' of Received headers as recipients (parse.rs received_for), so a BCC to <project>@domain routes like a To; allow-list/DMARC checks unchanged. Unmatched mail logs one info line (key, from, subject) in bridge.rs. Test added in mail_test.rs; docs/design/mail.md and CHANGELOG updated. Not done: delivering stuck object inbound/lf07te0v... - run the bridge once on it after landing."
ticket = "6gkv"
+++

Reported by the human via notes' aide, filed by the vesta orchestrator (2026-10-15). The human, 9:54 AM ET: "When I sent the first email to John, I included you as a BCC ... I have a feeling it was missed."

Found: s3://branam-bridle-mail/inbound/lf07te0vv2hu4tvpo8e1pnaqfieg1c5b9kc8ld01 (13:47 UTC) is still in inbound/, never delivered. From an allowed sender; To: jon@jonpeters.online; notes@dev.branam.us appears only in the Received headers ("for <notes@dev.branam.us>"). The bridge matches projects on To/Cc only, so a BCC is skipped, silently (nothing in ~/.bridle/mail-notes.log).

Ask: route by the envelope recipient (the SES receipt's recipients, or failing that the Received "for <addr>" header), so a BCC to <project>@dev.branam.us is delivered like a To. Log, don't silently skip, inbound mail that matches no project. Then deliver the stuck object above to notes.

Verify: email someone else with <project>@ in BCC from an allowed sender; the project's aide receives it.

Related: br-m38a (same report).

## Thread

### note · external:orchestrator@vesta · 2026-10-15T13:55:17.134Z
submitted by external:orchestrator@vesta

### note · agent:pm-1 · 2026-10-15T13:55:30.418Z
Brief (pm-1). Model: Sonnet. Size: S/M. Files: crates/bridle-mail/src/parse.rs ~142 (recipients are built from msg.to() and msg.cc() only; route_for ~68, unknown_name ~114) and the code that fetches the S3 object and calls it (bridge.rs; check whether the SES receipt/notification's recipient list is available there, else read the raw message headers). Fix: add the Received "for <addr>" addresses (all Received headers; only addresses on our domain count, as today) to the recipient list passed to routing; prefer the SES envelope recipients if the bridge already has them. Mail that matches no project: log one line at info/warn (object key, from, subject) instead of silently leaving it. Keep the existing sender allow-list checks. Tests with a fixture .eml that has the project address only in a Received "for" header (routes), one with no match (logged, not delivered), and a normal To (unchanged). Docs: the mail bridge design doc and CHANGELOG. Acceptance: just check. Out of scope: delivering the stuck object (br-6gkv ask: the orchestrator/human runs the bridge once on it after the fix lands; say so in the done note), retention or sweeping of unmatched mail. Same file as br-m38a (parse.rs): blocked-by edge so they run one after another.

### note · agent:6gkv · 2026-10-15T15:02:12.290Z
done: BCC routing via Received 'for' + unmatched-mail log; just check exit 0, 1712 tests, main already merged; d344eee4. Stuck object not delivered: run the bridge once on it after landing.

### note · agent:manager-2 · 2026-10-15T15:02:17.123Z
built: d344eee4 (branch bridle/6gkv)

### note · agent:manager-2 · 2026-10-15T15:07:34.401Z
integrated: 459a09a9e735ad2b2982d40458d312e143e7d87d (branch bridle/6gkv)

### note · agent:manager-2 · 2026-10-15T15:07:46.626Z
cleanup: removed agent 6gkv, branch bridle/6gkv
