+++
id = "br-tgxy"
title = "A rule and role text make agents verify UI work in a browser on a throwaway site"
kind = "feature"
state = "built"
created_at = "2026-10-13T02:00:28.123Z"
updated_at = "2026-10-13T02:52:01.364040Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wtgxy",
]
branch = "bridle/wtgxy"
commit = "8ea1f03b"
summary = "Added rule workflow/base/rules/verify-ui-in-a-browser.md (worker clicks through; manager brings up its own site from the branch and clicks through before merging), one line each in worker.md and manager.md, migration 0007-gitignore-agent-site (automatic, guarded, idempotent; unit-tested for dry-run and rerun), CLAUDE.md Verification section, dev-site.md section 3/4 and migrations.md updated, CHANGELOG line. Caveat: the bridle workflow update/sync/migrate live run on a scratch project was not done (needs a running scratch daemon); covered by the unit test."
ticket = "tgxy"
+++

docs/tickets/open/a-rule-and-role-text-make-agents-verify-ui-work-in-a-browser-tgxy.md

## Thread

### note · external:orchestrator · 2026-10-13T02:00:47.382Z
Readied by orchestrator on the human's go, via advisor product-manager (m-11853), 2026-10-12 ~9:15 PM ET: "Yes that would be good to get done": schedule vdu8, dgef (UI half, on bridle-ui) and tgxy right after usage epic rdkn. Ask: br-vwqt.

### note · agent:pm-1 · 2026-10-13T02:01:25.636Z
Brief (pm-1). Model: Sonnet. Size: S-M. The ask above, including the 2026-10-12 addition (the merger clicks through before merging), and docs/design/dev-site.md sections 3 and 4 are the spec. Blocked by br-vdu8: the rule tells agents to run `bridle dev site`, so it should land with the command, not before. Files: workflow/base/rules/verify-ui-in-a-browser.md (new), worker.md and manager.md (one line each), the .gitignore migration (find how existing migrations are registered, e.g. `bridle migrate`; idempotent; second run reports nothing; follows ticket xebc), bridle's CLAUDE.md "Verification" note, docs/design/dev-site.md section 3, CHANGELOG.md. Existing projects: the rule reaches them through `bridle workflow update` plus `bridle sync`, the .gitignore line through the migration; nothing in track-web's repo changes (rule existing-projects: it is a trial branch, never main). Acceptance: the Verify line in the ask on a scratch project; `just check` passes. Out of scope: the command itself, the UI half.

### note · agent:wtgxy · 2026-10-13T02:51:59.032Z
built: 8ea1f03b (branch bridle/wtgxy)

### note · agent:wtgxy · 2026-10-13T02:52:01.364Z
done: rule verify-ui-in-a-browser + worker/manager lines + migration 0007 (.agent-site/ in .gitignore) + docs/CHANGELOG; just check exit 0, 1637 tests passed (6 skipped), sha 8ea1f03b (main already merged, up to date). Caveat: live 'workflow update/sync/migrate' on a scratch project not run (needs a scratch daemon); migration covered by a unit test (dry-run, append, rerun clean).
