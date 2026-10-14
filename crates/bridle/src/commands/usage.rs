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

/// External and interactive sessions, next to the hosted agents' table. Marked as external;
/// a figure no session reported is "not measured", never 0.
fn print_external(external: &[ExternalUsage]) {
    if external.is_empty() {
        println!("external sessions: not measured (no statusline report yet)");
        return;
    }
    println!("external sessions (statusline reports; not in the totals above):");
    for e in external {
        let cost = e
            .cost_usd
            .map(|c| format!("${}", format_cost_dollars(c)))
            .unwrap_or_else(|| "not measured".to_string());
        let tokens = match (e.input_tokens, e.output_tokens) {
            (None, None) => "not measured".to_string(),
            (i, o) => format!("{} tokens", format_tokens(i.unwrap_or(0) + o.unwrap_or(0))),
        };
        println!(
            "  {:<28} {:>2} sessions {:>14} {:>16}",
            e.key, e.sessions, tokens, cost
        );
    }
}

/// `--agent`: resolve the name or id through the instance list (so removed agents work), then
/// print the series. A name shared by several instances picks the one with the latest turn.
async fn usage_agent_series(
    cli: &Cli,
    client: &Client,
    who: &str,
    since: Option<chrono::DateTime<Utc>>,
    bucket: Option<UsageBucketArg>,
) -> Result<(), CliError> {
    let instances = client
        .usage_instances(&UsageInstancesQuery::default())
        .await?;
    let inst = instances
        .iter()
        .find(|i| i.agent_id == who)
        .or_else(|| instances.iter().find(|i| i.name == who))
        .ok_or_else(|| CliError::Other(anyhow::anyhow!("no usage recorded for agent {who:?}")))?;
    let bucket = match bucket {
        Some(UsageBucketArg::Hour) => UsageBucket::Hour,
        Some(UsageBucketArg::Day) | None => UsageBucket::Day,
    };
    let series = client
        .usage_series(
            &inst.agent_id,
            &UsageSeriesQuery {
                since,
                bucket: Some(bucket),
            },
        )
        .await?;
    if cli.json {
        return Ok(render::print_json(&series)?);
    }
    let fmt = if bucket == UsageBucket::Hour {
        "%Y-%m-%d %H:00"
    } else {
        "%Y-%m-%d"
    };
    println!(
        "{} ({}), {}",
        inst.name,
        inst.agent_id,
        if inst.external {
            "external"
        } else if inst.removed {
            "removed"
        } else {
            "live"
        }
    );
    println!(
        "{:<16} {:>5} {:>10} {:>10} {:>10} {:>10} {:>9} {:>8}",
        "BUCKET", "TURNS", "INPUT", "OUTPUT", "CACHE-R", "CACHE-W", "COST", "BUSY"
    );
    for p in &series.points {
        println!(
            "{:<16} {:>5} {:>10} {:>10} {:>10} {:>10} {:>9} {:>8}",
            p.start.format(fmt),
            p.turns.map_or("-".to_string(), |t| t.to_string()),
            format_tokens(p.tokens.input),
            format_tokens(p.tokens.output),
            format_tokens(p.tokens.cache_read),
            format_tokens(p.tokens.cache_write),
            format_cost_dollars(p.cost_usd),
            p.busy_seconds.map_or("-".to_string(), format_duration_secs),
        );
    }
    let cost: f64 = series.points.iter().map(|p| p.cost_usd).sum();
    if inst.external {
        println!("total: ${}", format_cost_dollars(cost));
    } else {
        let turns: u32 = series.points.iter().filter_map(|p| p.turns).sum();
        println!("total: {turns} turns, ${}", format_cost_dollars(cost));
    }
    Ok(())
}

fn pct(u: Option<f64>) -> String {
    format_utilization(u)
}

