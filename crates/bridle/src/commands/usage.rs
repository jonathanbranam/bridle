//! Usage and budget commands: usage, cost, budget.

use super::*;

pub(super) fn is_known_rate_limit_window(window: &str) -> bool {
    matches!(
        window,
        "five_hour" | "seven_day" | "seven_day_opus" | "seven_day_sonnet"
    )
}

pub(super) fn format_utilization(u: Option<f64>) -> String {
    u.map(|u| format!("{:.0}%", u * 100.0))
        .unwrap_or_else(|| "-".to_string())
}

pub(super) fn format_tokens(tokens: u64) -> String {
    format_tokens_impl(tokens as usize)
}

pub(super) fn format_tokens_impl(tokens: usize) -> String {
    if tokens >= 10000 {
        let k = tokens as f64 / 1000.0;
        if k >= 100.0 {
            format!("{:.0}k", k)
        } else {
            format!("{:.1}k", k)
        }
    } else {
        tokens.to_string()
    }
}

pub(super) fn format_cost_dollars(cost: f64) -> String {
    if cost >= 1.0 {
        format!("{:.2}", cost)
    } else {
        format!("{:.4}", cost)
    }
}

pub(super) async fn usage(cli: &Cli, args: &UsageArgs) -> Result<(), CliError> {
    let client = client_for_read(cli).await?;

    let since = args
        .since
        .as_deref()
        .map(|s| {
            parse_duration(s)
                .map(|d| Utc::now() - d)
                .ok_or_else(|| CliError::Other(anyhow::anyhow!("bad --since duration: {s:?}")))
        })
        .transpose()?;

    if let Some(window) = &args.history {
        let points = client
            .usage_history(&UsageHistoryQuery {
                window: window.clone(),
                since,
            })
            .await?;
        if cli.json {
            render::print_json(&points)?;
        } else {
            println!("{:<20} {:>6} RESETS", "OBSERVED", "USED");
            for p in &points {
                let resets = p
                    .resets_at
                    .map(|r| r.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "-".to_string());
                println!(
                    "{:<20} {:>6} {}",
                    p.observed_at.format("%Y-%m-%d %H:%M:%S"),
                    format_utilization(p.utilization),
                    resets
                );
            }
        }
        return Ok(());
    }

    // `--by role|model` (and `--since` on its own) go through the turns
    // ledger directly; the plain per-agent view keeps using the existing
    // endpoint, unfiltered, as before.
    if since.is_some() || matches!(args.by, Some(UsageByArg::Role) | Some(UsageByArg::Model)) {
        let by = match args.by {
            Some(UsageByArg::Role) => UsageGroupBy::Role,
            Some(UsageByArg::Model) => UsageGroupBy::Model,
            Some(UsageByArg::Agent) | None => UsageGroupBy::Agent,
        };
        let breakdown = client
            .usage_breakdown(&UsageBreakdownQuery {
                since,
                by: Some(by),
            })
            .await?;
        if cli.json {
            render::print_json(&breakdown)?;
        } else {
            // `wall` only means anything grouped by agent (see UsageGroup::wall_seconds).
            let show_wall = by == UsageGroupBy::Agent;
            if show_wall {
                println!(
                    "{:<20} {:>5} {:>12} {:>9} {:>8} {:>8} {:>8}",
                    "KEY", "TURNS", "TOKENS", "COST", "CACHE", "BUSY", "WALL"
                );
            } else {
                println!(
                    "{:<20} {:>5} {:>12} {:>9} {:>8} {:>8}",
                    "KEY", "TURNS", "TOKENS", "COST", "CACHE", "BUSY"
                );
            }
            for g in &breakdown.groups {
                let tokens =
                    g.tokens.input + g.tokens.output + g.tokens.cache_read + g.tokens.cache_write;
                let cache = g
                    .cache_hit_ratio
                    .map(|r| format!("{:.1}%", r * 100.0))
                    .unwrap_or_else(|| "-".to_string());
                let busy = format_duration_secs(g.busy_seconds);
                if show_wall {
                    let wall = g
                        .wall_seconds
                        .map(format_duration_secs)
                        .unwrap_or_else(|| "-".to_string());
                    println!(
                        "{:<20} {:>5} {:>12} {:>9} {:>8} {:>8} {:>8}",
                        g.key,
                        g.turns,
                        format_tokens(tokens),
                        format_cost_dollars(g.cost_usd_total),
                        cache,
                        busy,
                        wall
                    );
                } else {
                    println!(
                        "{:<20} {:>5} {:>12} {:>9} {:>8} {:>8}",
                        g.key,
                        g.turns,
                        format_tokens(tokens),
                        format_cost_dollars(g.cost_usd_total),
                        cache,
                        busy
                    );
                }
            }
            let t = &breakdown.total_tokens;
            let total_tokens = t.input + t.output + t.cache_read + t.cache_write;
            println!(
                "total: {} turns, {} tokens, ${}",
                breakdown.total_turns,
                format_tokens(total_tokens),
                format_cost_dollars(breakdown.total_cost_usd)
            );
            if let Some(ratio) = breakdown.cache_hit_ratio {
                println!("cache hit ratio: {:.1}%", ratio * 100.0);
            }
        }
        return Ok(());
    }

    let usage = client.usage().await?;
    if cli.json {
        render::print_json(&usage)?;
    } else {
        println!(
            "{:<12} {:<14} {:<10} {:>5} {:>12} {:>9} {:>8} {:>8}",
            "ID", "NAME", "ROLE", "TURNS", "TOKENS", "COST", "BUSY", "WALL"
        );
        for a in &usage.agents {
            let tokens =
                a.tokens.input + a.tokens.output + a.tokens.cache_read + a.tokens.cache_write;
            let name = if a.removed {
                format!("{} (rm)", a.name)
            } else {
                a.name.clone()
            };
            let wall = a
                .wall_seconds
                .map(format_duration_secs)
                .unwrap_or_else(|| "-".to_string());
            println!(
                "{:<12} {:<14} {:<10} {:>5} {:>12} {:>9} {:>8} {:>8}",
                a.agent,
                name,
                a.role,
                a.turns,
                format_tokens(tokens),
                format_cost_dollars(a.cost_usd_total),
                format_duration_secs(a.busy_seconds),
                wall
            );
        }
        let t = &usage.total_tokens;
        let total_tokens = t.input + t.output + t.cache_read + t.cache_write;
        println!(
            "total: {} turns, {} tokens, ${}",
            usage.total_turns,
            format_tokens(total_tokens),
            format_cost_dollars(usage.total_cost_usd)
        );
        if let Some(ratio) = usage.cache_hit_ratio {
            println!("cache hit ratio: {:.1}%", ratio * 100.0);
        }
        for rl in &usage.rate_limits {
            if is_known_rate_limit_window(&rl.window) {
                let resets = rl
                    .resets_at
                    .map(|r| format!(", resets {}", local_time(r)))
                    .unwrap_or_default();
                println!(
                    "{:<10} {}{resets}",
                    rl.window,
                    format_utilization(rl.utilization)
                );
            }
        }
        if !usage.interactive_today.is_empty() {
            println!("today, interactive (bridle statusline):");
            for row in &usage.interactive_today {
                let cost = row
                    .cost_usd
                    .map(|c| format!("${}", format_cost_dollars(c)))
                    .unwrap_or_else(|| "-".to_string());
                let ctx = match row.context_used_percentage {
                    Some(p) => format!("{:.0}%", p * 100.0),
                    None => "-".to_string(),
                };
                println!(
                    "  {} {:<10} {:>9} ctx {ctx}",
                    row.observed_at.format("%H:%M:%S"),
                    row.model.as_deref().unwrap_or("-"),
                    cost
                );
            }
        }
    }
    Ok(())
}

