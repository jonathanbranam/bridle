+++
id = "br-jxm5"
title = "Gateway TS bindings: export the Spec* types (br-75zr follow-up) so bridle-ui can use the specs routes"
kind = "bug"
state = "open"
created_at = "2026-10-05T13:39:06.809Z"
updated_at = "2026-10-05T13:39:06.832756Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

br-75zr added the gateway specs routes, but the Spec* types in crates/bridle-gateway/src/specs.rs (SpecFile, SpecRequirement, SpecScenario, SpecDiagnostic, and any listing/response types the specs and links/resolve routes return) are not in export_all in crates/bridle-gateway/src/types.rs, so 'just gateway-types' doesn't emit them. bridle-ui's rule is no hand-written types, so ui-7sag (Specs index on the listing route) is blocked (manager-1, bridle-ui m-0415). Fix: add every type the specs routes put on the wire to export_all, regenerate the bindings the way gateway-types does, add/extend the test that checks export coverage if one exists. Acceptance: just check passes; the generated bindings include the spec types. Small; Sonnet. Within the human's overnight ask (br-75zr's scope).
