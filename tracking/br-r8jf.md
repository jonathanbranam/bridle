+++
id = "br-r8jf"
title = "Three small robustness fixes: pair-mint --replace-direct order, stop_check ps in async, sync never removes a [mail] entry"
kind = "fix"
state = "built"
created_at = "2026-10-16T21:14:33.963Z"
updated_at = "2026-10-17T05:28:32.040863Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:r8jf",
]
branch = "bridle/r8jf"
commit = "763037e1"
summary = "(1) token pair-mint --direct now revokes the earlier visitor token (role@machine) after the direct mint succeeds; test cli_e2e a_failed_direct_pair_mint_keeps_the_visitor_token. Caveat: the same-name revoke before the mint stays, because the daemon refuses a mint over an active token of that name (no replace for external tokens); fixing that is a token API change, out of scope. (2) stop_check_running is only file reads (no ps in this code); it now runs in spawn_blocking in tick_stall_check. (3) Decided intended: nothing records hand-added [mail] names, so sync only adds; documented in docs/design/cli.md. CHANGELOG line added."
ticket = "r8jf"
+++

docs/tickets/open/three-small-robustness-fixes-pair-mint-replace-direct-order-r8jf.md

## Thread

### note · external:aide · 2026-10-16T21:42:46.766Z
The human, 2026-10-16 ~5:35 PM ET (via aide), approving tvrp, whce, 8kzq, 9qws, vmrc, r8jf: "Yes that looks good." Readied by aide.

### note · agent:r8jf · 2026-10-17T05:28:32.040Z
built: 763037e1 (branch bridle/r8jf)