/// `bridle cost audit`: a static, local check (no daemon involved — it reads
/// `.bridle/config.toml` and `.bridle/cost-baseline.json` from the current
/// directory) of what bridle injects into agent context, per
/// docs/design/usage-and-budget.md ("Tracking token use over time") and
/// `bridle_daemon::cost_audit`.
pub(super) async fn cost(cli: &Cli, args: &CostArgs) -> Result<(), CliError> {
    match &args.action {
        CostAction::Audit(audit_args) => cost_audit(cli, audit_args).await,
    }
}

pub(super) async fn cost_audit(cli: &Cli, args: &CostAuditArgs) -> Result<(), CliError> {
    use bridle_daemon::config::Config;
    use bridle_daemon::cost_audit::{self, RoleAudit};

    let repo = std::env::current_dir().context("current directory")?;
    let config = Config::load(&repo).context("loading .bridle/config.toml")?;
    let current = cost_audit::measure(&config, &repo);
    let baseline = cost_audit::Baseline::load(&repo)
        .context("loading .bridle/cost-baseline.json")?
        .unwrap_or_default();
    let rows = cost_audit::compare(&current, &baseline);

    if cli.json {
        render::print_json(&rows)?;
    } else {
        println!(
            "{:<14} {:>9} {:>9} {:>8}",
            "ROLE", "CURRENT", "BASELINE", "CHANGE"
        );
        for r in &rows {
            print_cost_row(r);
        }
    }

    if args.check {
        let failing: Vec<&RoleAudit> = rows.iter().filter(|r| r.over_threshold).collect();
        if !failing.is_empty() {
            for r in &failing {
                eprintln!(
                    "role {} grew {:.1}% over baseline (threshold {:.0}%)",
                    r.role,
                    r.change_percent.unwrap_or(0.0),
                    cost_audit::GROWTH_THRESHOLD_PERCENT
                );
            }
            return Err(CliError::Other(anyhow::anyhow!(
                "{} role(s) exceeded the cost-growth threshold",
                failing.len()
            )));
        }
    }
    Ok(())
}

