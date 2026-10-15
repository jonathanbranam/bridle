---
id: br-6gkv
title: Mail bridge ignores mail where the project address is only in BCC
kind: bug
created: 2026-10-15
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: [br-m38a]
related: []
---

## The ask


Reported by the human via notes' aide, filed by the vesta orchestrator (2026-10-15). The human, 9:54 AM ET: "When I sent the first email to John, I included you as a BCC ... I have a feeling it was missed."

Found: s3://branam-bridle-mail/inbound/lf07te0vv2hu4tvpo8e1pnaqfieg1c5b9kc8ld01 (13:47 UTC) is still in inbound/, never delivered. From an allowed sender; To: jon@jonpeters.online; notes@dev.branam.us appears only in the Received headers ("for <notes@dev.branam.us>"). The bridge matches projects on To/Cc only, so a BCC is skipped, silently (nothing in ~/.bridle/mail-notes.log).

Ask: route by the envelope recipient (the SES receipt's recipients, or failing that the Received "for <addr>" header), so a BCC to <project>@dev.branam.us is delivered like a To. Log, don't silently skip, inbound mail that matches no project. Then deliver the stuck object above to notes.

Verify: email someone else with <project>@ in BCC from an allowed sender; the project's aide receives it.

Related: br-m38a (same report).
