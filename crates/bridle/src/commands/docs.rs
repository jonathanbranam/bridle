//! `bridle docs [topic]`: short overviews of how bridle works, embedded from `docs/cli/` so
//! they're available in any project without the bridle repo (ticket kae5). Local: no daemon.

use anyhow::anyhow;

use crate::cli::DocsArgs;
use crate::error::CliError;

/// (topic, one-line description, text). The order is the order `bridle docs` lists them.
const TOPICS: &[(&str, &str, &str)] = &[
    (
        "overview",
        "the parts of bridle and where state lives",
        include_str!("../../../../docs/cli/overview.md"),
    ),
    (
        "roles",
        "who decides what: human, manager, worker, ...",
        include_str!("../../../../docs/cli/roles.md"),
    ),
    (
        "priming-and-rules",
        "how rules and role prompts reach an agent",
        include_str!("../../../../docs/cli/priming-and-rules.md"),
    ),
    (
        "sessions",
        "hosted agents and interactive sessions",
        include_str!("../../../../docs/cli/sessions.md"),
    ),
    (
        "tasks-and-tickets",
        "task states, the task commands, tickets",
        include_str!("../../../../docs/cli/tasks-and-tickets.md"),
    ),
    (
        "queue",
        "the ordered tiers that say what to work on next",
        include_str!("../../../../docs/cli/queue.md"),
    ),
    (
        "messages",
        "sending, delivery timing, the inbox",
        include_str!("../../../../docs/cli/messages.md"),
    ),
    (
        "workflow-layers",
        "base, packs, project: how the workflow is layered",
        include_str!("../../../../docs/cli/workflow-layers.md"),
    ),
];

fn list() -> String {
    let mut out = String::from("Usage: bridle docs <topic>\n\nTopics:\n");
    for (name, about, _) in TOPICS {
        out.push_str(&format!("  {name:<20} {about}\n"));
    }
    out
}

pub fn run(args: &DocsArgs) -> Result<(), CliError> {
    match args.topic.as_deref() {
        None => print!("{}", list()),
        Some(topic) => {
            let (_, _, text) = TOPICS
                .iter()
                .find(|(name, ..)| *name == topic)
                .ok_or_else(|| anyhow!("unknown topic `{topic}`\n\n{}", list()))?;
            print!("{text}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_topic_has_text() {
        for (name, _, text) in TOPICS {
            assert!(!text.trim().is_empty(), "{name} is empty");
        }
    }

    #[test]
    fn every_embedded_file_is_listed() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/cli");
        for entry in std::fs::read_dir(dir).expect("docs/cli exists") {
            let path = entry.expect("dir entry").path();
            let stem = path
                .file_stem()
                .expect("stem")
                .to_string_lossy()
                .into_owned();
            assert!(
                TOPICS.iter().any(|(name, ..)| *name == stem),
                "docs/cli/{stem}.md is not listed in TOPICS"
            );
        }
    }

    #[test]
    fn unknown_topic_errors() {
        let args = DocsArgs {
            topic: Some("nope".into()),
        };
        assert!(run(&args).is_err());
    }
}
