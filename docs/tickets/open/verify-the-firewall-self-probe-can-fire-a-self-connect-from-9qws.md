---
id: br-9qws
title: "Verify the firewall self-probe can fire: a self-connect from the same host may never be filtered"
kind: fix
created: 2026-10-16
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: [br-t9j4]
related: []
---

## The ask

From the orchestrator's verification (br-7bs2/br-t9j4). The self-probe (probe_own_address in crates/bridle-daemon lib.rs, and crates/bridle-gateway reachable.rs) connects to its own tailnet address from the same host. It is unverified whether the macOS application firewall filters a self-connect at all, so the probe may never fire even when the binary is blocked.

Do: on a Mac, with a deliberately blocked binary (launch with a different identifier so the firewall dialog is pending), check whether the self-probe fails. Write the finding on the thread. If it passes while blocked, change the probe to come from a peer (for example, ask a peer daemon over the tailnet to connect back, or reuse an existing peer call) and keep the incident. Needs the human's Mac for the blocked-binary test: ask via the manager, do not disable the firewall. Blocked-by br-t9j4 (same files). Acceptance: finding recorded; if changed, just check and tests. Model: Sonnet.
