//! `bridle schedule add|list|rm` (docs/design/cli.md, "Scheduled messages").

use bridle_api::types::{Schedule, ScheduleAddRequest};
use chrono::{DateTime, Utc};

use super::{client_for, client_for_read, misc::require_body, read_text};
use crate::cli::{Cli, ScheduleAction, ScheduleArgs};
use crate::error::CliError;
use crate::render;

/// The time as the schedule's zone reads it, with UTC beside it.
fn show_time(t: DateTime<Utc>, tz: &str) -> String {
    match tz.parse::<chrono_tz::Tz>() {
        Ok(z) => format!(
            "{} ({})",
            t.with_timezone(&z).format("%Y-%m-%d %H:%M %Z"),
            t.format("%H:%M UTC")
        ),
        Err(_) => t.to_rfc3339(),
    }
}

fn when(s: &Schedule) -> String {
    s.next_fire_at
        .map_or_else(|| "-".to_string(), |t| show_time(t, &s.tz))
}

pub(super) async fn run(cli: &Cli, args: &ScheduleArgs) -> Result<(), CliError> {
    match &args.action {
        ScheduleAction::Add {
            to,
            at,
            cron,
            tz,
            message,
            message_file,
        } => {
            let body = require_body(read_text(message, message_file, "message")?)?;
            let req = ScheduleAddRequest {
                to: to.clone(),
                at: at.clone(),
                cron: cron.clone(),
                tz: tz.clone(),
                body,
            };
            let s = client_for(cli).await?.add_schedule(&req).await?;
            if cli.json {
                render::print_json(&s)?;
            } else {
                println!("{} -> {}, next {}", s.id, s.target, when(&s));
            }
        }
        ScheduleAction::List { all } => {
            let list = client_for_read(cli).await?.list_schedules(*all).await?;
            if cli.json {
                render::print_json(&list)?;
            } else if list.is_empty() {
                println!("no schedules");
            } else {
                for s in &list {
                    let what = s
                        .cron
                        .as_deref()
                        .map_or("once".to_string(), |c| format!("cron {c}"));
                    let first = s.body.lines().next().unwrap_or("");
                    println!(
                        "{}  {:<6} {:<24} {}  {}  {}",
                        s.id,
                        s.state,
                        s.target,
                        when(s),
                        what,
                        first
                    );
                }
            }
        }
        ScheduleAction::Rm { id } => {
            client_for(cli).await?.remove_schedule(id).await?;
            if !cli.json {
                println!("removed {id}");
            }
        }
    }
    Ok(())
}
