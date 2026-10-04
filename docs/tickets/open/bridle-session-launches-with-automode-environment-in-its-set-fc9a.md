---
id: fc9a
title: bridle session launches with autoMode.environment in its --settings (ufrw, conservative variant)
kind: feature
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [ufrw]
tasks: []
---

## The ask

Build the recommendation in [[auto-mode-s-classifier-context-per-machine-for-bridle-projec-ufrw|ufrw]] ("Recommendation", "Smallest first slice"), **conservative variant**: trust lines (repos, domains, buckets) come only from the machine file `~/.bridle/config.toml` `[auto_mode] environment`; a project's `.bridle/config.toml` `[auto_mode] environment` may only tighten (sensitive targets, prod hosts). Bridle never writes `~/.claude/settings.json`. `bridle auto-mode print` and spawn-time `autoMode` stay deferred, as ufrw says.

The human, 2026-10-04 (task br-bedz): "bedz - yes conservative approved".
