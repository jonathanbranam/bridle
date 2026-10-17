+++
id = "br-r8jf"
title = "Three small robustness fixes: pair-mint --replace-direct order, stop_check ps in async, sync never removes a [mail] entry"
kind = "fix"
state = "integrated"
created_at = "2026-10-16T21:14:33.963Z"
updated_at = "2026-10-17T06:14:29.842311Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:r8jf",
]
branch = "bridle/r8jf"
commit = "fa5444437150694264b419751dd2d0f10ce5a40f"
summary = "(1) token pair-mint --direct now revokes the earlier visitor token (role@machine) after the direct mint succeeds; test cli_e2e a_failed_direct_pair_mint_keeps_the_visitor_token. Caveat: the same-name revoke before the mint stays, because the daemon refuses a mint over an active token of that name (no replace for external tokens); fixing that is a token API change, out of scope. (2) stop_check_running is only file reads (no ps in this code); it now runs in spawn_blocking in tick_stall_check. (3) Decided intended: nothing records hand-added [mail] names, so sync only adds; documented in docs/design/cli.md. CHANGELOG line added."
ticket = "r8jf"
+++

docs/tickets/open/three-small-robustness-fixes-pair-mint-replace-direct-order-r8jf.md

## Thread

### note · external:aide · 2026-10-16T21:42:46.766Z
The human, 2026-10-16 ~5:35 PM ET (via aide), approving tvrp, whce, 8kzq, 9qws, vmrc, r8jf: "Yes that looks good." Readied by aide.

### note · agent:r8jf · 2026-10-17T05:28:32.040Z
built: 763037e1 (branch bridle/r8jf)

### note · agent:r8jf · 2026-10-17T05:28:36.036Z
done: pair-mint --direct revokes the visitor token after the mint (+test), stall-check file read in spawn_blocking, sync-never-removes-[mail] documented as intended; just check exit 0, 1787 tests passed (6 skipped), commit 763037e1 (main already merged). Note: stop_check_running has no ps, only file reads; and the same-name revoke before mint stays (daemon refuses a mint over an active token). One earlier run failed restart_land_test a_land_past_the_cap... under load; passes alone and in the re-run.

### note · agent:r8jf · 2026-10-17T05:54:15.101Z
update: merged new main (br-8kzq) into bridle/r8jf; just check exit 0, 1789 passed, 6 skipped; tip is now 38b1bac4 (use that sha).

### note · agent:manager-2 · 2026-10-17T06:14:29.842Z
integrated: fa5444437150694264b419751dd2d0f10ce5a40f (branch bridle/r8jf)
