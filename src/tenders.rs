//! Dated tender metadata and reports; measurement identities remain unchanged.
use super::*;

#[derive(Debug, Deserialize)]
pub(super) struct Tender {
    pub id: String,
    pub start: String,
    pub end: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Window {
    start: DateTime<Utc>,
    end: DateTime<Utc>, // Exclusive; config dates use UTC midnight.
}

fn date(value: &str) -> Result<DateTime<Utc>> {
    let d = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .with_context(|| format!("invalid tender date {value:?}; use YYYY-MM-DD"))?;
    if d.format("%Y-%m-%d").to_string() != value {
        bail!("tender dates must use YYYY-MM-DD");
    }
    Ok(DateTime::from_naive_utc_and_offset(
        d.and_hms_opt(0, 0, 0).unwrap(),
        Utc,
    ))
}

fn bounds(t: &Tender) -> Result<Window> {
    let w = Window {
        start: date(&t.start)?,
        end: t
            .end
            .as_deref()
            .map(date)
            .transpose()?
            .unwrap_or(DateTime::<Utc>::MAX_UTC),
    };
    if w.end <= w.start {
        bail!("tender {} must end after its start", t.id);
    }
    Ok(w)
}

pub(super) fn validate(config: &Config) -> Result<()> {
    let mut ids = HashSet::new();
    for t in &config.tenders {
        if t.id == "__current" {
            bail!("tender ID __current is reserved for the dashboard selector");
        }
        if t.id.is_empty()
            || !t
                .id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            bail!("tender IDs must contain only ASCII letters, digits, - or _");
        }
        if !ids.insert(t.id.as_str()) {
            bail!("duplicate tender ID {}", t.id);
        }
        bounds(t)?;
    }
    let mut endpoints = HashSet::new();
    for e in &config.endpoints {
        if !endpoints.insert((&e.url, &e.network, &e.zone)) {
            bail!(
                "duplicate endpoint identity: {} / {} / {}",
                e.url,
                e.network,
                e.zone
            );
        }
        let mut memberships = HashSet::new();
        for id in &e.tenders {
            if !ids.contains(id.as_str()) {
                bail!("endpoint {} references unknown tender {id}", e.url);
            }
            if !memberships.insert(id) {
                bail!("endpoint {} repeats tender {id}", e.url);
            }
        }
    }
    Ok(())
}

pub(super) fn render_metadata(
    config: &Config,
    results: &[EndpointResult],
    now: DateTime<Utc>,
) -> Result<String> {
    let mut out = String::from("\n# HELP rpc_tender_active Whether a configured tender is active now (UTC).\n# TYPE rpc_tender_active gauge\n");
    for t in &config.tenders {
        let w = bounds(t)?;
        out.push_str(&format!(
            "rpc_tender_active{{tender=\"{}\",zone=\"{}\"}} {}\n",
            escape_label(&t.id),
            escape_label(&config.zone),
            u8::from(w.start <= now && now < w.end)
        ));
    }
    out.push_str("# HELP rpc_endpoint_info Configured endpoint membership; empty tender means non-tender.\n# TYPE rpc_endpoint_info gauge\n");
    // No explicit timestamps: metadata remains queryable between 15-minute runs.
    for r in results {
        let e = &r.endpoint;
        let ids = if e.tenders.is_empty() {
            vec![""]
        } else {
            e.tenders.iter().map(String::as_str).collect()
        };
        for id in ids {
            out.push_str(&format!(
                "rpc_endpoint_info{{wss=\"{}\",network=\"{}\",zone=\"{}\",tender=\"{}\"}} 1\n",
                escape_label(&e.url),
                escape_label(&e.network),
                escape_label(&e.zone),
                escape_label(id)
            ));
        }
        // A synthetic selector makes Grafana's saved "Current" selection
        // dynamic without changing the dashboard or measurement identities.
        // Emit zero too, so ended memberships are not resurrected by lookback.
        let active = config.tenders.iter().any(|t| {
            e.tenders.contains(&t.id) && bounds(t).is_ok_and(|w| w.start <= now && now < w.end)
        });
        out.push_str(&format!(
            "rpc_endpoint_info{{wss=\"{}\",network=\"{}\",zone=\"{}\",tender=\"__current\"}} {}\n",
            escape_label(&e.url),
            escape_label(&e.network),
            escape_label(&e.zone),
            u8::from(active)
        ));
    }
    Ok(out)
}

