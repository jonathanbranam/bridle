// Plain node:test, no npm install: the vitest runner is injected as a recorder.
import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { registerBridleSpecs, createSteps } from "../index.mjs";

const bridle = process.env.BRIDLE_BIN || "bridle";
const root = fileURLToPath(new URL("./fixture-design/specs", import.meta.url));
let have = true;
try {
  execFileSync(bridle, ["--version"], { stdio: "ignore" });
} catch {
  have = false;
}
const opts = { skip: !have && "bridle not available" };

function recorder() {
  const suites = [];
  const tests = [];
  return {
    suites,
    tests,
    describe: (name, fn) => {
      suites.push(name);
      fn();
    },
    it: (name, fn) => tests.push({ name, fn }),
  };
}

function steps(log) {
  const s = createSteps();
  s.given(/two widgets/, (w) => (w.n = 2));
  s.when(/they are counted/, () => log.push("counted"));
  s.then(/the count is (\d+)/, (w, n) => assert.equal(w.n, Number(n)));
  s.when(/a widget is coloured (\w+)/, (w, c) => (w.c = c));
  s.then(/it reports (\w+)/, (w, c) => assert.equal(w.c, c));
  return s;
}

test("registers executable scenarios with ids, tags and example rows", opts, async () => {
  const r = recorder();
  await registerBridleSpecs({ root, steps: steps([]), runner: r });
  assert.deepEqual(r.suites, ["widgets: Widgets are counted", "widgets: Widgets have colours"]);
  assert.deepEqual(
    r.tests.map((t) => t.name),
    [
      "s-b310 Counting two widgets [smoke]",
      "s-b312 Colouring (colour=red)",
      "s-b312 Colouring (colour=blue)",
    ],
  );
  for (const t of r.tests) await t.fn();
});

test("BRIDLE_SPEC_SCENARIOS filters", opts, async () => {
  const r = recorder();
  await registerBridleSpecs({ root, steps: steps([]), runner: r, scenarios: ["s-b310"] });
  assert.equal(r.tests.length, 1);
});

test("unmatched step fails naming step and scenario id", opts, async () => {
  const r = recorder();
  const s = createSteps();
  s.given(/two widgets/, () => {});
  await registerBridleSpecs({ root, steps: s, runner: r, scenarios: ["s-b310"] });
  await assert.rejects(r.tests[0].fn(), /they are counted.*s-b310/);
});

test("export refusal throws with diagnostics", opts, async () => {
  const { mkdtempSync, writeFileSync } = await import("node:fs");
  const { tmpdir } = await import("node:os");
  const tmp = mkdtempSync(`${tmpdir()}/vb-`);
  writeFileSync(`${tmp}/bad.md`, "## Requirements\n\n### Requirement: A {#nope}\n\ntext\n");
  await assert.rejects(
    registerBridleSpecs({ root: tmp, steps: createSteps(), runner: recorder() }),
    /bad\.md:3:/,
  );
});
