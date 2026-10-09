//! Judging and reading one raw message. Everything here is pure: the caller has the bytes, the
//! config and the project; nothing touches the network or the disk.

use mail_parser::{Address, MessageParser, MimeHeaders};

use crate::config::MailConfig;

/// Where a mail is addressed: `<project>@`, `<project>+t-<task>@`, or a reply address
/// `<project>+r-<msgid>.<tag>@`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub project: String,
    pub task: Option<String>,
    /// `(question message id, token)`; the token is checked by the bridge, which has the key.
    pub reply: Option<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    pub name: String,
    pub data: Vec<u8>,
}

/// A mail that passed every check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accepted {
    pub route: Route,
    pub from: String,
    pub subject: String,
    /// New text only: quoted history and signature stripped, capped.
    pub text: String,
    pub attachments: Vec<Attachment>,
    /// Attachments not kept (wrong type or over the cap), as `name (reason)`.
    pub dropped: Vec<String>,
}

/// Why a mail was not accepted. Every one but `NotOurs` means drop silently and log.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Rejection {
    /// Addressed to another project (or none on our domain): another bridge's, so untouched.
    #[error("not for this project")]
    NotOurs,
    /// An allowlisted, authenticated sender wrote to a name that is no project: the bridge
    /// answers with the valid names.
    #[error("{from} wrote to unknown project {name}")]
    UnknownProject { name: String, from: String },
    #[error("unparseable message")]
    Malformed,
    #[error("no usable From address")]
    NoFrom,
    #[error("DMARC did not pass for {0}")]
    Dmarc(String),
    #[error("{0} is not on the allowlist")]
    NotAllowed(String),
    #[error("spam or virus verdict is not PASS")]
    Verdict,
    #[error("auto-reply or bounce")]
    AutoReply,
    /// A reply address whose token doesn't match the question, or that can't be checked.
    #[error("invalid reply token")]
    BadToken,
    /// A valid token for a question that is already answered, or isn't an open question.
    #[error("replayed reply: {0} is not an open question")]
    Replayed(String),
}

/// Parses `local@domain` recipients on our domain into a route for `project`.
pub fn route_for(address: &str, domain: &str, project: &str) -> Option<Route> {
    let (local, d) = address.rsplit_once('@')?;
    if !d.eq_ignore_ascii_case(domain) {
        return None;
    }
    let local = local.to_ascii_lowercase();
    let (name, tag) = match local.split_once('+') {
        Some((n, t)) => (n, Some(t)),
        None => (local.as_str(), None),
    };
    if name != project.to_ascii_lowercase() {
        return None;
    }
    let mut reply = None;
    let task = match tag {
        None => None,
        Some(t) if t.starts_with("r-") => {
            let (id, mac) = t[2..].split_once('.')?;
            let plain =
                |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
            if !plain(id) || !plain(mac) {
                return None;
            }
            reply = Some((id.to_string(), mac.to_string()));
            None
        }
        Some(t) => {
            let id = t.strip_prefix("t-")?;
            let ok = !id.is_empty()
                && id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
            if !ok {
                return None;
            }
            Some(id.to_string())
        }
    };
    Some(Route {
        project: name.to_string(),
        task,
        reply,
    })
}

/// The first recipient on our domain whose name is no project.
fn unknown_name(recipients: &[String], cfg: &MailConfig, project: &str) -> Option<String> {
    let valid = cfg.valid_projects(project);
    recipients.iter().find_map(|a| {
        let (local, d) = a.rsplit_once('@')?;
        if !d.eq_ignore_ascii_case(&cfg.domain) {
            return None;
        }
        let name = local.split('+').next().unwrap_or(local);
        (!name.is_empty() && !valid.iter().any(|p| p.eq_ignore_ascii_case(name)))
            .then(|| name.to_string())
    })
}

fn addresses(a: Option<&Address<'_>>) -> Vec<String> {
    a.map(|a| {
        a.iter()
            .filter_map(|x| x.address())
            .map(|s| s.trim().to_ascii_lowercase())
            .collect()
    })
    .unwrap_or_default()
}