fn merge(mut windows: Vec<Window>) -> Vec<Window> {
    windows.sort_by_key(|w| w.start);
    let mut merged: Vec<Window> = Vec::new();
    for w in windows {
        if let Some(last) = merged.last_mut() {
            if w.start <= last.end {
                last.end = last.end.max(w.end);
                continue;
            }
        }
        merged.push(w);
    }
    merged
}

fn eligible(
    config: &Config,
    endpoint: &Endpoint,
    args: &ReportCommand,
    period: &Window,
) -> Result<Vec<Window>> {
    if args.all_endpoints {
        return Ok(vec![period.clone()]);
    }
    let mut windows = Vec::new();
    for t in &config.tenders {
        if !endpoint.tenders.contains(&t.id)
            || (!args.tenders.is_empty() && !args.tenders.contains(&t.id))
        {
            continue;
        }
        let w = bounds(t)?;
        let start = w.start.max(period.start);
        let end = w.end.min(period.end);
        if start < end {
            windows.push(Window { start, end });
        }
    }
    Ok(merge(windows))
}

fn selector(metric: &str, e: &Endpoint, w: &Window, extra: &str) -> String {
    // Prometheus ranges are left-open/right-closed. Shift by 1ms so config's
    // [start, end) convention is respected for millisecond-resolution samples.
    format!(
        "{metric}{{wss=\"{}\",network=\"{}\",zone=\"{}\"{extra}}}[{}ms] @ {:.3}",
        escape_label(&e.url),
        escape_label(&e.network),
        escape_label(&e.zone),
        (w.end - w.start).num_milliseconds(),
        (w.end.timestamp_millis() - 1) as f64 / 1000.0
    )
}

