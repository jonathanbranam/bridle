+++
id = "br-0adb"
title = "Docs: the whole spec flow, and using it from a project"
kind = "chore"
state = "open"
created_at = "2026-09-29T04:35:08.415Z"
updated_at = "2026-09-29T04:35:08.415Z"
+++

Goal: one page a project can follow, after the P3 commands have merged (spec check/id/export/import openspec, goals list, arch list, explore check/new/conclude). Write docs/design/spec-flow.md (link it from docs/README.md and specs.md): the order to adopt (import openspec -> spec id -> spec check --require-ids), how a project wires `bridle spec check` into its own check/CI (example justfile / pre-commit line), how an adapter consumes `bridle spec export --format json` (data-contracts: replaces tools/spec-to-feature.py, see docs/design/specs-to-tests.md and docs/context/onboarding-data-contracts.md), goals and architecture file layout, explore workflow. Read the built commands' --help to be accurate; describe unbuilt commands only in a clearly-labelled 'not yet' list. Acceptance: `just check` passes; follow workflow/base/rules/doc-links.md. Model: Haiku. Out of scope: code changes. Run after br-7536 merges.
