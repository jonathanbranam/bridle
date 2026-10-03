+++
id = "br-ae60"
title = "P3-1: spec parser crate (bridle-spec): parse capability spec files into an AST with strict diagnostics"
kind = "feature"
state = "integrated"
created_at = "2026-09-29T03:21:22.036Z"
updated_at = "2026-09-29T03:34:59.069007Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/spec-parser"
commit = "38c020b"
summary = "New pure crate crates/bridle-spec: parse_str(file, text) / parse_file(path) -> Result<Spec, Vec<Diagnostic>> (file, 1-based line/col, 'expected X, found Y'). Model: Spec{title, requirements{title,id,protected,traces,text,line,scenarios{title,id,verification,tags,description,steps,examples}}}. Grammar mirrors spec-to-feature.py input layer (marker near-misses, step keywords, markup/placeholder escaping, Examples tables, fences/comments) plus {#id protected traces=a-12cd@3f9e} attrs (ids lowercase hex 4+, dup ids, unknown flags), requirement without scenario. Not ported: pytest tag registry (tags only syntax-checked). Non-executable bodies unparsed (steps empty). Goals/architecture tiers left alone. All six data-contracts specs copied unchanged as fixtures and parse; timing test included. Full just check passes."
+++

Goal: the one spec parser every later P3 command uses (check, ids, export, import, impact). Design: docs/design/specs.md, specs-to-tests.md, traceability.md (link syntax), goals-tier.md and architecture-tier.md (the same id-in-heading style, parse these too only if trivial; otherwise leave them), and docs/context/onboarding-data-contracts.md sections on the spec tools. Do: a new pure library crate crates/bridle-spec (no daemon, no async, no I/O beyond a path-taking convenience fn) that parses a design/specs/<capability>.md text into: capability title, requirements (title, SHALL text, id r-xxxx if present, flags such as protected, traces=<id>@<hash>) and scenarios (title, id s-xxxx, the Verification line: executable or non-executable, @tags, GIVEN/WHEN/THEN/AND steps with their text). Grammar: '### Requirement: <title> {#r-7fa2 protected traces=a-12cd@3f9e}', '#### Scenario: <title> {#s-b310}', '*Verification*: **executable** @tag' as the first line of a scenario, '- **GIVEN** ...' steps. Match data-contracts' current grammar exactly, which is defined by its Python tool: read-only reference in /Volumes/Data/work/data-contracts-workspace/data-contracts (tools/spec-to-feature.py, and openspec/specs/*/spec.md as real fixtures). Never write there. Copy two or three of its real spec files into crates/bridle-spec/tests/fixtures/ (unchanged) and test that they parse. Every diagnostic carries file, line and column, a message saying what was expected and what was found, and strict rejection of near-misses (wrong heading level, missing Verification line, unknown step keyword, duplicate ids within a file, requirement without a scenario) : that strictness is the value of the old tool. Parse everything and return all diagnostics, not the first. Target: parse all data-contracts specs in well under a second (add a simple timing test with generous bound). Acceptance: just check passes (add the crate to the workspace); unit tests for each diagnostic, round trip of ids and flags, fixtures parse. Model: Sonnet. Out of scope: any CLI, id assignment, export, import, hashing of upstream text, writing files. Keep the public API small and documented; the other P3 tasks are written against it.

## Thread

### note · agent:manager-2 · 2026-09-29T03:34:59.069Z
integrated: 38c020b (branch bridle/spec-parser)
