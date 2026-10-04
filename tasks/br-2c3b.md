+++
id = "br-2c3b"
title = "Landing survives merge.ff=only; land/tools_only tests ignore the host git config"
kind = "bug"
state = "integrated"
created_at = "2026-10-04T00:31:48.391Z"
updated_at = "2026-10-04T00:56:20.661720Z"
created_by = "agent:manager-2"
watchers = ["agent:manager-2"]
size = "S"
branch = "bridle/git-config-tests"
commit = "28842110dd8e54bb9a25379b97ddd57d2752f12a"
summary = "Both failures predate br-8eyu (identical on 7de2bc9 and 07f7a54). The daemon's squash merge in integrator.rs now passes -c merge.ff=true so a user's merge.ff=only cannot fail a land when main has moved (a real bug for the human's landings). The land_test and tools_only_test git helpers set GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1 so the host's config and ~/.git_template hooks cannot leak into test repos. just check passes without any override."
+++

Found while landing br-8eyu: the human's global merge.ff=only breaks the daemon's 'git merge --squash' in integrator.rs when main has moved (fixed with -c merge.ff=true), and the ~/.git_template hooks plus global config break land_test/tools_only_test (tests now set GIT_CONFIG_GLOBAL=/dev/null, GIT_CONFIG_NOSYSTEM=1). Both predate br-8eyu. Branch bridle/git-config-tests.

## Thread

### note · agent:manager-2 · 2026-10-04T00:55:18.715Z
integrated: 28842110dd8e54bb9a25379b97ddd57d2752f12a (branch bridle/git-config-tests)

### note · agent:manager-2 · 2026-10-04T00:56:20.661Z
cleanup: removed agent git-config-tests, branch bridle/git-config-tests
