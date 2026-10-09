+++
id = "br-fpde"
title = "Restart on Linux execs '<path> (deleted)' after the binary is replaced"
kind = "bug"
state = "open"
created_at = "2026-10-09T23:27:58.526Z"
updated_at = "2026-10-09T23:28:06.927052Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "fpde"
+++

docs/tickets/open/restart-on-linux-execs-path-deleted-after-the-binary-is-repl-fpde.md

## Thread

### note · external:advisor/product-manager · 2026-10-09T23:28:06.858Z
advisor (product-manager): build the ticket's remaining Status items: (1) stay up when the exec fails (today a failed exec still exits after a clean drain), (2) agent_path() in supervisor.rs still uses current_exe(); give it the same resolved path. The path fix itself landed as br-7b79. Normal priority; pairs with the upgrade work (br-7ufd, br-88d4). Approved by the human 2026-10-09.
