+++
id = "br-5nk5"
title = "stage-land refuses a clean merge when the user's git config says merge.ff=only"
kind = "bug"
state = "built"
created_at = "2026-10-19T11:00:02.919Z"
updated_at = "2026-10-19T20:07:27.870960Z"
created_by = "external:advisor/tj7q"
watchers = [
    "external:advisor/tj7q",
    "agent:w5nk5",
]
branch = "bridle/w5nk5"
commit = "7e39fc68"
summary = "stage-land's merge into main now runs with -c merge.ff=true (as integrator::land does), so a user's merge.ff=only no longer refuses a clean merge of a stage branch that is behind main. A real conflict still fails back to the writer. Two tests in crates/bridle-daemon/src/stage_land.rs cover the clean case and the conflict case, both under merge.ff=only. CHANGELOG Fixed line added. No doc change needed. just check on 7e39fc68: exit 0, 1894 run, 1894 passed, 6 skipped."
ticket = "5nk5"
+++

docs/tickets/open/stage-land-refuses-a-clean-merge-when-the-user-s-git-config-5nk5.md

## Thread

### note · agent:pm-1 · 2026-10-19T11:00:56.975Z
Brief (pm-1). Model: Haiku. Do the ask as written: in crates/bridle-daemon/src/stage_land.rs make the merge ignore the user's merge.ff (same guard and comment style as integrator::land). Test: branch behind main with merge.ff=only in the repo config lands cleanly; a real conflict still fails back to the writer. Acceptance: just check. Priority high: it makes every stage land need a manual rebase in the human's Stages try.

### note · agent:w5nk5 · 2026-10-19T14:11:53.948Z
built: 7e39fc68 (branch bridle/w5nk5)

### note · agent:w5nk5 · 2026-10-19T14:11:54.619Z
done: stage-land merge now uses -c merge.ff=true; two tests (clean merge behind main under merge.ff=only lands; conflict under it still fails to the writer); just check exit 0, 1894 run, 1894 passed, 6 skipped; tip 7e39fc68 on bridle/w5nk5 (main already contained, no merge needed)

### note · agent:manager-2 · 2026-10-19T14:12:01.589Z
Diff looks good. The ticket has no summary yet: write it with 'bridle ticket summary br-5nk5' (a few lines, becomes the landing commit body), then message me. No code change needed.

### note · agent:w5nk5 · 2026-10-19T14:12:16.208Z
summary written on br-5nk5 (stored, the daemon returns it on the ticket). Tip unchanged: 7e39fc68 on bridle/w5nk5; just check on it exit 0, 1894 run, 1894 passed, 6 skipped.

### note · agent:w5nk5 · 2026-10-19T20:07:27.870Z
done: merged main (dfe7eac3, br-stht) into bridle/w5nk5; tip 8683f0a8; just check exit 0, 1894 run, 1894 passed, 6 skipped; the stage-land diff is unchanged
