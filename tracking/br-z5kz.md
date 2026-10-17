+++
id = "br-z5kz"
title = "Manager waiting for a draining restart inside its turn deadlocks the restart"
kind = "bug"
state = "planned"
created_at = "2026-10-14T03:54:55.655Z"
updated_at = "2026-10-17T17:43:34.452165Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
summary = "A spawn refused during a restart drain now says to end the turn (daemon resumes the agent after the restart); manager role, daemon.md, CHANGELOG updated; upgrade_test asserts the text. Did not make a wake-blocked agent count as quiet (riskier) nor touch the double-wake note."
ticket = "z5kz"
+++

docs/tickets/open/manager-waiting-for-a-draining-restart-inside-its-turn-deadl-z5kz.md

## Thread

### note · external:orchestrator · 2026-10-14T18:57:22.260Z
orchestrator, 19:00Z: after the self-upgrade to c0814ca5 (includes br-nncj), the 'context at 150K' wake arrived once (18:5xZ), where this morning's came twice (m-12688 at 151K and m-12689 at 150K, 10 s apart). One sample; looks fixed. Will watch the next context wake too.

### note · external:orchestrator · 2026-10-14T23:15:36.471Z
Confirmed again (orchestrator, 2026-10-14 23:16Z): a 'context at 150K' wake arrived to the relaunched orchestrator session ({"threshold":150000,"tokens":150572}). Context wakes work after nncj; this can resolve.

### note · external:advisor/product-manager · 2026-10-17T15:26:13.385Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.

### note · agent:wz5kz · 2026-10-17T17:43:30.230Z
done: drain spawn refusal tells agent to end its turn + manager role/docs/CHANGELOG; just check exit 0, 1823 tests passed; 1db34fb6c8fb2c1bdc56408db59ed3e3d5fcc1f0

### note · agent:wz5kz · 2026-10-17T17:43:34.452Z
wz5kz: 'bridle ticket built' failed: 'forbidden: worker agents cannot use agent lifecycle endpoints'. Branch bridle/wz5kz, commit 1db34fb6; just check exit 0, 1823 passed. Manager please mark built.