pub fn evaluate(raw: &[u8], cfg: &MailConfig, project: &str) -> Result<Accepted, Rejection> {
    let msg = MessageParser::default()
        .parse(raw)
        .ok_or(Rejection::Malformed)?;

    let recipients: Vec<String> = addresses(msg.to())
        .into_iter()
        .chain(addresses(msg.cc()))
        .collect();
    let route = recipients
        .iter()
        .find_map(|a| route_for(a, &cfg.domain, project));
    let unknown = match route {
        Some(_) => None,
        None if cfg.answers_unknown(project) => unknown_name(&recipients, cfg, project),
        None => None,
    };
    if route.is_none() && unknown.is_none() {
        return Err(Rejection::NotOurs);
    }

    let from = addresses(msg.from())
        .into_iter()
        .next()
        .ok_or(Rejection::NoFrom)?;
    let from_domain = from
        .rsplit_once('@')
        .map(|(_, d)| d)
        .ok_or(Rejection::NoFrom)?;

    // The topmost SES-stamped header is ours; anything a sender put lower is theirs.
    let header = |name: &str| {
        msg.headers_raw()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.trim().to_string())
    };
    let ses_auth = msg
        .headers_raw()
        .filter(|(n, _)| n.eq_ignore_ascii_case("Authentication-Results"))
        .map(|(_, v)| v.trim())
        .find(|v| v.to_ascii_lowercase().starts_with("amazonses.com"));
    if !ses_auth.is_some_and(|v| dmarc_passes(v, from_domain)) {
        return Err(Rejection::Dmarc(from_domain.to_string()));
    }
    if !cfg.allows(&from) {
        return Err(Rejection::NotAllowed(from));
    }
    let pass = |name: &str| header(name).is_some_and(|v| v.eq_ignore_ascii_case("PASS"));
    if !(pass("X-SES-Spam-Verdict") && pass("X-SES-Virus-Verdict")) {
        return Err(Rejection::Verdict);
    }
    if is_auto_reply(&header, &from) {
        return Err(Rejection::AutoReply);
    }
    let Some(route) = route else {
        return Err(Rejection::UnknownProject {
            name: unknown.unwrap_or_default(),
            from,
        });
    };

    let body = msg.body_text(0).map(|c| c.into_owned()).unwrap_or_default();
    let text = cap(&strip_quoted(&body), cfg.max_body_chars);

    let mut attachments = Vec::new();
    let mut dropped = Vec::new();
    for part in msg.attachments() {
        let name = safe_name(part.attachment_name().unwrap_or("attachment"));
        let ctype = part.content_type();
        let is_text = ctype.is_some_and(|c| c.ctype().eq_ignore_ascii_case("text"));
        let lower = name.to_ascii_lowercase();
        if !(is_text || lower.ends_with(".md") || lower.ends_with(".txt")) {
            dropped.push(format!("{name} (only .md, .txt and text/* are kept)"));
        } else if raw_contents(&msg, part).len() > cfg.max_attachment_bytes {
            dropped.push(format!("{name} (over {} bytes)", cfg.max_attachment_bytes));
        } else {
            attachments.push(Attachment {
                name,
                data: raw_contents(&msg, part),
            });
        }
    }

    Ok(Accepted {
        route,
        from,
        subject: msg.subject().unwrap_or("").trim().to_string(),
        text,
        attachments,
        dropped,
    })
}

