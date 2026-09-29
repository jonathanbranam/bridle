# vitest-bridle

Registers bridle's executable spec scenarios as vitest tests, the TypeScript
counterpart of the python pack's pytest plugin (`docs/design/specs-to-tests.md`).
Nothing generated is committed: scenarios come from `bridle spec export --format json`
at collection time.

## Install

Copy or symlink this directory into the project (or add it as a `file:` dependency);
it needs only `vitest` and a `bridle` binary (`BRIDLE_BIN`, else `PATH`).

## Use

```ts
// specs.test.ts
import { registerBridleSpecs, createSteps } from "vitest-bridle";

const steps = createSteps();
steps.given(/^two widgets$/, (world) => { world.n = 2; });
steps.when(/^they are counted$/, (world) => { world.count = world.n; });
steps.then(/^the count is (\d+)$/, (world, n) => expect(world.count).toBe(Number(n)));

await registerBridleSpecs({ steps }); // options: root, capability, scenarios
```

- One `describe` per requirement, one `it` per executable scenario, named
  `s-b310 Title [tag]`; a Scenario Outline runs once per example row
  (`<col>` is substituted into the step text before matching).
- `given`/`when`/`then` take a regex that must match the whole step text;
  captures are passed after the per-scenario `world` object. `And`/`But` use the
  previous keyword. An unmatched step fails that test, naming the step and scenario id.
- `BRIDLE_SPEC_SCENARIOS=s-b310,s-b312` (or `scenarios: [...]`) runs only those.
- If any spec has errors, `bridle spec export` refuses and registration throws
  with its diagnostics. Run vitest from the project root (or pass `root`).

## Tests

`node --test test/adapter.test.mjs` (no npm install; the vitest runner is injected). `just check`
runs them through `crates/bridle/tests/ts_adapter.rs`, skipping when `node` is absent.
