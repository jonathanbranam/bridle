+++
id = "br-a54d"
title = "P6: TypeScript vitest adapter for bridle specs"
kind = "feature"
state = "planned"
created_at = "2026-09-29T09:12:21.856Z"
updated_at = "2026-09-29T09:12:24.000628Z"
+++

Goal (docs/design/specs-to-tests.md): the typescript stack pack gets the vitest equivalent of the Python adapter, for track-web (uses vitest), harness, otters, file-db. Put it in workflow/packs/typescript/adapters/vitest-bridle/ (create the pack dir if absent, following workflow/packs/python/ and workflow-layers.md): a small ESM/TS module exporting `registerBridleSpecs({ root?, capability?, scenarios?, steps })` that runs `bridle spec export --format json` (bridle from env BRIDLE_BIN or PATH; execFileSync) and, for each EXECUTABLE scenario, calls vitest's describe (per requirement) / it (per scenario, name contains the id 's-b310' and tags as [tag] suffixes) with a step registry: `steps.given(/regex/, fn)`, `when`, `then` matched against each step's text (And/But inherit the previous keyword); an unmatched step fails that test with a message naming the step and scenario id; Scenario Outline examples run once per row. Env BRIDLE_SPEC_SCENARIOS=s-1,s-2 filters. Export refusal (spec errors) throws with the diagnostics.

Learn the consumer: read bridle's docs/questions/open/onboarding-survey-track-web-and-harness-u8sm.md and docs/context/ for track-web's current test setup (read-only; never modify /Volumes/Data/work/track-web or any project). Own tests in the adapter dir using vitest against a tiny fixture spec dir; they must be skipped by `just check` if node/vitest/bridle aren't available (say in the thread whether they ran; use a node_modules-free approach if possible, e.g. node --test on plain JS, so no npm install is needed). README with install/usage. Update specs-to-tests.md and build-order.md P6 row note.

Acceptance: just check passes. Model: Sonnet. Out of scope: migrating any project, spec coverage.
