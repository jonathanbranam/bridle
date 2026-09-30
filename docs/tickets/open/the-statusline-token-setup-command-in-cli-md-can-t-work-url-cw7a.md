---
id: cw7a
title: "The statusline token setup command in cli.md can't work: --url drops the workspace"
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [r7cs]
---

## The ask


The human, 2026-09-30:

> two separate agents have given me the wrong command for creating the statusline token; this
> needs to be documented better somewhere - NOT in an agents context but maybe the CLI docs?
> This is a very rare operation so it's not a huge issue but FYI
>
> failed
>
> ```
> bridle main % (umask 077; bridle --url http://127.0.0.1:7401 token create statusline > ~/.bridle/statusline.token)
> error: no workspace found to read the human token from: set $BRIDLE_TOKEN
> ```
>
> worked:
>
> ```
>  % bridle --url http://127.0.0.1:7402 --token "$(cat /srv/shared/work/meta-notes-work/.bridle/tokens/human)" token create statusline > ~/.bridle/statusline.token; chmod 600 ~/.bridle/statusline.token
> ```

## What's there now

`docs/design/cli.md` (at b67cc80), the `statusline` entry, gives this one-time setup:

> One-time setup: `bridle --url <daemon url> token create statusline > ~/.bridle/statusline.token`
> (`--url` so the token is printed, not saved in `credentials.toml`)

Passing `--url` (or `$BRIDLE_URL`) resolves the endpoint with `workspace: None`
(`resolve_endpoint_with`, `crates/bridle-api/src/discovery.rs`), so the human token is never
read from the workspace. Unless `$BRIDLE_TOKEN` is set, the documented command always fails with
the error above, even when you run it inside a workspace. It needs `--token "$(cat <workspace>/.bridle/tokens/human)"`,
as in the command that worked. The error message doesn't mention `--token` either.