pub(super) fn print_cost_row(r: &bridle_daemon::cost_audit::RoleAudit) {
    let baseline = r
        .baseline_tokens
        .map(format_tokens_impl)
        .unwrap_or_else(|| "-".to_string());
    let change = r
        .change_percent
        .map(|c| format!("{c:+.1}%"))
        .unwrap_or_else(|| "new".to_string());
    let flag = if r.over_threshold { " !" } else { "" };
    println!(
        "{:<14} {:>9} {:>9} {:>8}{flag}",
        r.role,
        format_tokens_impl(r.current_tokens),
        baseline,
        change
    );
}

pub(super) async fn budget(cli: &Cli, args: &BudgetArgs) -> Result<(), CliError> {
    // Only the no-action form (`bridle budget`) is a read; every `BudgetAction`
    // writes, so it keeps the client-side check.
    let client = if args.action.is_none() {
        client_for_read(cli).await?
    } else {
        client_for(cli).await?
    };
    let budget = match &args.action {
        None => client.budget().await?,
        Some(BudgetAction::Hold(hold_args)) => {
            let until = resolve_hold_until(hold_args)?;
            client.budget_hold(&BudgetHoldRequest { until }).await?
        }
        Some(BudgetAction::Release) => client.budget_release().await?,
        Some(BudgetAction::Override(o)) => {
            let period = (o.period != "default").then(|| o.period.clone());
            let until = o.until.as_deref().map(parse_local_until).transpose()?;
            client
                .budget_override(&BudgetOverrideRequest { period, until })
                .await?
        }
        Some(BudgetAction::OverrideClear) => client.budget_override_clear().await?,
        Some(BudgetAction::MaxWorkers(a)) => {
            client
                .budget_max_workers(&MaxWorkersRequest { max_workers: a.n })
                .await?
        }
    };
    if cli.json {
        render::print_json(&budget)?;
    } else {
        if args.schedule {
            print_schedule(&budget);
            return Ok(());
        }
        println!("state  {}", budget.state);
        for reason in &budget.reasons {
            println!("why    {reason}");
        }
        if let Some(hold) = &budget.human_hold {
            let until = hold
                .until
                .map(|u| format!(", until {}", local_time(u)))
                .unwrap_or_else(|| ", until released".to_string());
            println!("hold   in force{until}");
        }
        if let Some(ov) = &budget.schedule_override {
            let period = ov.period.as_deref().unwrap_or("default");
            let until = ov
                .until
                .map(|u| format!(", until {}", local_time(u)))
                .unwrap_or_else(|| ", until cleared".to_string());
            println!("override {period}{until}");
        }
        let fh = &budget.five_hour;
        let from = match (fh.source.as_str(), &fh.period) {
            ("default", _) => "default".to_string(),
            (src, Some(p)) => format!("{src} {p}"),
            (src, None) => format!("{src} default"),
        };
        println!(
            "five_hour thresholds ({from}): hold {}, wind_down {}, stop {}",
            fh.hold_at, fh.wind_down_at, fh.stop_at
        );
        if let Some(span) = &fh.span {
            println!("  span {}", format_span(span));
        }
        if let Some(n) = fh.max_workers {
            println!("  period max_workers {n} (applied while overridden)");
        }
        match &fh.next_change {
            Some(n) => println!(
                "  next change {} -> {}: hold {}, wind_down {}, stop {}",
                local_time(n.at),
                n.period.as_deref().unwrap_or("default"),
                n.hold_at,
                n.wind_down_at,
                n.stop_at
            ),
            None => println!("  next change: none scheduled"),
        }
        for w in &budget.windows {
            let resets = w
                .resets_at
                .map(|r| format!(", resets {}", local_time(r)))
                .unwrap_or_default();
            let age = match w.age_secs {
                Some(s) => format!(", read {}", format_age(chrono::Duration::seconds(s as i64))),
                None => ", no reading".to_string(),
            };
            let status = match w.status.as_deref() {
                Some("allowed") | None => String::new(),
                Some(s) => format!(", status {s}"),
            };
            let staleness = if w.stale { " (stale)" } else { "" };
            println!(
                "  {:<16} {:<12} {}{resets}{status}{age}{staleness}",
                w.window,
                w.state.to_string(),
                format_utilization(w.utilization)
            );
        }
        let t = &budget.thresholds;
        let max_workers = match budget.max_workers_override {
            Some(n) => format!("{n} (override; configured {})", t.max_workers),
            None => t.max_workers.to_string(),
        };
        println!(
            "max_workers {max_workers}, wind_down_grace {}s, max_staleness {}s",
            t.wind_down_grace_secs, t.max_staleness_secs
        );
    }
    Ok(())
}

