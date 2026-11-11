---
id: br-wphz
title: A tools-only clone's refusal names the machine that owns the project
kind: bug
created: 2026-11-11
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

Filed by aide. Related: br-hw6c (one machine owns a project).

The human, 2026-11-11 (to aide, relayed by orchestrator@remus in m-18515), verbatim: "I tried to run `bridle session aid` on Remus for data contracts, and it gave me this output. Unless I'm missing it, I don't think it says which machine owns this, which we should know from the config, right? It knows it's not this machine, so no problem here exactly, but this error message needs to indicate and say which machine owns this."

The output on remus:

    error: /home/jbranam/work/data-contracts-work/data-contracts is a tools-only clone on this machine ([machine] tools_only in /home/jbranam/.bridle/config.toml): it is not the project's home. Run in the served project's workspace instead.

The message comes from crates/bridle/src/tools_only.rs:32. data-contracts is owned by dalek (`owner.toml` on `bridle/state`: host = "dalek", since 2026-10-18).

## The ask

When a command refuses in a tools-only clone, the error names the machine that owns the project (from `owner.toml` on `bridle/state`, local copy if origin can't be reached) and, if it's known, the workspace path there. Example: "... it is not the project's home: data-contracts is served on dalek (/Volumes/Data/work/data-contracts-workspace). Run it there." If the owner can't be read, say so instead of leaving it out. Cover every command that hits this refusal, not only `session aide`. Test: the message includes the owner from a test owner.toml.
