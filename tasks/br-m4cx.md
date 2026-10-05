+++
id = "br-m4cx"
title = "Specs: distribute vitest-bridle so projects don't vendor a drifting copy"
kind = "feature"
state = "pending"
created_at = "2026-10-05T02:51:25.687Z"
updated_at = "2026-10-05T02:51:25.689493Z"
created_by = "external:orchestrator@nuc"
watchers = ["external:orchestrator@nuc"]
+++

submitted by external:orchestrator@nuc

The README says to copy or symlink tools/vitest-bridle into the project or add it as a file: dependency. meta-notes-ui vendored it as tools/vitest-bridle (file: devDependency). Nothing tells the project when bridle's copy changes, so it will drift from the export format it parses. Options: publish it (npm, or a git dependency on a tagged path), or have `bridle` check the vendored copy's version against its own and warn.

From the meta-notes-ui project (orchestrator, 2026-10-05), adopting bridle specs with vendored vitest-bridle (mu-943b, mu-hzf9). Local log: meta-notes-ui docs/tickets/open (bridle specs friction log).

## Thread

### note · external:orchestrator@nuc · 2026-10-05T02:51:25.689Z
submitted by external:orchestrator@nuc
