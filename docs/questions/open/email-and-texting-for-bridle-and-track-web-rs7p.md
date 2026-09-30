---
id: rs7p
title: Email (and later texting) for bridle and track-web: send and receive
opened: 2026-09-29
repos: [bridle, track-web]
changes: []
specs: []
needs: []
see: [t6kq, hw6c]
---

The human, verbatim (2026-09-29): "a follow up general ability to consider is email and texting for both the life tracking app and for bridle. Actual email would be handy sometimes. I could send you work directly from my work laptop then. So, send and receive. That could go through my EC2 that runs track-web. Come up with some ideas and research. Email first"

Research by the orchestrator (a subagent), 2026-09-29; prices and limits as of then. Not scheduled: the human answers the open questions (section 5) first.

## Recommendation

Use **Amazon SES in us-east-1** (the EC2's region) for both directions, on a new
subdomain such as `bridle.branam.us`. Inbound mail lands in an **S3 bucket**. A small
**`bridle mail` bridge** process on each bridle machine polls that bucket, checks the
sender, and calls the daemon's HTTP API as an `external:mail` principal. The same
bridge follows the human inbox and sends questions and a daily digest through the SES
API. The EC2 doesn't need to be in bridle's mail path at all: S3 and SES are reachable
from the Mac and the NUC with outbound HTTPS only. track-web uses the same SES setup
from the EC2, with its own address and S3 prefix.

It costs cents a month and adds no new vendor or internet-facing server. Mail never
leaves the human's own AWS account, and SES reports SPF, DKIM and DMARC verdicts on
every message. Staying in the **SES sandbox** is a feature here: bridle can then only
send to addresses the human has verified.

Runner-up: **Resend**. Its free tier (3,000 emails a month, 100 a day) includes
inbound mail and an API to list received mail, so it can be polled with no public
endpoint and no IAM setup. The costs are a third party holding the mail, and it's
unverified whether Resend exposes authentication results.

## What exists today (checked, not guessed)

- The EC2 is `54.221.218.118`, which AWS's published ranges put in **us-east-1**. It's a
  t4g.micro running Caddy and a Node backend under pm2 (track-web `README.md`, `Caddyfile`).
- DNS for `branam.us` is on Google Cloud DNS (`ns-cloud-b*.googledomains.com`). There's
  a wildcard `*.branam.us` A record.
- **The apex MX is Google Workspace** (`aspmx.l.google.com`), with SPF
  `include:_spf.google.com ~all` and no DMARC record. Mail for the bare domain must not
  be touched, so the MX goes on a subdomain.
- track-web has no mail or SMS code today.

## 1. Inbound options

| Option | Path to a private daemon | Setup | Cost | Notes |
|---|---|---|---|---|
| **SES receiving → S3** | Bridge polls S3 (or an SQS queue fed by SNS) | MX + domain verification, bucket policy, receipt rule, scoped IAM key: about an hour | $0.10 per 1,000 mails + $0.09 per 1,000 256 KB chunks | Receiving works in us-east-1. Adds SPF/DKIM/DMARC results and spam/virus verdicts as headers. Up to 40 MB via S3 (150 KB if sent through SNS) |
| Resend inbound | Bridge polls the Receiving API | MX + DKIM on the subdomain, one API key | Free tier | One vendor for both directions |
| Mailgun routes | Webhook, so needs a public endpoint (the EC2) that the daemon polls | Medium | Free: 1 inbound route, 100 sends a day | Needs a relay built on the EC2 |
| Postmark inbound | Webhook, same as Mailgun | Medium | Pro plan only, $16.50/month | Good parsing, but paid |
| SendGrid Inbound Parse | Webhook | Medium | Free plan retired in 2025; paid from about $19.95/month | Not worth it |
| Cloudflare Email Routing + Workers | Worker pushes to the EC2, or stores mail in a queue or R2 | High: **DNS must move to Cloudflare** | Free | Too big a change for this |
| Postfix on the EC2 | Daemon polls the EC2 over HTTPS | High: an internet-facing MTA, spam filtering, TLS | $0 | Also needs SES as an outbound relay, because EC2 blocks outbound port 25 by default |
| IMAP mailbox (dedicated Gmail) | Bridge polls IMAP | About 15 minutes, no DNS | Free | Needs an app password (Google disabled basic auth). It's a bot on a consumer account, and the address is `@gmail.com` |

IMAP is the only simpler option: a fair fallback. The webhook services need a public
endpoint, i.e. a store-and-forward relay built into track-web: more parts than polling S3.

## 2. Outbound

- **SES API** (`SendEmail`), using Easy DKIM on the subdomain (three CNAMEs). The sandbox
  allows 200 mails a day, at 1 a second, **only to verified addresses**, which is enough
  for one human. Request production access only if track-web ever mails other family
  members. Add a DMARC record (`p=none` to start).
- Resend or Postmark: fine, but a second vendor. SMTP through the IMAP mailbox: fine if
  IMAP was chosen for inbound.

## 3. Proposed v1 architecture

**DNS:** `bridle.branam.us MX 10 inbound-smtp.us-east-1.amazonaws.com`, plus the SES
DKIM CNAMEs. **AWS:** a receipt rule for `bridle.branam.us` that writes to
`s3://…/inbound/` with scanning on, a 30-day lifecycle and SSE. An IAM user per machine
with only `s3:List/Get/Delete` on that prefix and `ses:SendEmail` from the subdomain.

**The bridge is outside the daemon:** a `bridle mail run` subcommand, or a
`bridle-mail` crate, running beside `serve`. It holds an `external:mail` token from
`credentials.toml`. Keeping it separate keeps the AWS SDK and parsing of untrusted
MIME out of the daemon, and fits the existing external-principal model. The bridge only
calls the send and inbox endpoints.

**Address → bridle concept:**
- `<project>@bridle.branam.us` → a `note` to that project's manager (or orchestrator).
  The body is the subject plus the new text, with quoted history stripped. Attachments
  are dropped in v1.
- `<project>+t-<task>@…` → a note on the task thread (the message `task` field).
- A reply to a question email → an answer with `reply_to: m-NNNN`. Outbound question
  mails set `Reply-To: <project>+r-<msgid>.<hmac>@bridle.branam.us`. The HMAC, keyed
  per machine, binds the reply to that one question, so a guessed or replayed address
  can't answer a different one.
- Several daemons (Mac, NUC): each bridge handles only the projects in its own registry
  and leaves other objects alone.

**Outbound:** the bridge follows the human inbox events.
- `question` messages are mailed right away. Subject: `[bridle/<project>] <agent>:
  <first line>`.
- Everything else waits for one **daily digest**, which lists open questions, blockers
  and task state. That keeps to the inbox rule of questions, blockers and decisions only.
- A question is marked read when its answer arrives, not when it's mailed.

**Security:**
- Accept a message only if all of these hold, and drop it silently otherwise (never
  auto-reply to a stranger):
  - SES reports DMARC `pass` for the `From:` domain.
  - The `From:` address is on an allowlist (the work address and the personal address).
  - The spam and virus verdicts pass.
  - It isn't an auto-reply or a bounce (the `Auto-Submitted` header, or a null sender).
- Log every rejection.
- Without a DMARC pass, `From:` can be spoofed trivially. The allowlist and DMARC
  together are the authentication. The reply token adds a per-question binding, not
  secrecy.
- **Prompt injection:** email becomes instructions, so strip quoted text and
  signatures, cap the size, and tag each message `via email` so agents and the record
  show its provenance. Consider a confirm step (reply `yes` to a short "queued: …"
  receipt) before anything that spawns work. That's optional in v1.
- **Email isn't confidential:** it crosses Google, the employer's mail system and AWS
  in clear text. Outbound mail carries summaries and links, never secrets, tokens or
  diffs. The employer can read everything sent to the work address.

**track-web:** the same domain setup. Give it its own subdomain (`track.branam.us`), whose receipt rule writes to
`inbound-track/` and publishes to SNS. The EC2 is public, so SNS can push over HTTPS
straight to `/api/mail/inbound`, after verifying the SNS signature. It sends with the
SES API using an instance role. No Postfix and no relay.

## The human: mail reaches the machine running the project (2026-09-30)

The human, verbatim: "I would want my email to find the right box that is working on the right
project."

So routing is by project, and the machine is whichever one serves that project now: the human
addresses a project (`meta-notes@…`), never a machine. How the v1 shape above meets it, and
the gaps (the advisor's reading):

- Every bridge polls the same bucket and takes only mail for the projects in its own registry
  (section 3, "Several daemons"). So mail follows the project to whichever box runs it.
- **Ownership must be unambiguous.** Two bridges must never both claim a project, and when a
  project moves the old bridge must let go. That's the owner record on `bridle/state` in hw6c:
  a bridge takes a project's mail only while its daemon owns the project.
- **Mail for a project no box is serving** (daemon down, machine lost, mid-move) waits in the
  bucket for the owner, and after some time (an hour?) the human gets a "not delivered yet: no
  machine is running `<project>`" reply, rather than silence.
- **An unknown project name** gets a reply listing the valid ones (allowlisted senders only).

## 4. Texting

- **Real SMS is heavy for one person.** US A2P 10DLC as a sole proprietor on Twilio
  costs a $4 brand fee, a $15 campaign vetting fee and $2 a month per campaign, plus
  the number and per-message fees. Approval takes days to weeks, with one number at 1
  message a second. AWS End User Messaging is similar: toll-free numbers need their own
  registration, and 10DLC campaigns are about $10 a month.
- **iMessage via the Mac:** fragile scripting of Messages.app; no good from the NUC.
- **Outbound only:** ntfy (free tier 250 messages a day, or self-hosted on the EC2) or
  Pushover ($4.99 once per platform). One HTTP POST each.
- **Recommend a Telegram bot.** It's free, two-way, needs no registration, and
  `getUpdates` long-polling works from a private machine. That's the same polling shape
  as the mail bridge. Allowlist the human's chat id. Bot chats aren't end-to-end
  encrypted, so the same "no secrets" rule applies. If only alerts are wanted, use ntfy.

## 5. Open questions for the human

1. Will the work laptop's mail system let you mail a personal domain, and does your
   employer publish DMARC (checked with `dig _dmarc.<employer>`)? Are there policy
   concerns about sending work text to a personal system?
2. Which subdomain(s)? `bridle.branam.us` and `track.branam.us`, or one `mail.branam.us`
   with plus addressing?
3. Where does a new emailed task go: the project's manager, the orchestrator, or the
   human inbox for triage?
4. Is a confirm step wanted before emailed work is acted on?
5. Digest time and time zone. Should blockers also mail immediately?
6. Should an email answer be recorded as `external:mail` or as `human`? Does `reply_to`
   need to accept a non-human answerer?
7. For texting: Telegram acceptable, or is outbound-only ntfy enough?

## Sources

- SES receiving concepts, actions and auth verdicts: https://docs.aws.amazon.com/ses/latest/dg/receiving-email-concepts.html
- SES receiving endpoints by region and sandbox quotas: https://docs.aws.amazon.com/general/latest/gr/ses.html
- SES pricing: https://aws.amazon.com/ses/pricing/
- Resend receiving and pricing: https://resend.com/docs/dashboard/receiving/introduction, https://resend.com/pricing
- Postmark pricing: https://postmarkapp.com/pricing
- Mailgun pricing: https://www.mailgun.com/pricing/
- SendGrid free plan retired: https://dreamlit.ai/blog/best-sendgrid-alternatives
- Cloudflare Email Routing (needs Cloudflare DNS; subdomains): https://developers.cloudflare.com/email-routing/limits/, https://developers.cloudflare.com/email-service/configuration/subdomains/
- Google app passwords and IMAP: https://www.getmailbird.com/gmail-oauth-changes-app-password-phase-out/
- Twilio 10DLC sole proprietor: https://support.twilio.com/hc/en-us/articles/4407882914971, https://www.sociocs.com/post/twilio-10dlc-explained/
- AWS toll-free registration: https://aws.amazon.com/blogs/messaging-and-targeting/how-to-register-for-a-us-toll-free-number-with-aws-end-user-messaging
- Pushover cost: https://support.pushover.net/i8-cost ; ntfy: https://docs.ntfy.sh/
