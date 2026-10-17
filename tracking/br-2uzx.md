+++
id = "br-2uzx"
title = "Usage wake fires on a stale pre-reset reading after the window has reset"
kind = "bug"
state = "integrated"
created_at = "2026-10-14T10:01:06.003Z"
updated_at = "2026-10-17T23:39:00.399584Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:w2uzx",
]
branch = "bridle/w2uzx"
commit = "daf24de414ed3767a35a57da82d5677e76e6b65f"
summary = "Store::upsert_rate_limit (store.rs) now drops a reading whose resets_at is more than 1 minute older than the stored one for that window, so a late pre-reset reading neither re-fires the usage wake nor reaches the governor (both read the stored rate_limits). Chose the store as the single choke point; a past-but-not-superseded resets_at is still kept. Test in wake.rs; CHANGELOG line added; no design doc described this."
ticket = "2uzx"
+++

docs/tickets/open/usage-wake-fires-on-a-stale-pre-reset-reading-after-the-wind-2uzx.md

## Thread

### note · external:advisor/product-manager · 2026-10-17T15:26:13.837Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.

### note · agent:w2uzx · 2026-10-17T23:38:53.147Z
built: 4a9683eb (branch bridle/w2uzx)

### note · agent:w2uzx · 2026-10-17T23:38:53.418Z
done: stale pre-reset usage readings dropped in the store upsert; just check exit 0, 1834 tests passed; 4a9683eb

### note · agent:manager-2 · 2026-10-17T23:39:00.399Z
integrated: daf24de414ed3767a35a57da82d5677e76e6b65f (branch bridle/w2uzx)
