+++
id = "br-wrmu"
title = "Ticket migration (syqn part 3): rename ticket frontmatter fields and qualify IDs in every project, as an xebc migration on trial branches only"
kind = "feature"
state = "planned"
created_at = "2026-10-11T02:19:12.766Z"
updated_at = "2026-10-11T02:19:43.968623Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
]
parent = "br-syqn"
+++

Ticket: docs/tickets/open/ticket-fields-get-standard-names-blocked-by-related-parent-c-syqn.md (migration bullet) and br-22ab's section 11 (migration, step 1). Blocked by br-epvy (part 2, qualified IDs); part 1 (br-syqn) is integrated. Goal: an xebc migration (crates/bridle/src/migrate.rs: Migration, Ctx, apply, refuse_if_uncommitted; follow rename_product_manager as the model, and read the xebc ticket and docs on migrations first) that, in a project docs/tickets/**, renames frontmatter fields needs->blocked_by, see->related, opened->created, filed_by->created_by, closed->resolved and rewrites id: and every reference (blocked_by, related, parent, tasks, wiki links that carry a bare ID in frontmatter) to the project-qualified form from part 2. Idempotent (a ticket already new-form is skipped), refuses on uncommitted ticket files, writes one commit. RULE existing-projects (must): it never runs against another project main or dev; it commits to the project trial or integration branch only, and bridle own repo is not an existing project in that sense (its merge model applies). The runner must refuse on a protected branch for projects other than bridle: check how migrate.rs and the trial-adoption work (ticket 63rv) decide that, and reuse it; if there is no such guard yet, say so on the task and ask via bridle task ask before building around it. Then run it on this repo as part of the change (a separate commit) so bridle ticket check stays clean, and say how a project owner runs it for theirs (bridle migrate or the existing command). Files: crates/bridle/src/migrate.rs and the migration list, tests, docs/design/cli.md or the migration doc, docs/README.md ticket conventions, CHANGELOG. Acceptance: just check passes; tests on a temp repo: old-form tickets migrate, new-form skipped, rerun is a no-op, uncommitted refused, refused on a protected branch of an existing project; bridle ticket check clean after running on this repo. Migration: this IS the migration; readers keep accepting old forms until all projects ran it (do not remove them here). Model: Sonnet. Out of scope: removing old-form reading, ticket types, thread move.
