//! `@role` mentions in a comment (br-4cgx): who a comment names, and the messages they get.
//! Shared by ticket comments (`server.rs`) and document comments (`doc_watch.rs`).

use bridle_api::discovery;
use bridle_api::types::MessageKind;

use crate::outbox::Outbox;
use crate::server::{excerpt, resolve_name};
use crate::store::{OutboxRow, Store};
use crate::supervisor::{AgentManager, ToTarget};

/// The distinct names `text` mentions, in order of first appearance. An `@` counts only at the
/// start of the text or after a character that cannot end a name (so `a@b` and email addresses
/// are not mentions), and `\@name` is escaped plain text. A name is letters, digits, `-`, `_`,
/// `/` (named sessions: `advisor/product-manager`) and `:` (a project qualifier, see `split_project`); a trailing `/` or `:` (sentence punctuation) is dropped.
pub fn mentioned(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<String> = Vec::new();
    for (i, &c) in chars.iter().enumerate() {
        if c != '@' {
            continue;
        }
        if i > 0 {
            let prev = chars[i - 1];
            if prev == '\\' || prev.is_alphanumeric() || matches!(prev, '_' | '.' | '-' | '/') {
                continue;
            }
        }
        let name: String = chars[i + 1..]
            .iter()
            .take_while(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '/' | ':'))
            .collect();
        let name = name.trim_end_matches(['/', ':']);
        if !name.is_empty() && !out.iter().any(|n| n == name) {
            out.push(name.to_string());
        }
    }
    out
}

/// Where a mentioned name goes: this project's role or agent, or `role` on another project
/// (`ui:orchestrator`; the role may carry its explicit `external:` form).
pub fn split_project(name: &str) -> (Option<&str>, &str) {
    match name.split_once(':') {
        Some((project, role)) if !project.is_empty() && !role.is_empty() => (Some(project), role),
        _ => (None, name),
    }
}

/// The project a qualifier names: a project this machine can reach by name, or by its ticket ID
/// prefix (`ui` for `bridle-ui`). `None` when neither is known or it cannot be forwarded to.
fn project_of(outbox: &Outbox, qualifier: &str) -> Option<String> {
    let project = if outbox.resolve(qualifier).is_ok() {
        qualifier.to_string()
    } else {
        discovery::list_registry()
            .into_iter()
            .find(|d| d.task_prefix.as_deref() == Some(qualifier))?
            .project
    };
    (project == outbox.project() || outbox.check_destination(&project).is_ok()).then_some(project)
}

/// Percent-encode everything but unreserved characters; `keep_slash` leaves `/` for a path value.
pub fn encode(s: &str, keep_slash: bool) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) || (keep_slash && b == b'/') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// The UI URL for a ticket id or a document path, as `bridle link` prints it.
pub fn target_url(base: &str, project: &str, target: &str) -> String {
    let p = encode(project, false);
    if target.contains('/') || target.ends_with(".md") {
        format!("{base}/p/{p}/docs?path={}", encode(target, true))
    } else {
        format!("{base}/p/{p}/tickets/{}", encode(target, false))
    }
}

/// `target_url` under the configured `[gateway] public_url`; `None` when none is set.
pub fn link_for(repo: &std::path::Path, project: &str, target: &str) -> Option<String> {
    let home = bridle_api::discovery::bridle_home();
    let base = crate::config::ui_base_url(&home, Some(repo))
        .ok()
        .flatten()?;
    Some(target_url(&base, project, target))
}