/// A part's bytes as the sender wrote them (transfer encoding undone, no charset decoding).
/// The parser decodes text parts by their declared charset, so a UTF-8 file labelled
/// iso-8859-1 (or unlabelled and not UTF-8 clean) comes out as mojibake. Re-parsing the body
/// slice as an unlabelled binary part gets the transfer-decoded bytes untouched.
fn raw_contents(msg: &mail_parser::Message<'_>, part: &mail_parser::MessagePart<'_>) -> Vec<u8> {
    let is_text = matches!(
        part.body,
        mail_parser::PartType::Text(_) | mail_parser::PartType::Html(_)
    );
    if !is_text {
        return part.contents().to_vec();
    }
    let raw = msg.raw_message();
    let (start, end) = (
        part.raw_body_offset() as usize,
        part.raw_end_offset() as usize,
    );
    let Some(body) = raw.get(start..end) else {
        return part.contents().to_vec();
    };
    let cte = part
        .headers()
        .iter()
        .find(|h| matches!(h.name, mail_parser::HeaderName::ContentTransferEncoding))
        .and_then(|h| h.value.as_text())
        .unwrap_or("7bit");
    let mut synthetic = format!(
        "Content-Type: application/octet-stream\r\nContent-Transfer-Encoding: {cte}\r\n\r\n"
    )
    .into_bytes();
    synthetic.extend_from_slice(body);
    mail_parser::MessageParser::default()
        .parse(&synthetic)
        .and_then(|m| m.parts.first().map(|p| p.contents().to_vec()))
        .unwrap_or_else(|| part.contents().to_vec())
}

/// `dmarc=pass` in an Authentication-Results value, for the From domain when it says which.
fn dmarc_passes(value: &str, from_domain: &str) -> bool {
    value.split(';').map(str::trim).any(|clause| {
        let lower = clause.to_ascii_lowercase();
        let Some(rest) = lower.strip_prefix("dmarc=") else {
            return false;
        };
        let mut words = rest.split_whitespace();
        words.next() == Some("pass")
            && words
                .find_map(|w| w.strip_prefix("header.from="))
                .is_none_or(|d| d.trim_matches(|c| c == ';' || c == '"') == from_domain)
    })
}

fn is_auto_reply(header: &impl Fn(&str) -> Option<String>, from: &str) -> bool {
    let auto = header("Auto-Submitted").is_some_and(|v| !v.eq_ignore_ascii_case("no"));
    let precedence = header("Precedence").is_some_and(|v| {
        matches!(
            v.to_ascii_lowercase().as_str(),
            "bulk" | "junk" | "auto_reply" | "list"
        )
    });
    let null_sender = header("Return-Path").is_some_and(|v| v.replace(' ', "") == "<>");
    let local = from.split('@').next().unwrap_or("");
    auto || precedence
        || null_sender
        || header("X-Autoreply").is_some()
        || header("X-Autorespond").is_some()
        || matches!(local, "mailer-daemon" | "postmaster")
}

/// Keeps a file name from climbing out of the directory it is written to.
fn safe_name(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or("");
    let cleaned: String = base
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let cleaned = cleaned.trim_start_matches('.');
    if cleaned.is_empty() {
        "attachment".to_string()
    } else {
        cleaned.to_string()
    }
}

pub(crate) fn cap(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let cut: String = text.chars().take(max_chars).collect();
    format!("{cut}\n[truncated: the mail body was over {max_chars} characters]")
}

/// The new text of a reply: drops the quoted history (`>` lines, "On ... wrote:", Outlook's
/// "Original Message" and From/Sent header blocks) and the signature (`-- `).
pub fn strip_quoted(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut keep = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        let next = lines.get(i + 1).map(|l| l.trim()).unwrap_or("");
        let wrote = |s: &str| s.starts_with("On ") && s.ends_with("wrote:");
        let header_block = t.starts_with("From:")
            && lines[i + 1..]
                .iter()
                .take(3)
                .any(|l| l.starts_with("Sent:") || l.starts_with("Date:"));
        if *line == "-- "
            || t == "--"
            || t.starts_with("-----Original Message")
            || t.starts_with("________________")
            || wrote(t)
            || (t.starts_with("On ") && next.ends_with("wrote:"))
            || header_block
        {
            break;
        }
        if t.starts_with('>') {
            continue;
        }
        keep.push(*line);
    }
    keep.join("\n").trim().to_string()
}
