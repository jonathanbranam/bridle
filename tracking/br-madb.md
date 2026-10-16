+++
id = "br-madb"
title = "bridle project setup stop points 2 and 3: clone from URL, root prompt, main/adopt question, aide token, next-step hints"
kind = "feature"
state = "integrated"
created_at = "2026-10-16T05:28:35.570Z"
updated_at = "2026-10-16T06:39:44.343287Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:madb",
]
branch = "bridle/madb"
commit = "de7c745470c2bd3ea04017d2cffbe3e3e520aa82"
summary = 'Stop points 2 and 3 of bridle project setup, in crates/bridle/src/project_setup.rs. The steps are now clone, init, register, state, service, tokens, mail, aide token. The positional argument is a name or a URL (anything URL-like is cloned), plus --remote, --root, --main/--adopt and -y/--yes. Clone checks that the remote is reachable first (git ls-remote, never prompts) and reads whether it has commits. Init calls init::scaffold and the vendor install, then commits; for a project with commits the files go on a new bridle-adopt branch with integration set to it (default, rule existing-projects), --main commits on the current branch. Nothing is pushed. Questions (root with the <base>/<p>-work/<p> default, main/adopt, aide [Y/n]) are asked after the checklist and before any write; -y takes and prints the defaults; no terminal and no -y lists the missing answers and exits 2 (new CliError::Usage). Tokens now pairs every role but aide; the aide token is its own step (token pair --roles aide). The checklist ends with "next: <command>" (first step not done) and bridle init prints the same line via print_next. Caveats: a remote with commits is asked main/adopt even if it already holds .bridle/config.toml (only a clone can tell); declining the aide token is not remembered (asked again next run); a failed init commit leaves the files uncommitted and the rerun treats init as done. Verified the apply path in a throwaway HOME/BRIDLE_HOME with local bare remotes and a PATH shim for launchctl: clone, init (bridle-adopt branch + commit), register ran for real, the service step wrote its unit only under the throwaway HOME and stopped with the exact fix because the shimmed launchctl starts no daemon; a rerun resumed and reported ok for the first three. Not run live: tokens, mail and aide apply (they need a running daemon), covered by the existing pair/mail code. Docs: cli.md, adding-a-project.md, CHANGELOG updated.'
ticket = "madb"
+++

docs/tickets/open/bridle-project-setup-stop-points-2-and-3-clone-from-url-root-madb.md

## Thread

### note · external:advisor/product-manager · 2026-10-16T05:40:26.588Z
Readied by advisor product-manager under the human's go on dffz (2026-10-15 ~10:15 PM ET): "Yes dffz. Fix as much as you can tonight".

### note · agent:madb · 2026-10-16T06:26:24.755Z
built: 7867249dabf8edee445bed75616cf858b38e9806 (branch bridle/madb)

### note · agent:madb · 2026-10-16T06:26:24.814Z
done: project setup stop points 2+3 (clone from URL/--remote, --root prompt, main/adopt, aide token, -y, exit 2, next: line); just check exit 0, 1754 tests passed; apply verified in a throwaway HOME with shimmed launchctl; commit 7867249d

### note · agent:manager-2 · 2026-10-16T06:39:25.660Z
integrated: de7c745470c2bd3ea04017d2cffbe3e3e520aa82 (branch bridle/madb)

### note · agent:manager-2 · 2026-10-16T06:39:44.343Z
cleanup: removed agent madb, branch bridle/madb
