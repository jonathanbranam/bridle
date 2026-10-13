---
id: br-tgxy
title: A rule and role text make agents verify UI work in a browser on a throwaway site
kind: feature
created: 2026-10-06
repos: [bridle]
changes: []
specs: []
blocked_by: [br-vdu8]
related: [br-vwqt]
resolved: 2026-10-13T02:52:36Z
---

## The ask

Section 3 and 4 of [[docs/design/dev-site|dev-site]]. Add `workflow/base/rules/verify-ui-in-a-browser.md` (text in the design), the one-line additions to `worker.md` and `manager.md`, a migration that appends `.agent-site/` to a project's `.gitignore` (idempotent; follows ticket xebc's automatic-by-default direction), and a "Verification" note in bridle's CLAUDE.md. track-web gets the rule only; nothing in its repo changes (rule existing-projects).

Verify: `bridle workflow update` plus `bridle sync` in a scratch project renders the rule; `bridle migrate --dry-run` reports the gitignore line and a second run reports nothing.

## Added 2026-10-12: someone clicks around before merge

The human (2026-10-12, verbatim in vwqt): "Probably even the worker agent should be able to do that, the one who's actually building it, but definitely before something's merged, somebody needs to be able to click around on the site."

The design has the worker click through and the manager only check the worker's done line. Change it: before merging a UI task, the merger (manager) brings up its own throwaway site from the branch and clicks through the acceptance items itself, as rule mergers-must-verify-checks-themselves (zk4p) does for checks. Update the rule text, manager.md and docs/design/dev-site.md section 3 to match.