/// Sends each name `body` mentions one `task_update` message: `{here}: {actor} mentioned you:
/// ...`. Names resolve in this project, or through the outbox for `project:role` (br-4cgx).
/// `already` (and the author, who is not told of their own mention) are skipped. Returns the
/// local targets told, so a watcher notice can skip them, and the names that resolve to nobody.
#[allow(clippy::too_many_arguments)]
pub async fn send(
    store: &Store,
    manager: &AgentManager,
    outbox: Option<&Outbox>,
    actor: &str,
    author: Option<&ToTarget>,
    here: &str,
    link: Option<&str>,
    body: &str,
    already: &[ToTarget],
) -> (Vec<ToTarget>, Vec<String>) {
    let link = link.map(|l| format!(" {l}")).unwrap_or_default();
    let line = format!("{here}{link}: {actor} mentioned you: {}", excerpt(body));
    let mut told: Vec<ToTarget> = Vec::new();
    let mut unknown: Vec<String> = Vec::new();
    // `ui:orchestrator` and `ui:external:orchestrator` are one recipient.
    let mut remote: Vec<(String, String)> = Vec::new();
    for name in mentioned(body) {
        let (qualifier, role) = split_project(&name);
        let project = match (qualifier, outbox) {
            (None, _) => None,
            (Some(q), Some(outbox)) => match project_of(outbox, q) {
                Some(p) if p == outbox.project() => None,
                Some(p) => Some((p, outbox)),
                None => {
                    unknown.push(name);
                    continue;
                }
            },
            (Some(_), None) => {
                unknown.push(name);
                continue;
            }
        };
        if let Some((project, outbox)) = project {
            let key = (
                project.clone(),
                role.strip_prefix("external:").unwrap_or(role).to_string(),
            );
            if remote.contains(&key) {
                continue;
            }
            remote.push(key);
            let from = if actor.contains('@') {
                actor.to_string()
            } else {
                format!("{actor}@{}", outbox.machine_name())
            };
            let queued = store
                .outbox_enqueue(OutboxRow {
                    id: String::new(),
                    project: project.clone(),
                    from,
                    to: role.to_string(),
                    kind: MessageKind::TaskUpdate,
                    body: line.clone(),
                    reply_to: None,
                    when: Default::default(),
                    attempts: 0,
                    last_error: None,
                })
                .await;
            match queued {
                Ok(_) => {
                    let outbox = outbox.clone();
                    tokio::spawn(async move { outbox.flush(&project).await });
                }
                Err(e) => tracing::warn!(%name, "mention not queued: {e}"),
            }
            continue;
        }
        let Some(targets) = resolve_name(store, role).await else {
            unknown.push(name);
            continue;
        };
        for target in targets {
            if already.contains(&target) || told.contains(&target) || author == Some(&target) {
                continue;
            }
            match manager
                .send(
                    actor.to_string(),
                    target.clone(),
                    MessageKind::TaskUpdate,
                    line.clone(),
                    Default::default(),
                    None,
                )
                .await
            {
                Ok(_) => told.push(target),
                Err(e) => tracing::warn!(%name, "mention not sent: {e}"),
            }
        }
    }
    (told, unknown)
}

/// Tells `author` which mentions went to nobody (nothing is sent for an empty `unknown`).
pub async fn report_unknown(
    manager: &AgentManager,
    actor: &str,
    author: ToTarget,
    here: &str,
    unknown: &[String],
) {
    if unknown.is_empty() {
        return;
    }
    let names = unknown
        .iter()
        .map(|n| format!("@{n}"))
        .collect::<Vec<_>>()
        .join(", ");
    let line = format!(
        "{here}: no such recipient for {names}; nobody was told (write \\@name for plain text)"
    );
    let _ = manager
        .send(
            actor.to_string(),
            author,
            MessageKind::TaskUpdate,
            line,
            Default::default(),
            None,
        )
        .await;
}

#[cfg(test)]
mod tests {
    use super::mentioned;

    #[test]
    fn target_urls_match_bridle_link() {
        assert_eq!(
            super::target_url("http://ui:1", "bridle", "br-4cgx"),
            "http://ui:1/p/bridle/tickets/br-4cgx"
        );
        assert_eq!(
            super::target_url("http://ui:1", "bridle", "docs/a b.md"),
            "http://ui:1/p/bridle/docs?path=docs/a%20b.md"
        );
    }

    #[test]
    fn finds_each_distinct_name_once() {
        assert_eq!(
            mentioned("@orchestrator I approve. cc @advisor/product-manager, @orchestrator."),
            ["orchestrator", "advisor/product-manager"]
        );
    }

    #[test]
    fn a_backslash_escapes_the_mention() {
        assert!(mentioned(r"\@orchestrator is plain text").is_empty());
    }

    #[test]
    fn an_email_address_is_not_a_mention() {
        assert!(mentioned("mail dev@example.com or a@b").is_empty());
    }

    #[test]
    fn splits_a_project_qualifier_off_a_name() {
        use super::split_project;
        assert_eq!(split_project("orchestrator"), (None, "orchestrator"));
        assert_eq!(
            split_project("ui:orchestrator"),
            (Some("ui"), "orchestrator")
        );
        assert_eq!(
            split_project("bridle-ui:external:orchestrator"),
            (Some("bridle-ui"), "external:orchestrator")
        );
    }

    #[test]
    fn a_trailing_colon_is_punctuation_and_a_project_qualifier_stays() {
        assert_eq!(mentioned("@aide: look"), ["aide"]);
        assert_eq!(mentioned("(@ui:orchestrator)"), ["ui:orchestrator"]);
    }
}
