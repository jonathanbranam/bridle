+++
id = "br-32ea"
title = "k7mw-a: machine/project config and client routing, tokens keyed by machine"
kind = "feature"
state = "integrated"
created_at = "2026-09-30T11:04:27.077Z"
updated_at = "2026-09-30T11:19:46.872364Z"
branch = "bridle/k7mw-a"
commit = "2698df28cfae4ef7b350da33d888ca932c6504fa"
summary = "Machine config types in bridle-api/src/machines.rs (MachineMap: [machine] name, [machines], [projects]; slice C reuses ProjectPlace.port; the daemon's RawConfig accepts the new keys). resolve_endpoint routes a --project/BRIDLE_PROJECT listed on another machine to http://host:port (Endpoint.machine set), else the registry; a listed project with no [machine] name errors. resolve_token takes the machine and reads [principal.<machine>][project]; plain [principal] keys are unchanged. 'This machine' = [machine] name. Docs: cli.md, principals.md, CHANGELOG."
+++

Slice A of k7mw (br-36c7; design in docs/tickets/open/projects-on-other-machines-by-config-k7mw.md, read it fully). Goal: hand-written config and client side. (1) Config: [machines] name=host and [projects] name={machine,port} in ~/.bridle/config.toml, parsed into bridle-daemon config (shared type others will reuse for the listen port). Need a way to know 'this machine' (name: config key matching hostname, or a top-level machine="mbp" setting; pick the simplest and document). (2) Client: --project X (and BRIDLE_PROJECT if present) resolves to http://<host>:<port> when the project's machine isn't this one, else the local registry as today. Trust the config: no probing, no owner.toml. (3) credentials.toml: plain [principal] tables stay 'this machine'; [principal.<machine>] sub-tables hold that machine's daemons' tokens; CLI picks the entry for the machine the config names; existing files keep working. BRIDLE_AS=orchestrator against another machine uses principal 'orchestrator@<this machine>' credentials from [orchestrator.<machine>]... check the design section 3-4 and follow it. Files: crates/bridle-daemon config, crates/bridle-api discovery/credentials, crates/bridle CLI. Tests: config parsing, routing to remote vs local, credentials lookup incl. old-format files. Docs: cli.md, config docs, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: the daemon binding its port (slice C), token minting/@ names (slice B), discovery, forwarding. Touches config types: slice C waits for this. Runs alongside slice B.

## Thread

### note · agent:k7mw-a · 2026-09-30T11:19:42.180Z
done: machine config + remote routing + machine-keyed credentials; just check passes (872 tests); 624ebc9

### note · agent:manager-2 · 2026-09-30T11:19:46.872Z
integrated: 2698df28cfae4ef7b350da33d888ca932c6504fa (branch bridle/k7mw-a)
