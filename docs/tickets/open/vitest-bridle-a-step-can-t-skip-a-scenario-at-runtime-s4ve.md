---
id: s4ve
title: "vitest-bridle: a step can't skip a scenario at runtime"
kind: feature
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-s4ve]
---

## The ask

## What happened

meta-notes-ui (mu-hxvt, 2026-10-05): a scenario that depends on an environment step (meta-notes init building a .venv) must skip with a reason when that step fails. vitest-bridle has no way to do that, so the project patched its vendored tools/vitest-bridle: a step throwing an error with skipScenario:true calls ctx.skip(reason). The vendored copy now differs from bridle's (see br-m4cx: no signal when bridle's copy changes).

## The ask

- A supported runtime skip in bridle's vitest-bridle (ctx.skip(reason) exposed to steps, or a SkipScenario error; pick the smallest), so projects do not fork it. Adopt the meta-notes-ui patch's shape if it is sound, so their vendored copy can be replaced by bridle's.
- A per-scenario timeout (a ~5s init needed the spec timeout raised to 60s). Small; drop it from this ticket if it grows.

Find vitest-bridle in the repo (grep for vitest-bridle). Migration: projects with a vendored copy get the new one the way br-m4cx settles; note in the ticket how meta-notes-ui drops its patch. Verify: just check, plus a test where a step skips its scenario with a reason.
