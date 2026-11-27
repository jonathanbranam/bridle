+++
id = "br-ysmy"
title = "Daemons fetching a release workflow at once share one scratch dir and can fail"
kind = "bug"
state = "built"
created_at = "2026-11-27T15:51:19.099Z"
updated_at = "2026-11-27T16:22:58.842948Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:wysmy",
]
priority = "low"
priority_at = "2026-11-27T15:51:20.262964Z"
branch = "bridle/wysmy"
commit = "408aeb67"
summary = "workflow_checkout::ensure (crates/bridle-daemon/src/workflow_checkout.rs) no longer uses a shared scratch dir. Each call clones into its own tempdir under ~/.bridle/workflow (prefix .fetching-<tag>-), then renames the clone to <home>/workflow/<tag>. If the rename fails and the target is already a complete checkout (another daemon won), that is success; otherwise the error is returned. The old pre-clone remove_dir_all of the target is gone: it could delete a checkout another daemon had just renamed into place. A new test runs six concurrent ensure calls of one tag against a temp home and asserts all succeed, the checkout is whole, and no scratch dir is left. Same pattern as install_with (br-kf7a). Caveat: a half checkout left by an older build under the tag name is not cleaned up automatically; the rename-onto-it then fails with an error that is retried at the next start."
ticket = "ysmy"
+++

docs/tickets/open/workflow-fetch-race-ysmy.md

## Thread

### note · agent:pm-1 · 2026-11-27T15:51:20.262Z
priority: normal -> low

### note · external:orchestrator · 2026-11-27T15:51:40.836Z
readied by external:orchestrator

### note · external:orchestrator · 2026-11-27T15:51:40.962Z
Readied by orchestrator (dalek): low bug, self-evident fix (rule ready-without-asking).

### note · agent:wysmy · 2026-11-27T16:22:49.344Z
built: 408aeb67 (branch bridle/wysmy)

### note · agent:wysmy · 2026-11-27T16:22:49.633Z
done: workflow_checkout::ensure uses a per-call tempdir scratch and renames into place; a placed checkout counts as success; concurrent-ensure test added; CHANGELOG under Fixed; main merged; commit 408aeb67; just check exit 0, 2045 tests run, 2045 passed, 6 skipped

### note · agent:manager-2 · 2026-11-27T16:22:58.842Z
Review: one regression before I land. You removed remove_dir_all(dest), so an incomplete dir already under the tag name (e.g. an interrupted prune or an older build) now fails every fetch for good, not just once: nothing ever removes it. Fix: when the rename fails and dest has no workflow/base, move dest aside into your own scratch dir (rename, so it is safe: complete checkouts only appear atomically), retry the rename once, and treat a complete dest as success. Add a test: an incomplete dest dir with a stray file is replaced. Then merge main, re-run just check and message me the new sha, exit status and count.
