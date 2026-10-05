+++
id = "br-46me"
title = "[at restart] Run the daemon self-upgrade, then restart the gateway once"
kind = "feature"
state = "integrated"
created_at = "2026-10-05T03:00:59.360Z"
updated_at = "2026-10-05T23:42:18.829357Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "human",
]
+++

Main is green with p88z, bnhn, bek3 and e9yu. The orchestrator's request for the upgrade was refused by Claude Code's auto-mode classifier ("Interfere With Workloads"), so it needs your hands:

1. `bridle daemon restart --upgrade` (builds verified main, restarts at a quiet point, resumes every agent).
2. Then `bridle gateway --detach` once (bek3: the old gateway can't self-update; after this it re-execs itself when the binary changes).

Or allow `bridle daemon restart --upgrade` for the orchestrator in Claude Code's permissions, and it will do step 1 itself from now on.

## Thread

### note · external:orchestrator · 2026-10-05T03:00:59.362Z
created for the human, priority normal

### note · external:orchestrator · 2026-10-05T03:00:59.364Z
To-do for you (normal priority): [at restart] Run the daemon self-upgrade, then restart the gateway once. Finish it with `bridle task done br-46me`.

### note · external:orchestrator · 2026-10-05T03:52:31.609Z
Update (orchestrator, 03:55Z): the daemon upgrades itself automatically after merges (c548182b at 01:50Z, which already includes bek3). The next attempt, 0d69d617, failed its self-check by timeout (ticket up82); the next merge retries it. So step 1 is only needed if it is still failing in the morning (check 'bridle events --kind upgrade.'). Step 2, 'bridle gateway --detach' once, is still needed for bek3.

### note · human · 2026-10-05T23:42:18.829Z
done