/// A time for the human, in the machine's local timezone (`bridle budget`'s
/// own choice; `--json` stays UTC).
pub(super) fn local_time(t: chrono::DateTime<Utc>) -> String {
    t.with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M %Z")
        .to_string()
}

pub(super) fn format_span(s: &bridle_api::types::ScheduleSpan) -> String {
    format!("{} {}-{}", s.days.join(","), s.start, s.end)
}

/// `bridle budget --schedule`: the resolved periods in match order (the
/// first match wins), start/end in host-local time.
pub(super) fn print_schedule(budget: &bridle_api::types::BudgetStatus) {
    if budget.schedule.is_empty() {
        println!("no [[budget.schedule]] periods; the plain [budget] thresholds always apply");
        return;
    }
    for p in &budget.schedule {
        println!(
            "{:<12} {}  hold {}, wind_down {}, stop {}{}",
            p.name,
            p.span
                .as_ref()
                .map_or_else(|| "override only".to_string(), format_span),
            p.hold_at,
            p.wind_down_at,
            p.stop_at,
            p.max_workers
                .map(|n| format!(", max_workers {n}"))
                .unwrap_or_default()
        );
    }
}

/// `--for 3h` (a plain duration from now) or `--until 18:00` (a local
/// `HH:MM`, resolved to the next occurrence: today if still ahead, else
/// tomorrow); neither given holds until `bridle budget release`.
pub(super) fn resolve_hold_until(
    args: &BudgetHoldArgs,
) -> Result<Option<chrono::DateTime<Utc>>, CliError> {
    if let Some(for_) = &args.for_ {
        let dur = parse_duration(for_)
            .ok_or_else(|| CliError::Other(anyhow::anyhow!("bad --for duration: {for_:?}")))?;
        return Ok(Some(Utc::now() + dur));
    }
    if let Some(until) = &args.until {
        return Ok(Some(parse_local_until(until)?));
    }
    Ok(None)
}

/// `HH:MM` local time, resolved to the next occurrence: today if still
/// ahead, else tomorrow. Shared by `bridle budget hold --until` and
/// `bridle budget override --until`.
pub(super) fn parse_local_until(until: &str) -> Result<chrono::DateTime<Utc>, CliError> {
    let (h, m) = until
        .split_once(':')
        .and_then(|(h, m)| Some((h.parse::<u32>().ok()?, m.parse::<u32>().ok()?)))
        .ok_or_else(|| CliError::Other(anyhow::anyhow!("bad --until time: {until:?}")))?;
    let now = Local::now();
    let mut target = now
        .date_naive()
        .and_hms_opt(h, m, 0)
        .and_then(|dt| Local.from_local_datetime(&dt).single())
        .ok_or_else(|| CliError::Other(anyhow::anyhow!("bad --until time: {until:?}")))?;
    if target <= now {
        target += chrono::Duration::days(1);
    }
    Ok(target.with_timezone(&Utc))
}

/// A plain `<n><unit>` duration (`s`/`m`/`h`/`d`), matching config.toml's.
pub(super) fn parse_duration(s: &str) -> Option<chrono::Duration> {
    let s = s.trim();
    let (num, unit) = s.split_at(s.len().checked_sub(1)?);
    let n: i64 = num.parse().ok()?;
    match unit {
        "s" => Some(chrono::Duration::seconds(n)),
        "m" => Some(chrono::Duration::minutes(n)),
        "h" => Some(chrono::Duration::hours(n)),
        "d" => Some(chrono::Duration::days(n)),
        _ => None,
    }
}
