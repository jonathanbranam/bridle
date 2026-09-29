// vitest adapter for bridle specs: registers one vitest test per executable
// scenario, bound to a step registry. See README.md.
import { execFileSync } from "node:child_process";

const KEYWORDS = ["given", "when", "then"];

/** The step registry: `given/when/then(regex, fn)`. */
export function createSteps() {
  const defs = { given: [], when: [], then: [] };
  const reg = (kind) => (pattern, fn) => {
    defs[kind].push({ pattern, fn });
  };
  return { given: reg("given"), when: reg("when"), then: reg("then"), defs };
}

/** Run `bridle spec export --format json`; throws with the diagnostics on refusal. */
export function exportSpecs({ root, bridle = process.env.BRIDLE_BIN || "bridle", cwd } = {}) {
  const args = ["spec", "export", "--format", "json"];
  if (root) args.push("--root", root);
  try {
    return JSON.parse(
      execFileSync(bridle, args, {
        cwd,
        encoding: "utf8",
        maxBuffer: 256 * 1024 * 1024,
        stdio: ["ignore", "pipe", "pipe"],
      }),
    );
  } catch (e) {
    const diag = e.stderr ? String(e.stderr).trim() : e.message;
    throw new Error(`bridle spec export failed:\n${diag}`);
  }
}

function fill(text, header, row) {
  return text.replace(/<([^<>]+)>/g, (m, name) => {
    const i = header.indexOf(name);
    return i < 0 ? m : row[i];
  });
}

async function runSteps(sc, header, row, defs) {
  const world = {};
  let kind = null;
  for (const step of sc.steps) {
    const k = step.keyword.toLowerCase();
    if (KEYWORDS.includes(k)) kind = k; // And/But inherit
    const text = header ? fill(step.text, header, row) : step.text;
    let hit = null;
    for (const d of defs[kind]) {
      const m = d.pattern.exec(text);
      if (m && m[0] === text) {
        hit = { d, m };
        break;
      }
    }
    if (!hit) {
      throw new Error(
        `no step definition for ${step.keyword} "${text}" (scenario ${sc.id}, line ${step.line})`,
      );
    }
    await hit.d.fn(world, ...hit.m.slice(1));
  }
}

/**
 * Register the executable scenarios as vitest tests.
 * `runner` ({describe, it}) defaults to vitest's; tests inject their own.
 */
export async function registerBridleSpecs({
  root,
  capability,
  scenarios,
  steps,
  runner,
  bridle,
  cwd,
} = {}) {
  if (!steps || !steps.defs) throw new Error("registerBridleSpecs: `steps` (createSteps()) is required");
  const { describe, it } = runner ?? (await import("vitest"));
  const wanted = scenarios ?? (process.env.BRIDLE_SPEC_SCENARIOS
    ? process.env.BRIDLE_SPEC_SCENARIOS.split(",").map((s) => s.trim()).filter(Boolean)
    : null);
  const doc = exportSpecs({ root, bridle, cwd });
  for (const spec of doc.specs) {
    if (capability && spec.capability !== capability) continue;
    for (const req of spec.requirements) {
      const scs = req.scenarios.filter(
        (s) => s.executable && (!wanted || (s.id && wanted.includes(s.id))),
      );
      if (!scs.length) continue;
      describe(`${spec.capability}: ${req.title}`, () => {
        for (const sc of scs) {
          const tags = sc.tags.map((t) => ` [${t}]`).join("");
          const name = `${sc.id ?? "s-????"} ${sc.title}${tags}`;
          if (sc.examples) {
            const { header, rows } = sc.examples;
            for (const row of rows) {
              it(`${name} (${header.map((h, i) => `${h}=${row[i]}`).join(", ")})`, () =>
                runSteps(sc, header, row, steps.defs));
            }
          } else {
            it(name, () => runSteps(sc, null, null, steps.defs));
          }
        }
      });
    }
  }
}
