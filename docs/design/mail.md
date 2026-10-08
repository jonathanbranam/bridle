# Mail

> **Status (checked 2026-10-03):** Built and in use: `bridle mail run` (`crates/bridle-mail`): inbound from S3, outbound questions and the daily digest through SES, signed replies, following the owner, "got it" replies; it runs only on a machine whose `~/.bridle/config.toml` has `[mail]`, started by hand or as a service with `bridle mail install`

`bridle mail run` is bridle's email bridge (ticket rs7p, which holds the design and the human's
answers): inbound mail from S3, then outbound question mails and a daily digest through SES. It lives in the `bridle-mail` crate so the AWS SDK and the parsing of untrusted MIME stay
out of the daemon. It holds an `external:mail` token (mint one, put it in `credentials.toml`, run
with `BRIDLE_AS=mail` or `--token`) and only calls `POST /v1/messages`.

Mail follows the project's owner (see "Following the owner" below); "got it" replies go back to the sender.

## Running it as a service

`bridle mail install [--force]` (project from `--project` or the working directory) writes a
LaunchAgent (`~/Library/LaunchAgents/dev.bridle.mail.<project>.plist`, macOS) or a systemd user
unit (`~/.config/systemd/user/bridle-mail-<project>.service`, Linux) that runs
`bridle --project <project> mail run` with `BRIDLE_AS=mail` and `HOME` set (so the AWS chain finds
`~/.aws/credentials`), at login/boot, restarting on a crash only, logging stdout and stderr to
`~/.bridle/mail-<project>.log`. One service per project. It prints the load commands (and, on
Linux, `sudo loginctl enable-linger <user>`) and never runs them; it refuses to overwrite without
`--force`, and refuses when `[mail]` is missing or invalid or the `mail` token for the project is
not in `credentials.toml`. `bridle mail uninstall` removes the file and prints the unload command.

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
| `projects` | none | every project name mail may go to, on any machine; the bridge of the first one answers mail to any other name |
| `not_delivered_after_secs` | 3600 | how long mail may wait for its project's owner before the sender is told |
| `max_body_chars` | 20000 | longer bodies are cut and marked |
| `max_attachment_bytes` | 1048576 | per attachment |
| `poll_secs` | 30 | |
| `to` | `dev@branam.us` | where question mails and the digest go (SES sandbox: a verified address) |
| `digest_at` | `06:30` | the digest's time, `HH:MM` on the bridge machine's clock (US Eastern on the human's machines) |

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
4. Sent to `external:advisor` while the unnamed advisor is running (below), else `external:orchestrator`; with a task address, as a note on that
   task's thread (`task`). The object is deleted only after the daemon accepted it, so a down
   daemon just means a retry on the next poll.

Tests use `FakeStore` and a recording sink: nothing touches AWS or a daemon.

## Outbound

Each pass, after the inbound poll, the bridge reads the human's inbox through the daemon
(`GET /v1/messages?to=human`) and sends through SES (`ses:SendEmail`, same AWS chain and `region`),
from `<project>@domain`, to `[mail] to`. Nothing is mailed except:

- **Questions**, once each, as they arrive (unread, unanswered `question` messages). Subject
  `[bridle/<project>] <agent>: <first line>`; the body is the question text (capped at
  `max_body_chars`) and a footer. The mailed ids are kept in `<state dir>/mail/questions`, so a
  restart doesn't repeat them; a failed send is retried next pass.
- **One digest a day**, the first pass at or after `digest_at` on a day with no digest yet
  (`<state dir>/mail/digest` holds the date; a bridge started late sends it at once). It lists the
  open questions, the task questions, the other unread inbox items (blockers and decisions), task
  counts by state and the claimed tasks. Notes and answers are never mailed on their own.

Mail isn't confidential and crosses several systems, so outbound carries the question text, ids,
titles and counts only: no tokens, and no diffs or transcripts.

### Replies

`Reply-To` is `<project>+r-<msgid>.<tag>@domain`. The tag is the first 96 bits of
HMAC-SHA256(machine key, `<project>\n<msgid>`); the key is 32 random bytes in
`~/.bridle/mail.key` (mode 0600, created on first run). The tag binds a reply to that one
question; it isn't secret, since the address is in the mail. Authentication is still the allowlist
plus DMARC, as for any inbound mail.

A reply that passes those checks is sent to the agent that asked, as an `answer` with
`reply_to: <msgid>`, with the usual `via email from <from> (SES message <id>)` note as the body.
It is rejected and logged (silently to the sender, and not fetched again) when the tag doesn't
verify, or when the question is no longer open (already answered or read): a replay. The question
is marked read only when the answer arrives. For the answer to close the question as the human's,
`external:mail` must be in `[messages] answer_for_human`; the thread then shows the
answer from `external:mail` with the SES message id, i.e. the audit trail of the channel.

## Following the owner

Every machine may run a bridge for the projects it has; all read the same bucket.

- **Ownership.** A bridge takes its project's mail only while this machine owns the project: the
  host in `<workspace>/.bridle/state/owner.toml` (hw6c) is this machine's `hostname`, or there is
  no record. Otherwise the mail is left in the bucket for the owner.
- **Waiting mail.** Mail this bridge can't deliver (not the owner, or the daemon won't take it)
  is timed from when the bridge first saw it (in memory, so a restart restarts the clock). After
  `not_delivered_after_secs` the allowlisted sender gets one reply, `not delivered yet: no machine
  is running <project>`, and the mail stays for the owner. The notified keys are in
  `<state dir>/mail/notified`.
- **Unknown names.** Mail from an allowlisted, DMARC-passing sender to a name that is no project
  gets a reply listing the valid ones (`projects` plus the bridge's own), and the object is
  deleted. Only the bridge of the first name in `projects` answers, so several bridges don't
  each reply; strangers get nothing.
- **Advisor liveness.** `bridle session advisor` (unnamed advisor only) writes
  `$BRIDLE_HOME/advisor-<project>.pid` (`<pid> <ps lstart> <epoch>`, as `orchestrator.pid`) and
  removes it when claude ends. The bridge treats the advisor as running only while that pid exists
  with that start time. Running: mail goes to `external:advisor`, whose session runs
  `bridle wait-for-wake --mail`; otherwise to `external:orchestrator`.
- **"Got it".** Each delivered message id is kept with its sender in `<state dir>/mail/senders`. A
  message to `external:mail` with `reply_to` set to that id (the recipient's `bridle send
  external:mail "got it: ..." --reply-to <id>`) is emailed to the sender as `Re: <subject>` from
  `<project>@domain` and marked read.