fn total_expr(
    metric: &str,
    e: &Endpoint,
    windows: &[Window],
    extra: &str,
    operation: &str,
) -> String {
    windows
        .iter()
        .map(|w| {
            let s = selector(metric, e, w, extra);
            let term = match operation {
                "square" => format!(
                    "(stdvar_over_time({s}) + avg_over_time({s}) ^ 2) * count_over_time({s})"
                ),
                _ => format!("{operation}_over_time({s})"),
            };
            format!("(sum({term}) or vector(0))")
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

async fn value(client: &PrometheusClient, expr: &str, now: i64) -> Result<Option<f64>> {
    Ok(client
        .query(expr, now)
        .await?
        .first()
        .and_then(|s| s.value.1.parse::<f64>().ok())
        .filter(|x| x.is_finite()))
}

async fn moments(
    client: &PrometheusClient,
    metric: &str,
    e: &Endpoint,
    windows: &[Window],
    now: i64,
) -> Result<(Option<f64>, Option<f64>)> {
    let sum = total_expr(metric, e, windows, "", "sum");
    let count = total_expr(metric, e, windows, "", "count");
    let square = total_expr(metric, e, windows, "", "square");
    let mean = format!("({sum}) / ({count})");
    let avg = value(client, &mean, now).await?;
    let stddev = value(
        client,
        &format!("sqrt(clamp_min(({square}) / ({count}) - ({mean}) ^ 2, 0))"),
        now,
    )
    .await?;
    Ok((avg, stddev))
}

async fn calls(
    client: &PrometheusClient,
    chain: &str,
    job: &str,
    windows: &[Window],
    now: i64,
) -> Result<Option<f64>> {
    let mut total = 0.0;
    let mut found = false;
    for w in windows {
        let expr = format!(
            "sum(increase(substrate_rpc_calls_started{{chain=\"{}\",job=\"{}\"}}[{}ms] @ {:.3}))",
            escape_label(chain),
            escape_label(job),
            (w.end - w.start).num_milliseconds(),
            (w.end.timestamp_millis() - 1) as f64 / 1000.0
        );
        if let Some(v) = value(client, &expr, now).await? {
            total += v;
            found = true;
        }
    }
    Ok(found.then_some(total / 1_000_000.0))
}

fn average(values: impl Iterator<Item = Option<f64>>) -> Option<f64> {
    let v: Vec<_> = values.flatten().collect();
    (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
}

fn report_tender_ids(
    config: &Config,
    e: &Endpoint,
    args: &ReportCommand,
    period: &Window,
) -> Result<Vec<String>> {
    let mut ids = Vec::new();
    for t in &config.tenders {
        let w = bounds(t)?;
        if e.tenders.contains(&t.id)
            && (args.all_endpoints || args.tenders.is_empty() || args.tenders.contains(&t.id))
            && w.start < period.end
            && period.start < w.end
        {
            ids.push(t.id.clone());
        }
    }
    Ok(ids)
}

pub(super) async fn run_report(config: &Config, args: &ReportCommand) -> Result<()> {
    for id in &args.tenders {
        if !config.tenders.iter().any(|t| &t.id == id) {
            bail!("unknown tender {id}");
        }
    }
    let short = match &args.mode {
        Some(ReportMode::Short(s)) => Some(s),
        _ => None,
    };
    let p = if let Some(s) = short {
        ReportPeriod::from_short_args(s)?
    } else {
        ReportPeriod::from_args(
            args.month.as_deref(),
            args.start.as_deref(),
            args.end.as_deref(),
        )?
    };
    let window = Window {
        start: p.start,
        end: if short.is_none() && args.month.is_some() {
            p.end + TimeDelta::seconds(1)
        } else {
            p.end
        },
    };
    let mut selected = Vec::new();
    for e in &config.endpoints {
        let windows = eligible(config, e, args, &window)?;
        if !windows.is_empty() {
            selected.push((e, windows));
        }
    }
    selected.sort_by(|(a, _), (b, _)| {
        a.network
            .cmp(&b.network)
            .then(a.url.cmp(&b.url))
            .then(a.zone.cmp(&b.zone))
    });
    if selected.is_empty() {
        bail!("no endpoints eligible for the selected tender(s) and dates; use --all-endpoints to bypass tender filtering");
    }
    let client = PrometheusClient::new(
        args.prometheus_url
            .as_deref()
            .unwrap_or(&config.report.prometheus_url),
    )?;
    let mut rows = Vec::new();
    let mut report_tenders = HashSet::new();
    for (e, windows) in selected {
        let ids = report_tender_ids(config, e, args, &window)?;
        report_tenders.extend(ids.iter().cloned());
        let tender_names = if ids.is_empty() {
            "—".to_owned()
        } else {
            ids.join(", ")
        };
        let connect = if short.is_none() {
            moments(&client, "rpc_connect", e, &windows, p.end_timestamp()).await?
        } else {
            (None, None)
        };
        let block = if short.is_none() {
            moments(&client, "rpc_getblock", e, &windows, p.end_timestamp()).await?
        } else {
            (None, None)
        };
        let uptime = if short.is_none() {
            let sum = total_expr("rpc_error", e, &windows, ",error=\"getblock\"", "sum");
            let n = total_expr("rpc_error", e, &windows, ",error=\"getblock\"", "count");
            value(
                &client,
                &format!("100 * (1 - ({sum}) / ({n}))"),
                p.end_timestamp(),
            )
            .await?
        } else {
            None
        };
        let version = if short.is_none() {
            let w = windows.last().unwrap();
            let expr = format!("last_over_time({})", selector("rpc_version", e, w, ""));
            client
                .query(&expr, p.end_timestamp())
                .await?
                .last()
                .and_then(|s| s.metric.get("version"))
                .map(|v| normalize_version(v))
        } else {
            None
        };
        let usage = if let (Some(chain), Some(job)) = (
            config.report.network_to_chain.get(&e.network),
            e.provider.as_deref(),
        ) {
            calls(&client, chain, job, &windows, p.end_timestamp()).await?
        } else {
            None
        };
        rows.push((e, connect, block, uptime, version, usage, tender_names));
    }
    println!(
        "## RPC providers report for {} ({} - {})",
        short.and_then(|s| s.label.as_deref()).unwrap_or(&p.label),
        format_report_datetime(window.start),
        format_report_datetime(window.end - TimeDelta::seconds(1))
    );
    println!(
        "\n- **Tenders**: {}",
        config
            .tenders
            .iter()
            .filter(|t| report_tenders.contains(&t.id))
            .map(|t| t.id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    if args.all_endpoints {
        println!("- **Endpoint selection**: all endpoints, including non-tender endpoints");
    }
    if short.is_none() {
        println!(
            "- **Connect Time**: Avg = {} s (Std Dev = {} s)",
            format_optional_f64(average(rows.iter().map(|r| r.1 .0)), 2),
            format_optional_f64(average(rows.iter().map(|r| r.1 .1)), 2)
        );
        println!(
            "- **Block Retrieval Time**: Avg = {} s (Std Dev = {} s)",
            format_optional_f64(average(rows.iter().map(|r| r.2 .0)), 2),
            format_optional_f64(average(rows.iter().map(|r| r.2 .1)), 2)
        );
        println!("- **Uptime**: % of time endpoint could fetch block");
        println!("- **Binary Version**: Node version at end of period");
    }
    println!(
        "- **RPC Calls**: RPC calls in the selected {} day period in millions",
        p.days
    );
    if short.is_none() {
        println!("\n| Endpoint | Zone | Network | Tender | Average Connect Time (s) | Average Block Retrieval Time (s) | Uptime (%) | Binary Version | RPC Calls (M) |");
        println!("|---|---|---|---|---|---|---|---|---|");
    } else {
        println!("\n| Endpoint | Zone | Network | Tender | RPC Calls (M) |");
        println!("|---|---|---|---|---|");
    }
    for (e, c, b, u, v, usage, tender_names) in rows {
        if short.is_some() {
            println!(
                "| {} | {} | {} | {} | {} |",
                report_endpoint_url(&e.url),
                e.zone,
                e.network,
                tender_names,
                format_optional_f64(usage, 1),
            );
        } else {
            println!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                report_endpoint_url(&e.url),
                e.zone,
                e.network,
                tender_names,
                format_optional_f64(c.0, 2),
                format_optional_f64(b.0, 2),
                format_optional_f64(u, 2),
                v.unwrap_or_else(|| "N/A".into()),
                format_optional_f64(usage, 1)
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> Config {
        toml::from_str(
            r#"
output_path="/tmp/test.prom"
lock_path="/tmp/test.lock"
[[tenders]]
id="5"
start="2026-10-01"
end="2026-11-10"
[[tenders]]
id="5c"
start="2026-11-01"
end="2027-07-01"
[[endpoints]]
url="wss://test.example"
zone="eu-central"
network="polkadot"
tenders=["5","5c"]
"#,
        )
        .unwrap()
    }
    fn args(extra: &[&str]) -> ReportCommand {
        let mut argv = vec!["rpc-monitor", "report", "--month", "2026-11"];
        argv.extend_from_slice(extra);
        match Args::try_parse_from(argv).unwrap().command.unwrap() {
            Command::Report(a) => a,
        }
    }
    #[test]
    fn overlaps_merge_without_double_counting() {
        let c = config();
        validate(&c).unwrap();
        let p = Window {
            start: date("2026-10-01").unwrap(),
            end: date("2026-12-01").unwrap(),
        };
        assert_eq!(
            eligible(&c, &c.endpoints[0], &args(&[]), &p).unwrap(),
            vec![p]
        );
    }
    #[test]
    fn october_excludes_november_tender_and_override_respects_dates() {
        let c = config();
        let p = Window {
            start: date("2026-10-01").unwrap(),
            end: date("2026-11-01").unwrap(),
        };
        assert!(
            eligible(&c, &c.endpoints[0], &args(&["--tender", "5c"]), &p)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            eligible(&c, &c.endpoints[0], &args(&["--all-endpoints"]), &p).unwrap(),
            vec![p]
        );
    }
    #[test]
    fn disjoint_windows_preserve_gap() {
        let w = |a, b| Window {
            start: date(a).unwrap(),
            end: date(b).unwrap(),
        };
        assert_eq!(
            merge(vec![
                w("2026-10-01", "2026-10-05"),
                w("2026-10-10", "2026-10-12")
            ])
            .len(),
            2
        );
    }
    #[test]
    fn rejects_unknown_duplicate_and_reversed_definitions() {
        let mut c = config();
        c.endpoints[0].tenders.push("unknown".into());
        assert!(validate(&c).is_err());
        let mut c = config();
        c.tenders[0].end = Some("2026-09-01".into());
        assert!(validate(&c).is_err());
        let mut c = config();
        c.tenders[1].id = "5".into();
        assert!(validate(&c).is_err());
    }
    #[test]
    fn active_is_start_inclusive_end_exclusive() {
        let c = config();
        let r = render_metadata(&c, &[], date("2026-11-01").unwrap()).unwrap();
        assert!(r.contains("tender=\"5c\",zone=\"\"} 1"));
        let r = render_metadata(&c, &[], date("2027-07-01").unwrap()).unwrap();
        assert!(r.contains("tender=\"5c\",zone=\"\"} 0"));
    }
    #[test]
    fn cli_conflicts_and_repeatable_selection() {
        assert_eq!(args(&["--tender", "5", "--tender", "5c"]).tenders.len(), 2);
        assert!(Args::try_parse_from([
            "rpc-monitor",
            "report",
            "--all-endpoints",
            "--tender",
            "5"
        ])
        .is_err());
    }
    #[test]
    fn metadata_keeps_measurement_identity_and_current_can_turn_off() {
        let mut c = config();
        c.zone = "eu-central".into();
        let mut r = EndpointResult {
            endpoint: c.endpoints[0].clone(),
            archive_status: ArchiveStatus::Archive,
            get_block_secs: Some(0.2),
            block_height: Some(1001),
            connect_secs: Some(0.1),
            version: Some("1.25.0".into()),
            errors: Vec::new(),
        };
        let original = render_prometheus(
            &c.zone,
            123,
            1.0,
            &[EndpointResult {
                endpoint: r.endpoint.clone(),
                archive_status: r.archive_status,
                get_block_secs: r.get_block_secs,
                block_height: r.block_height,
                connect_secs: r.connect_secs,
                version: r.version.clone(),
                errors: Vec::new(),
            }],
        );
        let active =
            render_metadata(&c, std::slice::from_ref(&r), date("2026-11-01").unwrap()).unwrap();
        assert!(active.contains("tender=\"__current\"} 1\n"));
        let inactive =
            render_metadata(&c, std::slice::from_ref(&r), date("2027-07-01").unwrap()).unwrap();
        assert!(inactive.contains("tender=\"__current\"} 0\n"));
        r.endpoint.tenders.clear();
        let metadata =
            render_metadata(&c, std::slice::from_ref(&r), date("2026-11-01").unwrap()).unwrap();
        assert!(metadata.contains("tender=\"\"} 1\n"));
        assert_eq!(original, render_prometheus(&c.zone, 123, 1.0, &[r]));
    }
    #[test]
    fn range_boundary_includes_start_but_excludes_end() {
        let c = config();
        let w = Window {
            start: date("2026-11-01").unwrap(),
            end: date("2026-11-02").unwrap(),
        };
        let s = selector("rpc_connect", &c.endpoints[0], &w, "");
        assert!(s.contains("[86400000ms]"));
        assert!(s.ends_with("1793577599.999"));
    }
}
