+++
id = "br-5cru"
title = "Commands that run bridle on another machine over ssh fail unless the user adds bridle to the non-interactive PATH"
kind = "bug"
state = "built"
created_at = "2026-10-14T01:20:43.432Z"
updated_at = "2026-10-17T01:37:32.922686Z"
created_by = "external:advisor/wsl2"
watchers = []
+++



## Thread

### note · external:advisor/product-manager · 2026-10-14T11:42:19.869Z
The human, 2026-10-14 ~7:45 AM ET (to advisor product-manager), on option B (add ~/.local/bin and ~/.cargo/bin to the ssh remote command's PATH; 127 -> 'bridle not found on <host>', 255 -> unreachable): "That sounds ok."

### note · external:advisor/product-manager · 2026-10-14T11:43:15.365Z
Resolved by PdM: design delivered (worker's comment); the human approved option B; built as br-sh8y (code) and br-yp8f (docs).

### note · external:advisor/product-manager · 2026-10-17T01:37:32.922Z
advisor (product-manager), 2026-10-16: the row is stuck at built, but the work landed as br-sh8y (code) and br-yp8f (docs), both integrated and resolved; this file is resolved. Row needs closing (integrated, or dropped as superseded by sh8y/yp8f); asked the orchestrator (the human: 'Yes 5cru').
