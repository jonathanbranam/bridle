# Mail (inbound)

`bridle mail run` is the inbound half of email (ticket rs7p, which holds the design and the human's
answers). It lives in the `bridle-mail` crate so the AWS SDK and the parsing of untrusted MIME stay
out of the daemon. It holds an `external:mail` token (mint one, put it in `credentials.toml`, run
with `BRIDLE_AS=mail` or `--token`) and only calls `POST /v1/messages`.

Out of scope so far: outbound mail, the digest, reply tokens, project ownership gating, replies to
senders, advisor liveness.

## Config

`[mail]` in `~/.bridle/config.toml` (the daemon accepts and ignores it). AWS credentials come from
the standard AWS chain, never from bridle's files.

| Key | Default | Meaning |
|---|---|---|
| `bucket` | required | the S3 bucket SES writes to |
| `prefix` | `inbound/` | the receipt rule's key prefix |
| `region` | AWS default | |
| `domain` | `dev.branam.us` | only recipients on this domain count |
| `allow` | required | addresses and `@domains`; a domain entry still needs DMARC pass for that domain |
| `advisor_running` | `false` | route to `external:advisor` instead of `external:orchestrator` (a switch until the advisor pid file exists) |
| `max_body_chars` | 20000 | longer bodies are cut and marked |
| `max_attachment_bytes` | 1048576 | per attachment |
| `poll_secs` | 30 | |

## Behaviour

The bridge serves one project: `--project`, or the project of the workspace it runs in. Each poll
lists the prefix and, per object:

1. Recipient `<project>@domain` or `<project>+t-<task>@domain` (To or Cc). Anything else is another
   bridge's: left untouched.
2. Accepted only if all hold: the topmost `Authentication-Results` stamped `amazonses.com` says
   `dmarc=pass` for the From domain; the From address is on `allow`; `X-SES-Spam-Verdict` and
   `X-SES-Virus-Verdict` are `PASS`; not an auto-reply or bounce (`Auto-Submitted` other than `no`,
   `Precedence` bulk/junk/list/auto_reply, null `Return-Path`, `X-Autoreply`, mailer-daemon/postmaster).
   Otherwise dropped silently and logged; the object stays for the bucket's lifecycle rule and isn't
   fetched again until the bridge restarts.
3. The note is `via email from <from> (SES message <id>)`, the subject, and the new text (quoted
   history and signature stripped, HTML converted to text, capped). `.md`, `.txt` and `text/*`
   attachments up to the cap are written to `<workspace>/.bridle/inbox/mail/<id>/<name>` and listed
   by path; others are dropped and listed as dropped.
4. Sent to `external:orchestrator` or `external:advisor`; with a task address, as a note on that
   task's thread (`task`). The object is deleted only after the daemon accepted it, so a down
   daemon just means a retry on the next poll.

Tests use `FakeStore` and a recording sink: nothing touches AWS or a daemon.
