+++
id = "br-yp8f"
title = "Docs: the Remote PATH step for remote bridle over ssh (add-a-machine, WSL2 guide, cli.md)"
kind = "docs"
state = "built"
created_at = "2026-10-14T11:42:51.116Z"
updated_at = "2026-10-14T21:28:59.102024Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:yp8f",
]
branch = "bridle/yp8f"
commit = "671cde1a"
summary = """Docs only. Added a "Remote PATH: bridle over ssh" subsection to docs/context/add-a-machine.md (end of section 2, before section 3): which commands run bridle over ssh (token pair, project stop|start --machine, project preflight --to), the macOS /etc/zshenv and Ubuntu/WSL2 /etc/zsh/zshenv lines, and the `ssh <host> 'command -v bridle'` check. docs/context/windows-wsl2-host.md (step 6): the `export PATH` line fixes only the interactive shell, with a pointer to the add-a-machine step. CHANGELOG Unreleased: one Docs line. Decision: br-sh8y is already on main (commits 8ddad90b and b9d7cdf7 ancestry), so the ticket's "until br-sh8y lands" is stale; the step says since br-sh8y those commands also look in ~/.local/bin and ~/.cargo/bin, so the zshenv line matters only when bridle is installed elsewhere. Caveat: `bridle ticket check` reports one pre-existing problem in an unrelated resolved ticket (eurm: references br-n7cg, not a ticket); not touched here."""
ticket = "yp8f"
+++

docs/tickets/open/docs-the-remote-path-step-for-remote-bridle-over-ssh-add-a-m-yp8f.md

## Thread

### note · agent:yp8f · 2026-10-14T21:28:51.032Z
done: Remote PATH step added to add-a-machine.md and WSL2 guide, CHANGELOG Docs line; just check exit 0, 1683 run, 1683 passed, 6 skipped (baseline 1683); commit 671cde1a. Note: bridle ticket check exits 1 on a pre-existing eurm problem (references br-n7cg), not from this change.

### note · agent:manager-2 · 2026-10-14T21:28:59.102Z
built: 671cde1a (branch bridle/yp8f)
