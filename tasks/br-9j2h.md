+++
id = "br-9j2h"
title = "Rename the product-manager role to project-manager (proj-mgr), with a reviewed migration for existing projects"
kind = "feature"
state = "planned"
created_at = "2026-10-04T12:26:59.333Z"
updated_at = "2026-10-04T12:29:22.769971Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: 9j2h
Rename the product-manager role to project-manager (proj-mgr where a short name is needed). Ticket: docs/tickets/open/rename-the-product-manager-role-to-project-manager-proj-mgr-9j2h.md (read it; parent decision 7r2c). Approved by the human 2026-10-02: 'Yeah let's rename it to project manager can be shortened to proj-mgr if needed in naming.' (br-8b6c, dropped in the k7tm sort, was approved after all.)
Build: (1) rename the role: workflow/base/roles/product-manager.md -> project-manager.md; every 'product-manager' occurrence (about 88 in 55 files: the role lists in workflow/base/rules/*.md, .bridle/config.toml, .bridle/roles/orchestrator.md, crates/bridle-daemon/src/{config,server,supervisor}.rs incl. the PM-or-human queue gate require_pm_or_human, crates/bridle/src/{prime.rs,commands/orchestrator.rs}, docs/design/*, docs/README.md, docs/context/naming.md, docs/briefs/tasks.md). Leave history alone: CHANGELOG past entries, docs/spikes, docs/context/incidents.md and resolved tickets keep the old name. Agent names like pm-1 stay. Prose 'product manager' in current docs/role text becomes 'project manager'. (2) A project migration, using the mechanism in crates/bridle/src/migrate.rs and docs/design/migrations.md (a new entry in its MIGRATIONS list), that renames the role in an existing project's .bridle/config.toml and role overrides (.bridle/roles/product-manager.md -> project-manager.md, config keys/values); idempotent; refuses on uncommitted changes in files it edits, as the others do. Does the config accept the old name as an alias for one release? Decide the smaller option and say which in the docs. (3) Bridle's own repo is migrated in this build (run the migration on it, commit the result).
HARD RULE (existing-projects): do NOT run the migration on meta-notes, track-web or any other existing project, and don't touch their files. The human reviews it first. Show the human what it does via the migration's docs and a test fixture. Automatic start-up migrations (br-2718) are parked and not on main; if they land before this does, this migration must be flagged manual_only. Say that in a note in docs/design/migrations.md.
Tests: migration renames config and role file, is idempotent, refuses with uncommitted edits; a role named project-manager resolves and gets the queue gate; just check passes. Docs: migrations.md, roles-and-config.md, CHANGELOG. Model: Sonnet. Out of scope: the product-partner idea (7r2c option 2), renaming the advisor.

## Thread

### note · external:orchestrator · 2026-10-04T12:27:00.716Z
Approved by the human 2026-10-02 (ticket 7r2c, Decided): 'Yeah let's rename it to project manager can be shortened to proj-mgr if needed in naming.' Migration of meta-notes/track-web needs the human's review before it runs there.