/// `--week`: the latest week from the reset, its days and the five-hour windows inside it.
async fn usage_week(
    cli: &Cli,
    client: &Client,
    since: Option<chrono::DateTime<Utc>>,
) -> Result<(), CliError> {
    let periods = client.usage_periods(&UsagePeriodsQuery { since }).await?;
    if cli.json {
        return Ok(render::print_json(&periods)?);
    }
    let Some(week) = periods.weeks.last() else {
        println!("no seven_day readings yet");
        return Ok(());
    };
    println!(
        "week {} to {} ({}): peak {}{}",
        week.starts_at.format("%Y-%m-%d %H:%M"),
        week.resets_at.format("%Y-%m-%d %H:%M"),
        periods.timezone,
        pct(Some(week.peak)),
        if week.hit_limit {
            ", hit the limit"
        } else {
            ""
        }
    );
    println!("{:<12} {:>9} {:>9}  HIT", "DAY", "7-DAY", "5-HOUR");
    for d in &week.days {
        println!(
            "{:<12} {:>9} {:>9}  {}",
            d.date,
            pct(d.peak_seven_day),
            pct(d.peak_five_hour),
            if d.hit_five_hour { "5h limit" } else { "" }
        );
    }
    println!("{:<18} {:>6}  HIT", "5-HOUR WINDOW ENDS", "PEAK");
    for w in periods
        .windows
        .iter()
        .filter(|w| w.resets_at > week.starts_at && w.starts_at < week.resets_at)
    {
        println!(
            "{:<18} {:>6}  {}",
            w.resets_at.format("%Y-%m-%d %H:%M"),
            pct(Some(w.peak)),
            if w.hit_limit { "limit" } else { "" }
        );
    }
    Ok(())
}

/// `--by name|session`: the daemon's role -> name -> session tree, flattened to indented rows.
async fn usage_groups(
    cli: &Cli,
    client: &Client,
    since: Option<chrono::DateTime<Utc>>,
    with_sessions: bool,
) -> Result<(), CliError> {
    let groups = client
        .usage_groups(&UsageGroupsQuery { since, until: None })
        .await?;
    if cli.json {
        return Ok(render::print_json(&groups)?);
    }
    println!(
        "{:<30} {:>5} {:>9} {:>9} {:>8}",
        "KEY", "TURNS", "COST", "GROWTH", "SHARE"
    );
    for role in &groups.roles {
        print_group(role, 0);
        for name in &role.children {
            print_group(name, 1);
            if with_sessions {
                for s in &name.children {
                    print_group(s, 2);
                }
            }
        }
    }
    println!("total: ${}", format_cost_dollars(groups.total_cost_usd));
    println!(
        "seven_day points gained: {:.1}{}",
        groups.seven_day_points,
        groups
            .points_per_dollar
            .map(|p| format!(", {p:.2} points per dollar"))
            .unwrap_or_default()
    );
    println!("{}", groups.share_note);
    Ok(())
}

fn print_group(g: &UsageShareGroup, depth: usize) {
    let key = if g.key.is_empty() {
        "(no session)"
    } else {
        &g.key
    };
    let key = format!(
        "{}{key}{}",
        "  ".repeat(depth),
        if g.external { " (ext)" } else { "" }
    );
    println!(
        "{:<30} {:>5} {:>9} {:>9} {:>7.1}",
        key,
        g.turns.map_or("-".to_string(), |t| t.to_string()),
        format_cost_dollars(g.cost_usd),
        g.context_growth,
        g.share_points
    );
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

    if let Some(who) = &args.agent {
        return usage_agent_series(cli, &client, who, since, args.bucket).await;
    }

    if args.week {
        return usage_week(cli, &client, since).await;
    }

    if matches!(args.by, Some(UsageByArg::Name) | Some(UsageByArg::Session)) {
        let with_sessions = matches!(args.by, Some(UsageByArg::Session));
        return usage_groups(cli, &client, since, with_sessions).await;
    }

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
            Some(UsageByArg::Agent | UsageByArg::Name | UsageByArg::Session) | None => {
                UsageGroupBy::Agent
            }
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
            print_external(&breakdown.external);
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
        print_external(&usage.external);
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
