use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, NaiveDate, TimeDelta, Utc};
use clap::{Args as ClapArgs, Parser, Subcommand};
use fs2::FileExt;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::net::{lookup_host, TcpStream};
use tokio::time::timeout;
use tokio_rustls::{rustls::RootCertStore, TlsConnector};
use tokio_tungstenite::{client_async, connect_async, tungstenite::Message};
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Monitor Substrate RPC endpoints and write Prometheus textfile metrics."
)]
struct Args {
    #[arg(
        short,
        long,
        env = "RPC_MONITOR_CONFIG",
        default_value = "monitor.toml"
    )]
    config: PathBuf,

    #[arg(short = 'z', long, env = "RPC_MONITOR_ZONE")]
    zone: Option<String>,

    #[arg(long, env = "RPC_MONITOR_OUTPUT")]
    output: Option<PathBuf>,

    #[arg(long, env = "RPC_MONITOR_DRY_RUN")]
    dry_run: bool,

    #[arg(long, env = "RPC_MONITOR_CHECK_IPS")]
    check_ips: bool,

    #[arg(long, env = "RPC_MONITOR_ALL_ZONES")]
    all_zones: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    Report(ReportCommand),
}

#[derive(ClapArgs, Debug)]
struct ReportCommand {
    #[arg(long)]
    month: Option<String>,

    #[arg(long)]
    prometheus_url: Option<String>,

    #[command(subcommand)]
    mode: Option<ReportMode>,
}

#[derive(Subcommand, Debug)]
enum ReportMode {
    /// Generate the compact RPC-calls-only tender report for an explicit period.
    Short(ShortReportCommand),
}

#[derive(ClapArgs, Debug)]
struct ShortReportCommand {
    /// Start boundary as DD-MM-YYYY, YYYY-MM-DD or an RFC3339 timestamp.
    #[arg(long)]
    start: String,

    /// End boundary as DD-MM-YYYY, YYYY-MM-DD or an RFC3339 timestamp.
    #[arg(long)]
    end: String,

    /// Optional report title suffix.
    #[arg(long)]
    label: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default)]
    zone: String,
    output_path: PathBuf,
    lock_path: PathBuf,
    #[serde(default = "default_concurrency")]
    concurrency: usize,
    #[serde(default)]
    timeouts: Timeouts,
    #[serde(default)]
    zero_hashes: HashMap<String, String>,
    #[serde(default)]
    report: ReportConfig,
    endpoints: Vec<Endpoint>,
}

#[derive(Clone, Debug, Deserialize)]
struct ReportConfig {
    #[serde(default = "default_prometheus_url")]
    prometheus_url: String,
    #[serde(default)]
    network_to_chain: HashMap<String, String>,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            prometheus_url: default_prometheus_url(),
            network_to_chain: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Timeouts {
    #[serde(default = "default_connect_secs")]
    connect_secs: u64,
    #[serde(default = "default_block_zero_secs")]
    block_zero_secs: u64,
    #[serde(default = "default_get_block_secs")]
    get_block_secs: u64,
    #[serde(default = "default_block_height_secs")]
    block_height_secs: u64,
    #[serde(default = "default_version_secs")]
    version_secs: u64,
}

impl Default for Timeouts {
    fn default() -> Self {
        Self {
            connect_secs: default_connect_secs(),
            block_zero_secs: default_block_zero_secs(),
            get_block_secs: default_get_block_secs(),
            block_height_secs: default_block_height_secs(),
            version_secs: default_version_secs(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Endpoint {
    url: String,
    network: String,
    zone: String,
    #[allow(dead_code)]
    provider: Option<String>,
}

#[derive(Debug)]
struct EndpointResult {
    endpoint: Endpoint,
    block_zero_secs: Option<f64>,
    get_block_secs: Option<f64>,
    block_height: Option<u64>,
    connect_secs: Option<f64>,
    version: Option<String>,
    errors: Vec<CheckError>,
    skipped: Vec<&'static str>,
}

#[derive(Clone, Debug)]
struct CheckError {
    kind: &'static str,
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow!("failed to install rustls ring crypto provider"))?;

    init_logging();

    let normalized_args = normalize_args(std::env::args_os());
    if normalized_args
        .iter()
        .any(|arg| arg == "--version" || arg == "-V")
    {
        println!("rpc-monitor {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let args = Args::parse_from(normalized_args);
    let mut config = load_config(&args.config)?;
    if let Some(zone) = args.zone {
        config.zone = zone;
    }
    if let Some(output) = args.output {
        config.output_path = output;
    }

    if let Some(Command::Report(report_args)) = args.command {
        run_report(&config, &report_args).await?;
        return Ok(());
    }

    if args.check_ips {
        validate_config(&config, false)?;
        let endpoints = endpoints_for_ip_check(&config, args.all_zones);
        run_ip_checks(&config, endpoints).await?;
        return Ok(());
    }

    validate_config(&config, true)?;

    let lock = acquire_lock(&config.lock_path)?;
    let run_started = Instant::now();
    let timestamp_ms = unix_timestamp_millis()?;
    let selected = selected_endpoints(&config);

    info!(
        zone = %config.zone,
        endpoints = selected.len(),
        "starting rpc monitor run"
    );

    let mut results = run_checks(&config, selected).await;
    results.sort_by(|a, b| {
        a.endpoint
            .network
            .cmp(&b.endpoint.network)
            .then_with(|| a.endpoint.url.cmp(&b.endpoint.url))
    });
    let script_secs = run_started.elapsed().as_secs_f64();
    let metrics = render_prometheus(&config.zone, timestamp_ms, script_secs, &results);
    log_errors(&config.zone, &results);

    if args.dry_run {
        print!("{metrics}");
    } else {
        write_prometheus_textfile(&config.output_path, &metrics)?;
    }

    drop(lock);
    info!(duration_secs = script_secs, "monitor run complete");
    Ok(())
}

fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("rpc_monitor=info,warn"));

    if let Ok(journald_layer) = tracing_journald::layer() {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(journald_layer)
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }
}

fn normalize_args(args: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    let mut normalized = Vec::new();
    let mut args = args.into_iter();

    if let Some(program) = args.next() {
        normalized.push(program);
    }

    while let Some(arg) = args.next() {
        if arg == "-zone" {
            normalized.push(OsString::from("--zone"));
            if let Some(value) = args.next() {
                normalized.push(value);
            }
            continue;
        }

        if let Some(arg_str) = arg.to_str() {
            if let Some(value) = arg_str.strip_prefix("-zone=") {
                normalized.push(OsString::from(format!("--zone={value}")));
                continue;
            }
        }

        normalized.push(arg);
    }

    normalized
}

fn load_config(path: &Path) -> Result<Config> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read config {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("failed to parse config {}", path.display()))
}

fn validate_config(config: &Config, require_zone: bool) -> Result<()> {
    if require_zone && config.zone.trim().is_empty() {
        bail!("zone must not be empty");
    }
    if config.endpoints.is_empty() {
        bail!("at least one endpoint is required");
    }
    if config.concurrency == 0 {
        bail!("concurrency must be greater than zero");
    }
    for endpoint in &config.endpoints {
        if !endpoint.url.starts_with("ws://") && !endpoint.url.starts_with("wss://") {
            bail!("endpoint {} must start with ws:// or wss://", endpoint.url);
        }
        if endpoint.network.trim().is_empty() || endpoint.zone.trim().is_empty() {
            bail!("endpoint {} has an empty network or zone", endpoint.url);
        }
    }
    Ok(())
}

fn endpoints_for_ip_check(config: &Config, all_zones: bool) -> Vec<Endpoint> {
    let mut endpoints = config
        .endpoints
        .iter()
        .filter(|endpoint| {
            all_zones || config.zone.trim().is_empty() || endpoint.zone == config.zone
        })
        .cloned()
        .collect::<Vec<_>>();
    endpoints.sort_by(|a, b| {
        a.zone
            .cmp(&b.zone)
            .then_with(|| a.network.cmp(&b.network))
            .then_with(|| a.url.cmp(&b.url))
    });
    endpoints
}

async fn run_ip_checks(config: &Config, endpoints: Vec<Endpoint>) -> Result<()> {
    if endpoints.is_empty() {
        bail!("no endpoints matched the selected zone");
    }

    let deadline = Duration::from_secs(config.timeouts.connect_secs);
    println!("endpoint\tzone\tnetwork\tip\tport\tconnect\tssl\twebsocket\tduration_seconds\terror");

    for endpoint in endpoints {
        let parts = endpoint_url_parts(&endpoint.url)?;
        let mut seen = HashSet::new();
        let addresses = lookup_host((parts.host.as_str(), parts.port))
            .await
            .with_context(|| format!("failed to resolve {}", parts.host))?
            .filter(|addr| seen.insert(*addr))
            .collect::<Vec<_>>();

        if addresses.is_empty() {
            print_ip_check_row(IpCheckRow {
                endpoint: &endpoint,
                ip: &parts.host,
                port: parts.port,
                connect: "fail",
                ssl: "-",
                websocket: "-",
                duration_seconds: None,
                error: Some("no DNS records"),
            });
            continue;
        }

        for address in addresses {
            let started = Instant::now();
            match timeout(
                deadline,
                check_endpoint_ip_websocket(&endpoint.url, &parts, address),
            )
            .await
            {
                Ok(Ok(())) => {
                    let ip = address.ip().to_string();
                    print_ip_check_row(IpCheckRow {
                        endpoint: &endpoint,
                        ip: &ip,
                        port: address.port(),
                        connect: "ok",
                        ssl: "ok",
                        websocket: "ok",
                        duration_seconds: Some(started.elapsed().as_secs_f64()),
                        error: None,
                    });
                }
                Ok(Err(err)) => {
                    let ip = address.ip().to_string();
                    let error = err.to_string();
                    let (connect, ssl, websocket) = err.statuses();
                    print_ip_check_row(IpCheckRow {
                        endpoint: &endpoint,
                        ip: &ip,
                        port: address.port(),
                        connect,
                        ssl,
                        websocket,
                        duration_seconds: None,
                        error: Some(&error),
                    });
                }
                Err(_) => {
                    let ip = address.ip().to_string();
                    let error = format!("{}s timeout", deadline.as_secs());
                    print_ip_check_row(IpCheckRow {
                        endpoint: &endpoint,
                        ip: &ip,
                        port: address.port(),
                        connect: "timeout",
                        ssl: "-",
                        websocket: "-",
                        duration_seconds: None,
                        error: Some(&error),
                    });
                }
            }
        }
    }

    Ok(())
}

async fn check_endpoint_ip_websocket(
    url: &str,
    parts: &EndpointUrlParts,
    address: std::net::SocketAddr,
) -> IpCheckResult {
    let stream = TcpStream::connect(address)
        .await
        .map_err(|err| IpCheckError::new(IpCheckStage::Connect, err))?;

    if parts.scheme == "ws" {
        let (mut socket, _) = client_async(url, stream)
            .await
            .map_err(|err| IpCheckError::new(IpCheckStage::Websocket, err))?;
        socket
            .close(None)
            .await
            .map_err(|err| IpCheckError::new(IpCheckStage::Websocket, err))?;
        return Ok(());
    }

    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let tls_config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(tls_config));
    let server_name = rustls::pki_types::ServerName::try_from(parts.host.clone())
        .map_err(|err| IpCheckError::new(IpCheckStage::Ssl, err))?;
    let tls_stream = connector
        .connect(server_name, stream)
        .await
        .map_err(|err| IpCheckError::new(IpCheckStage::Ssl, err))?;
    let (mut socket, _) = client_async(url, tls_stream)
        .await
        .map_err(|err| IpCheckError::new(IpCheckStage::Websocket, err))?;
    socket
        .close(None)
        .await
        .map_err(|err| IpCheckError::new(IpCheckStage::Websocket, err))?;
    Ok(())
}

type IpCheckResult = std::result::Result<(), IpCheckError>;

#[derive(Clone, Copy, Debug)]
enum IpCheckStage {
    Connect,
    Ssl,
    Websocket,
}

#[derive(Debug)]
struct IpCheckError {
    stage: IpCheckStage,
    source: anyhow::Error,
}

impl IpCheckError {
    fn new(stage: IpCheckStage, source: impl Into<anyhow::Error>) -> Self {
        Self {
            stage,
            source: source.into(),
        }
    }

    fn statuses(&self) -> (&'static str, &'static str, &'static str) {
        match self.stage {
            IpCheckStage::Connect => ("fail", "-", "-"),
            IpCheckStage::Ssl => ("ok", "fail", "-"),
            IpCheckStage::Websocket => ("ok", "ok", "fail"),
        }
    }

    fn stage_name(&self) -> &'static str {
        match self.stage {
            IpCheckStage::Connect => "connect",
            IpCheckStage::Ssl => "ssl",
            IpCheckStage::Websocket => "websocket",
        }
    }
}

impl std::fmt::Display for IpCheckError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} failed: {}", self.stage_name(), self.source)
    }
}

impl std::error::Error for IpCheckError {}

struct IpCheckRow<'a> {
    endpoint: &'a Endpoint,
    ip: &'a str,
    port: u16,
    connect: &'a str,
    ssl: &'a str,
    websocket: &'a str,
    duration_seconds: Option<f64>,
    error: Option<&'a str>,
}

fn print_ip_check_row(row: IpCheckRow<'_>) {
    let duration = row
        .duration_seconds
        .map(|duration| format!("{duration:.6}"))
        .unwrap_or_default();
    let error = row.error.unwrap_or_default();
    let line = format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        row.endpoint.url,
        row.endpoint.zone,
        row.endpoint.network,
        row.ip,
        row.port,
        row.connect,
        row.ssl,
        row.websocket,
        duration,
        error
    );

    if row.connect == "ok" && (row.ssl == "ok" || row.ssl == "-") && row.websocket == "ok" {
        println!("{line}");
    } else {
        println!("\x1b[31m{line}\x1b[0m");
    }
}

struct EndpointUrlParts {
    scheme: String,
    host: String,
    port: u16,
}

fn endpoint_url_parts(url: &str) -> Result<EndpointUrlParts> {
    let (scheme, rest, default_port) = if let Some(rest) = url.strip_prefix("wss://") {
        ("wss", rest, 443)
    } else if let Some(rest) = url.strip_prefix("ws://") {
        ("ws", rest, 80)
    } else {
        bail!("endpoint {url} must start with ws:// or wss://");
    };
    let authority = rest.split('/').next().unwrap_or(rest);

    if let Some(after_bracket) = authority.strip_prefix('[') {
        let (host, suffix) = after_bracket
            .split_once(']')
            .ok_or_else(|| anyhow!("invalid IPv6 endpoint authority in {url}"))?;
        let port = if let Some(port) = suffix.strip_prefix(':') {
            port.parse()
                .with_context(|| format!("invalid port in endpoint {url}"))?
        } else {
            default_port
        };
        return Ok(EndpointUrlParts {
            scheme: scheme.to_owned(),
            host: host.to_owned(),
            port,
        });
    }

    if let Some((host, port)) = authority.rsplit_once(':') {
        if !host.contains(':') {
            let port = port
                .parse()
                .with_context(|| format!("invalid port in endpoint {url}"))?;
            return Ok(EndpointUrlParts {
                scheme: scheme.to_owned(),
                host: host.to_owned(),
                port,
            });
        }
    }

    Ok(EndpointUrlParts {
        scheme: scheme.to_owned(),
        host: authority.to_owned(),
        port: default_port,
    })
}

fn acquire_lock(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create lock directory {}", parent.display()))?;
    }

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .with_context(|| format!("failed to open lock file {}", path.display()))?;

    file.try_lock_exclusive()
        .map_err(|_| anyhow!("another rpc-monitor run is already active"))?;
    Ok(file)
}

fn selected_endpoints(config: &Config) -> Vec<Endpoint> {
    let mut endpoints = config
        .endpoints
        .iter()
        .filter(|endpoint| endpoint.zone == config.zone)
        .cloned()
        .collect::<Vec<_>>();
    endpoints.sort_by(|a, b| a.network.cmp(&b.network).then_with(|| a.url.cmp(&b.url)));
    endpoints
}

async fn run_report(config: &Config, args: &ReportCommand) -> Result<()> {
    validate_config(config, false)?;

    if let Some(ReportMode::Short(short_args)) = &args.mode {
        return run_short_report(config, args, short_args).await;
    }

    let period = ReportPeriod::from_args(args.month.as_deref())?;
    let prometheus_url = args
        .prometheus_url
        .as_deref()
        .unwrap_or(&config.report.prometheus_url);
    let client = PrometheusClient::new(prometheus_url)?;
    let mut endpoints = config.endpoints.clone();
    endpoints.sort_by(|a, b| a.network.cmp(&b.network).then_with(|| a.url.cmp(&b.url)));

    println!(
        "## RPC providers report for {} ({} - {})",
        period.label,
        format_report_datetime(period.start),
        format_report_datetime(period.end)
    );
    println!();

    let mean_connect = fetch_global_stat(&client, "rpc_connect", "avg", &period).await?;
    let stddev_connect = fetch_global_stat(&client, "rpc_connect", "stddev", &period).await?;
    let mean_block = fetch_global_stat(&client, "rpc_getblockzero", "avg", &period).await?;
    let stddev_block = fetch_global_stat(&client, "rpc_getblockzero", "stddev", &period).await?;

    println!(
        "- **Connect Time**: Avg = {} s (Std Dev = {} s)",
        format_optional_f64(mean_connect, 2),
        format_optional_f64(stddev_connect, 2)
    );
    println!(
        "- **Block Retrieval Time**: Avg = {} s (Std Dev = {} s)",
        format_optional_f64(mean_block, 2),
        format_optional_f64(stddev_block, 2)
    );
    println!("- **Uptime**: % of time endpoint could fetch block");
    println!("- **Binary Version**: Node version at end of period");
    println!(
        "- **RPC Calls**: RPC calls in the selected {} day period in millions",
        period.days
    );
    println!();

    println!("| Endpoint                        | Zone         | Network            | Average Connect Time (s) | Average Block Retrieval Time (s) | Uptime (%) | Binary Version         | RPC Calls (M) |");
    println!("|---------------------------------|--------------|--------------------|--------------------------|----------------------------------|------------|------------------------|---------------|");

    for endpoint in endpoints {
        let connect_latency = fetch_latency(&client, "rpc_connect", &endpoint, &period).await?;
        let block_latency = fetch_latency(&client, "rpc_getblockzero", &endpoint, &period).await?;
        let uptime = fetch_uptime(&client, &endpoint, &period).await?;
        let binary_version = fetch_version(&client, &endpoint, &period).await?;
        let rpc_calls = if let (Some(chain), Some(job)) = (
            config.report.network_to_chain.get(&endpoint.network),
            endpoint.provider.as_deref(),
        ) {
            fetch_rpc_calls(&client, chain, job, &period).await?
        } else {
            None
        };

        if connect_latency.is_none()
            && block_latency.is_none()
            && uptime.is_none()
            && binary_version.is_none()
            && rpc_calls.is_none()
        {
            continue;
        }

        println!(
            "| {:<33} | {:<12} | {:<20} | {:<24} | {:<32} | {:<10} | {:<22} | {:<13} |",
            report_endpoint_url(&endpoint.url),
            endpoint.zone,
            endpoint.network,
            format_optional_f64(connect_latency, 2),
            format_optional_f64(block_latency, 2),
            format_optional_f64(uptime, 2),
            binary_version.unwrap_or_else(|| "N/A".to_owned()),
            format_optional_f64(rpc_calls, 1)
        );
    }

    Ok(())
}

async fn run_short_report(
    config: &Config,
    args: &ReportCommand,
    short_args: &ShortReportCommand,
) -> Result<()> {
    let period = ReportPeriod::from_short_args(short_args)?;
    let prometheus_url = args
        .prometheus_url
        .as_deref()
        .unwrap_or(&config.report.prometheus_url);
    let client = PrometheusClient::new(prometheus_url)?;
    let mut endpoints = config.endpoints.clone();
    endpoints.sort_by(|a, b| {
        format!("{}|{}", a.network, a.url).cmp(&format!("{}|{}", b.network, b.url))
    });

    let label = short_args.label.as_deref().unwrap_or(&period.label);
    println!("## RPC providers report {label}");
    println!();
    println!("- **RPC Calls**: RPC calls in millions");
    println!();
    println!("| Endpoint | Zone | Network | RPC Calls (M) |");
    println!("|----------|------|---------|---------------|");

    for endpoint in endpoints {
        let rpc_calls = if let (Some(chain), Some(job)) = (
            config.report.network_to_chain.get(&endpoint.network),
            endpoint.provider.as_deref(),
        ) {
            fetch_rpc_calls(&client, chain, job, &period).await?
        } else {
            None
        };

        println!(
            "| {} | {} | {} | {} |",
            report_endpoint_url(&endpoint.url),
            endpoint.zone,
            endpoint.network,
            format_optional_f64(rpc_calls, 1)
        );
    }

    Ok(())
}

fn report_endpoint_url(url: &str) -> String {
    match endpoint_url_parts(url) {
        Ok(parts) if is_default_endpoint_port(&parts) => {
            format!("{}://{}", parts.scheme, parts.host)
        }
        Ok(parts) => format!("{}://{}:{}", parts.scheme, parts.host, parts.port),
        Err(_) => url.to_owned(),
    }
}

fn is_default_endpoint_port(parts: &EndpointUrlParts) -> bool {
    (parts.scheme == "wss" && parts.port == 443) || (parts.scheme == "ws" && parts.port == 80)
}

struct ReportPeriod {
    label: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    days: u32,
    prom_range: String,
}

impl ReportPeriod {
    fn from_args(month: Option<&str>) -> Result<Self> {
        if let Some(month) = month {
            return Self::from_month(month);
        }

        let end = Utc::now();
        let days = 30;
        let start = end - TimeDelta::days(days.into());
        Ok(Self {
            label: "last 30 days".to_owned(),
            start,
            end,
            days,
            prom_range: "30d".to_owned(),
        })
    }

    fn from_month(month: &str) -> Result<Self> {
        let (year, month_number) = parse_month(month)?;
        let days = days_in_month(year, month_number)?;
        let start_date = NaiveDate::from_ymd_opt(year, month_number, 1)
            .ok_or_else(|| anyhow!("invalid month {month}"))?;
        let (next_year, next_month) = if month_number == 12 {
            (year + 1, 1)
        } else {
            (year, month_number + 1)
        };
        let next_date = NaiveDate::from_ymd_opt(next_year, next_month, 1)
            .ok_or_else(|| anyhow!("invalid month {month}"))?;
        let start = DateTime::<Utc>::from_naive_utc_and_offset(
            start_date
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow!("invalid month start for {month}"))?,
            Utc,
        );
        let end = DateTime::<Utc>::from_naive_utc_and_offset(
            next_date
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow!("invalid month end for {month}"))?,
            Utc,
        ) - TimeDelta::seconds(1);

        Ok(Self {
            label: start.format("%B %Y").to_string(),
            start,
            end,
            days,
            prom_range: format!("{days}d"),
        })
    }

    fn from_short_args(args: &ShortReportCommand) -> Result<Self> {
        let end = parse_report_end_boundary(&args.end)?;
        let start = parse_report_boundary(&args.start, "start")?;
        if start >= end {
            bail!("--start must be before --end");
        }
        let prom_range = format!("{}s", (end - start).num_seconds());
        let days = ((end - start).num_seconds() as f64 / 86_400.0).ceil() as u32;

        Ok(Self {
            label: format!(
                "{} to {}",
                format_report_datetime(start),
                format_report_datetime(end)
            ),
            start,
            end,
            days,
            prom_range,
        })
    }

    fn prom_range(&self) -> String {
        self.prom_range.clone()
    }

    fn end_timestamp(&self) -> i64 {
        self.end.timestamp()
    }
}

struct PrometheusClient {
    base_url: String,
    client: reqwest::Client,
}

impl PrometheusClient {
    fn new(base_url: &str) -> Result<Self> {
        let base_url = base_url.trim_end_matches('/').to_owned();
        if base_url.is_empty() {
            bail!("Prometheus URL must not be empty");
        }
        Ok(Self {
            base_url,
            client: reqwest::Client::new(),
        })
    }

    async fn query(&self, query: &str, time: i64) -> Result<Vec<PrometheusSample>> {
        let url = format!("{}/api/v1/query", self.base_url);
        let time = time.to_string();
        let response = self
            .client
            .get(&url)
            .query(&[("query", query), ("time", time.as_str())])
            .send()
            .await
            .with_context(|| format!("failed to query Prometheus at {url}"))?
            .error_for_status()
            .with_context(|| format!("Prometheus query failed at {url}"))?;
        let response = response
            .json::<PrometheusResponse>()
            .await
            .context("failed to parse Prometheus response")?;
        if response.status != "success" {
            bail!("Prometheus returned status {}", response.status);
        }
        Ok(response.data.result)
    }
}

#[derive(Debug, Deserialize)]
struct PrometheusResponse {
    status: String,
    data: PrometheusData,
}

#[derive(Debug, Deserialize)]
struct PrometheusData {
    result: Vec<PrometheusSample>,
}

#[derive(Debug, Deserialize)]
struct PrometheusSample {
    #[serde(default)]
    metric: HashMap<String, String>,
    value: (f64, String),
}

async fn fetch_latency(
    client: &PrometheusClient,
    metric: &str,
    endpoint: &Endpoint,
    period: &ReportPeriod,
) -> Result<Option<f64>> {
    let query = format!(
        "avg(avg_over_time({metric}{{wss=\"{}\",zone=\"{}\",network=\"{}\"}}[{}]))",
        escape_label(&endpoint.url),
        escape_label(&endpoint.zone),
        escape_label(&endpoint.network),
        period.prom_range()
    );
    fetch_first_value(client, &query, period).await
}

async fn fetch_global_stat(
    client: &PrometheusClient,
    metric: &str,
    operation: &str,
    period: &ReportPeriod,
) -> Result<Option<f64>> {
    let query = match operation {
        "avg" => format!("avg(avg_over_time({metric}[{}]))", period.prom_range()),
        "stddev" => format!("avg(stddev_over_time({metric}[{}]))", period.prom_range()),
        _ => bail!("unsupported report operation {operation}"),
    };
    fetch_first_value(client, &query, period).await
}

async fn fetch_uptime(
    client: &PrometheusClient,
    endpoint: &Endpoint,
    period: &ReportPeriod,
) -> Result<Option<f64>> {
    let query = format!(
        "(1 - avg_over_time(rpc_error{{wss=\"{}\",zone=\"{}\",network=\"{}\",error=\"blockzero\"}}[{}])) * 100",
        escape_label(&endpoint.url),
        escape_label(&endpoint.zone),
        escape_label(&endpoint.network),
        period.prom_range()
    );
    fetch_first_value(client, &query, period).await
}

async fn fetch_version(
    client: &PrometheusClient,
    endpoint: &Endpoint,
    period: &ReportPeriod,
) -> Result<Option<String>> {
    let query = format!(
        "last_over_time(rpc_version{{wss=\"{}\",zone=\"{}\",network=\"{}\"}}[20m])",
        escape_label(&endpoint.url),
        escape_label(&endpoint.zone),
        escape_label(&endpoint.network)
    );
    let samples = client.query(&query, period.end_timestamp()).await?;
    Ok(samples.last().map(|sample| {
        sample
            .metric
            .get("version")
            .map(|version| normalize_version(version))
            .unwrap_or_else(|| "N/A".to_owned())
    }))
}

async fn fetch_rpc_calls(
    client: &PrometheusClient,
    chain: &str,
    job: &str,
    period: &ReportPeriod,
) -> Result<Option<f64>> {
    let query = format!(
        "sum(increase(substrate_rpc_calls_started{{chain=\"{}\",job=\"{}\"}}[{}]))",
        escape_label(chain),
        escape_label(job),
        period.prom_range()
    );
    Ok(fetch_first_value(client, &query, period)
        .await?
        .map(|value| value / 1_000_000.0))
}

async fn fetch_first_value(
    client: &PrometheusClient,
    query: &str,
    period: &ReportPeriod,
) -> Result<Option<f64>> {
    let samples = client.query(query, period.end_timestamp()).await?;
    Ok(samples
        .first()
        .and_then(|sample| sample.value.1.parse::<f64>().ok()))
}

fn parse_month(value: &str) -> Result<(i32, u32)> {
    let (year, month) = value
        .split_once('-')
        .ok_or_else(|| anyhow!("invalid date format. Use YYYY-MM, for example 2026-05"))?;
    if year.len() != 4 || month.len() != 2 {
        bail!("invalid date format. Use YYYY-MM, for example 2026-05");
    }
    let year = year
        .parse::<i32>()
        .with_context(|| format!("invalid year in {value}"))?;
    let month = month
        .parse::<u32>()
        .with_context(|| format!("invalid month in {value}"))?;
    if !(1..=12).contains(&month) {
        bail!("month must be between 01 and 12");
    }
    Ok((year, month))
}

fn parse_report_boundary(value: &str, name: &str) -> Result<DateTime<Utc>> {
    if let Ok(timestamp) = DateTime::parse_from_rfc3339(value) {
        return Ok(timestamp.with_timezone(&Utc));
    }
    let date = ["%d-%m-%Y", "%Y-%m-%d"]
        .into_iter()
        .find_map(|format| NaiveDate::parse_from_str(value, format).ok())
        .ok_or_else(|| {
            anyhow!("invalid {name} boundary {value:?}; use DD-MM-YYYY, YYYY-MM-DD or RFC3339")
        })?;
    Ok(DateTime::<Utc>::from_naive_utc_and_offset(
        date.and_hms_opt(0, 0, 0)
            .ok_or_else(|| anyhow!("invalid {name} boundary {value:?}"))?,
        Utc,
    ))
}

fn parse_report_end_boundary(value: &str) -> Result<DateTime<Utc>> {
    if let Ok(timestamp) = DateTime::parse_from_rfc3339(value) {
        return Ok(timestamp.with_timezone(&Utc));
    }
    Ok(parse_report_boundary(value, "end")? + TimeDelta::days(1))
}

fn days_in_month(year: i32, month: u32) -> Result<u32> {
    let start = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| anyhow!("invalid month {year:04}-{month:02}"))?;
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let next = NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .ok_or_else(|| anyhow!("invalid month {year:04}-{month:02}"))?;
    Ok((next - start).num_days() as u32)
}

fn format_report_datetime(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

fn format_optional_f64(value: Option<f64>, precision: usize) -> String {
    match value {
        Some(value) => format!("{value:.precision$}"),
        None => "N/A".to_owned(),
    }
}

fn normalize_version(version: &str) -> String {
    if let Some((plain, suffix)) = version.split_once('-') {
        let numeric = plain
            .split('.')
            .all(|part| !part.is_empty() && part.chars().all(|char| char.is_ascii_digit()));
        let hex_suffix = !suffix.is_empty() && suffix.chars().all(|char| char.is_ascii_hexdigit());
        if numeric && hex_suffix {
            return plain.to_owned();
        }
    }
    "Invalid".to_owned()
}

async fn run_checks(config: &Config, endpoints: Vec<Endpoint>) -> Vec<EndpointResult> {
    let zero_hashes = config.zero_hashes.clone();
    let timeouts = Timeouts {
        connect_secs: config.timeouts.connect_secs,
        block_zero_secs: config.timeouts.block_zero_secs,
        get_block_secs: config.timeouts.get_block_secs,
        block_height_secs: config.timeouts.block_height_secs,
        version_secs: config.timeouts.version_secs,
    };

    futures_util::stream::iter(endpoints.into_iter().map(|endpoint| {
        let zero_hashes = zero_hashes.clone();
        let timeouts = Timeouts {
            connect_secs: timeouts.connect_secs,
            block_zero_secs: timeouts.block_zero_secs,
            get_block_secs: timeouts.get_block_secs,
            block_height_secs: timeouts.block_height_secs,
            version_secs: timeouts.version_secs,
        };
        async move { check_endpoint(endpoint, zero_hashes, timeouts).await }
    }))
    .buffer_unordered(config.concurrency)
    .collect()
    .await
}

async fn check_endpoint(
    endpoint: Endpoint,
    zero_hashes: HashMap<String, String>,
    timeouts: Timeouts,
) -> EndpointResult {
    let mut errors = Vec::new();
    let mut skipped = Vec::new();

    let connect_secs =
        match timed_connect(&endpoint.url, Duration::from_secs(timeouts.connect_secs)).await {
            Ok(secs) => Some(secs),
            Err(err) => {
                errors.push(CheckError {
                    kind: "connect",
                    message: err.to_string(),
                });
                None
            }
        };

    let block_zero_secs = match zero_hashes.get(&endpoint.network) {
        Some(hash) => match timed_rpc_call(
            &endpoint.url,
            "chain_getBlock",
            json!([hash]),
            Duration::from_secs(timeouts.block_zero_secs),
        )
        .await
        {
            Ok((secs, value)) if is_block_zero_response(&value) => Some(secs),
            Ok((_, value)) => {
                errors.push(CheckError {
                    kind: "blockzero",
                    message: format!("unexpected block response: {value}"),
                });
                None
            }
            Err(err) => {
                errors.push(CheckError {
                    kind: "blockzero",
                    message: err.to_string(),
                });
                None
            }
        },
        // Some networks, such as bulletin chains, are not expected to be archive
        // endpoints. If no zero hash is configured, skip this check and emit
        // rpc_error{error="blockzero"} = 0 so dashboards treat it as healthy.
        None => {
            skipped.push("blockzero");
            None
        }
    };

    let get_block_secs = match timed_rpc_call(
        &endpoint.url,
        "chain_getBlock",
        json!([]),
        Duration::from_secs(timeouts.get_block_secs),
    )
    .await
    {
        Ok((secs, value)) if block_response_height(&value).is_some() => Some(secs),
        Ok((_, value)) => {
            errors.push(CheckError {
                kind: "getblock",
                message: format!("unexpected latest block response: {value}"),
            });
            None
        }
        Err(err) => {
            errors.push(CheckError {
                kind: "getblock",
                message: err.to_string(),
            });
            None
        }
    };

    let block_height = match timed_rpc_call(
        &endpoint.url,
        "chain_getHeader",
        json!([]),
        Duration::from_secs(timeouts.block_height_secs),
    )
    .await
    {
        Ok((_, value)) => match latest_block_height(&value) {
            Some(height) => Some(height),
            None => {
                errors.push(CheckError {
                    kind: "height",
                    message: format!("unexpected latest header response: {value}"),
                });
                None
            }
        },
        Err(err) => {
            errors.push(CheckError {
                kind: "height",
                message: err.to_string(),
            });
            None
        }
    };

    let version = match timed_rpc_call(
        &endpoint.url,
        "system_version",
        json!([]),
        Duration::from_secs(timeouts.version_secs),
    )
    .await
    {
        Ok((_, value)) => match value.as_str() {
            Some(version) if !version.trim().is_empty() => Some(version.to_owned()),
            _ => {
                errors.push(CheckError {
                    kind: "version",
                    message: format!("unexpected version response: {value}"),
                });
                None
            }
        },
        Err(err) => {
            errors.push(CheckError {
                kind: "version",
                message: err.to_string(),
            });
            None
        }
    };

    EndpointResult {
        endpoint,
        block_zero_secs,
        get_block_secs,
        block_height,
        connect_secs,
        version,
        errors,
        skipped,
    }
}

async fn timed_connect(url: &str, deadline: Duration) -> Result<f64> {
    let started = Instant::now();
    let connect = timeout(deadline, connect_async(url))
        .await
        .with_context(|| format!("timeout after {}s", deadline.as_secs()))?;
    let (mut socket, _) = connect.with_context(|| format!("websocket connect failed for {url}"))?;
    let elapsed = started.elapsed().as_secs_f64();
    let _ = socket.close(None).await;
    Ok(elapsed)
}

async fn timed_rpc_call(
    url: &str,
    method: &str,
    params: Value,
    deadline: Duration,
) -> Result<(f64, Value)> {
    let started = Instant::now();
    let response = timeout(deadline, rpc_call(url, method, params))
        .await
        .with_context(|| format!("{method} timeout after {}s", deadline.as_secs()))??;
    Ok((started.elapsed().as_secs_f64(), response))
}

async fn rpc_call(url: &str, method: &str, params: Value) -> Result<Value> {
    let (mut socket, _) = connect_async(url)
        .await
        .with_context(|| format!("websocket connect failed for {url}"))?;

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });
    socket
        .send(Message::Text(request.to_string().into()))
        .await
        .with_context(|| format!("failed to send {method} request"))?;

    while let Some(message) = socket.next().await {
        let message = message.with_context(|| format!("failed reading {method} response"))?;
        match message {
            Message::Text(text) => {
                let value: Value = serde_json::from_str(&text)
                    .with_context(|| format!("invalid JSON-RPC response for {method}: {text}"))?;
                if value.get("id") != Some(&json!(1)) {
                    continue;
                }
                if let Some(error) = value.get("error") {
                    bail!("{method} returned JSON-RPC error: {error}");
                }
                return value
                    .get("result")
                    .cloned()
                    .ok_or_else(|| anyhow!("{method} response did not include result"));
            }
            Message::Close(frame) => bail!("{method} connection closed before response: {frame:?}"),
            _ => {}
        }
    }

    bail!("{method} connection closed before response")
}

fn render_prometheus(
    zone: &str,
    timestamp_ms: u128,
    script_secs: f64,
    results: &[EndpointResult],
) -> String {
    let mut out = String::new();
    out.push_str("# HELP rpc_getblockzero time to get block 0\n");
    out.push_str("# TYPE rpc_getblockzero gauge\n");
    for result in results {
        if let Some(value) = result.block_zero_secs {
            push_metric(
                &mut out,
                "rpc_getblockzero",
                &result.endpoint,
                zone,
                &[],
                value,
                timestamp_ms,
            );
        }
    }

    out.push('\n');
    out.push_str("# HELP rpc_getblock time to get latest block\n");
    out.push_str("# TYPE rpc_getblock gauge\n");
    for result in results {
        if let Some(value) = result.get_block_secs {
            push_metric(
                &mut out,
                "rpc_getblock",
                &result.endpoint,
                zone,
                &[],
                value,
                timestamp_ms,
            );
        }
    }

    out.push('\n');
    out.push_str("# HELP rpc_connect time to connect\n");
    out.push_str("# TYPE rpc_connect gauge\n");
    for result in results {
        if let Some(value) = result.connect_secs {
            push_metric(
                &mut out,
                "rpc_connect",
                &result.endpoint,
                zone,
                &[],
                value,
                timestamp_ms,
            );
        }
    }

    out.push('\n');
    out.push_str("# HELP rpc_blockheight latest observed chain block height\n");
    out.push_str("# TYPE rpc_blockheight gauge\n");
    for result in results {
        if let Some(height) = result.block_height {
            push_metric(
                &mut out,
                "rpc_blockheight",
                &result.endpoint,
                zone,
                &[],
                height as f64,
                timestamp_ms,
            );
        }
    }

    out.push('\n');
    out.push_str("# HELP rpc_blockdrift difference between observed endpoint height and highest observed network height\n");
    out.push_str("# TYPE rpc_blockdrift gauge\n");
    let network_heights = max_block_heights_by_network(results);
    for result in results {
        if let Some(height) = result.block_height {
            if let Some(max_height) = network_heights.get(&result.endpoint.network) {
                push_metric(
                    &mut out,
                    "rpc_blockdrift",
                    &result.endpoint,
                    zone,
                    &[],
                    max_height.saturating_sub(height) as f64,
                    timestamp_ms,
                );
            }
        }
    }

    out.push('\n');
    out.push_str("# HELP rpc_version binary version\n");
    out.push_str("# TYPE rpc_version gauge\n");
    for result in results {
        if let Some(version) = &result.version {
            push_metric(
                &mut out,
                "rpc_version",
                &result.endpoint,
                zone,
                &[("version", version)],
                1.0,
                timestamp_ms,
            );
        }
    }

    out.push('\n');
    out.push_str("# HELP rpc_error rpc-error\n");
    out.push_str("# TYPE rpc_error gauge\n");
    for result in results {
        for error_kind in [
            "blockzero",
            "connect",
            "code",
            "getblock",
            "height",
            "version",
        ] {
            let value = if result.errors.iter().any(|error| error.kind == error_kind) {
                1.0
            } else if result.skipped.contains(&error_kind) {
                0.0
            } else {
                0.0
            };
            push_metric(
                &mut out,
                "rpc_error",
                &result.endpoint,
                zone,
                &[("error", error_kind)],
                value,
                timestamp_ms,
            );
        }
    }

    out.push('\n');
    out.push_str("# HELP rpc_script Script run duration\n");
    out.push_str("# TYPE rpc_script gauge\n");
    out.push_str(&format!(
        "rpc_script{{zone=\"{}\"}} {:.2} {}\n",
        escape_label(zone),
        script_secs,
        timestamp_ms
    ));
    out
}

fn max_block_heights_by_network(results: &[EndpointResult]) -> HashMap<String, u64> {
    let mut heights = HashMap::new();
    for result in results {
        if let Some(height) = result.block_height {
            heights
                .entry(result.endpoint.network.clone())
                .and_modify(|max_height| *max_height = height.max(*max_height))
                .or_insert(height);
        }
    }
    heights
}

fn push_metric(
    out: &mut String,
    name: &str,
    endpoint: &Endpoint,
    zone: &str,
    extra_labels: &[(&str, &str)],
    value: f64,
    timestamp_ms: u128,
) {
    out.push_str(name);
    out.push('{');
    out.push_str(&format!(
        "wss=\"{}\",network=\"{}\",zone=\"{}\"",
        escape_label(&endpoint.url),
        escape_label(&endpoint.network),
        escape_label(zone)
    ));
    for (key, label_value) in extra_labels {
        out.push_str(&format!(",{}=\"{}\"", key, escape_label(label_value)));
    }
    out.push_str(&format!("}} {:.6} {}\n", value, timestamp_ms));
}

fn escape_label(value: &str) -> String {
    value
        .replace('\\', r"\\")
        .replace('\n', r"\n")
        .replace('"', r#"\""#)
}

fn is_block_zero_response(value: &Value) -> bool {
    block_response_height(value).is_some_and(|height| height == 0)
}

fn block_response_height(value: &Value) -> Option<u64> {
    value
        .pointer("/block/header/number")
        .and_then(Value::as_str)
        .and_then(parse_hex_u64)
}

fn latest_block_height(value: &Value) -> Option<u64> {
    value
        .pointer("/number")
        .and_then(Value::as_str)
        .and_then(parse_hex_u64)
}

fn parse_hex_u64(value: &str) -> Option<u64> {
    u64::from_str_radix(value.strip_prefix("0x").unwrap_or(value), 16).ok()
}

fn log_errors(zone: &str, results: &[EndpointResult]) {
    for result in results {
        for err in &result.errors {
            error!(
                wss = %result.endpoint.url,
                network = %result.endpoint.network,
                zone = %zone,
                error = %err.kind,
                reason = %err.message,
                "rpc monitor check failed: check={} wss={} network={} zone={} reason={}",
                err.kind,
                result.endpoint.url,
                result.endpoint.network,
                zone,
                err.message
            );
        }
    }
}

fn write_prometheus_textfile(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }

    let tmp = path.with_extension("txt.new");
    fs::write(&tmp, contents).with_context(|| format!("failed to write {}", tmp.display()))?;
    let file = File::open(&tmp).with_context(|| format!("failed to reopen {}", tmp.display()))?;
    file.sync_all()
        .with_context(|| format!("failed to sync {}", tmp.display()))?;

    match fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(err) if path.exists() => {
            warn!(
                output = %path.display(),
                error = %err,
                "atomic replace failed; retrying with remove-and-rename fallback"
            );
            fs::remove_file(path)
                .with_context(|| format!("failed to remove old output {}", path.display()))?;
            fs::rename(&tmp, path)
                .with_context(|| format!("failed to move {} to {}", tmp.display(), path.display()))
        }
        Err(err) => Err(err)
            .with_context(|| format!("failed to move {} to {}", tmp.display(), path.display())),
    }
}

fn unix_timestamp_millis() -> Result<u128> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before UNIX_EPOCH")?
        .as_millis())
}

fn default_concurrency() -> usize {
    8
}

fn default_prometheus_url() -> String {
    "http://localhost:9090".to_owned()
}

fn default_connect_secs() -> u64 {
    5
}

fn default_block_zero_secs() -> u64 {
    20
}

fn default_get_block_secs() -> u64 {
    10
}

fn default_block_height_secs() -> u64 {
    10
}

fn default_version_secs() -> u64 {
    10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_prometheus_label_values() {
        assert_eq!(
            escape_label(
                r#"a\b"c
d"#
            ),
            r#"a\\b\"c\nd"#
        );
    }

    #[test]
    fn filters_and_sorts_endpoints_by_zone() {
        let config = Config {
            zone: "eu-central".to_owned(),
            output_path: PathBuf::from("index.txt"),
            lock_path: PathBuf::from("monitor.lock"),
            concurrency: 1,
            timeouts: Timeouts::default(),
            zero_hashes: HashMap::new(),
            report: ReportConfig::default(),
            endpoints: vec![
                Endpoint {
                    url: "wss://z.example".to_owned(),
                    network: "polkadot".to_owned(),
                    zone: "us-east".to_owned(),
                    provider: None,
                },
                Endpoint {
                    url: "wss://b.example".to_owned(),
                    network: "kusama".to_owned(),
                    zone: "eu-central".to_owned(),
                    provider: None,
                },
                Endpoint {
                    url: "wss://a.example".to_owned(),
                    network: "polkadot".to_owned(),
                    zone: "eu-central".to_owned(),
                    provider: None,
                },
            ],
        };

        let endpoints = selected_endpoints(&config);
        assert_eq!(endpoints.len(), 2);
        assert_eq!(endpoints[0].url, "wss://b.example");
        assert_eq!(endpoints[1].url, "wss://a.example");
    }

    #[test]
    fn validates_block_zero_response() {
        let response = json!({
            "block": {
                "header": {
                    "number": "0x0"
                }
            }
        });
        assert!(is_block_zero_response(&response));

        let wrong_block = json!({
            "block": {
                "header": {
                    "number": "0x1"
                }
            }
        });
        assert!(!is_block_zero_response(&wrong_block));
    }

    #[test]
    fn parses_block_response_height() {
        let response = json!({
            "block": {
                "header": {
                    "number": "0x2a"
                }
            }
        });

        assert_eq!(block_response_height(&response), Some(42));
        assert_eq!(
            block_response_height(&json!({"block": {"header": {}}})),
            None
        );
    }

    #[test]
    fn parses_latest_block_height() {
        let response = json!({
            "parentHash": "0xabc",
            "number": "0x1a",
            "stateRoot": "0xdef"
        });

        assert_eq!(latest_block_height(&response), Some(26));
        assert_eq!(latest_block_height(&json!({"number": "not-hex"})), None);
        assert_eq!(latest_block_height(&json!({})), None);
    }

    #[test]
    fn computes_max_block_height_by_network() {
        let results = vec![
            EndpointResult {
                endpoint: Endpoint {
                    url: "wss://a.example".to_owned(),
                    network: "polkadot".to_owned(),
                    zone: "eu-central".to_owned(),
                    provider: None,
                },
                block_zero_secs: None,
                get_block_secs: None,
                block_height: Some(10),
                connect_secs: None,
                version: None,
                errors: Vec::new(),
                skipped: Vec::new(),
            },
            EndpointResult {
                endpoint: Endpoint {
                    url: "wss://b.example".to_owned(),
                    network: "polkadot".to_owned(),
                    zone: "eu-central".to_owned(),
                    provider: None,
                },
                block_zero_secs: None,
                get_block_secs: None,
                block_height: Some(15),
                connect_secs: None,
                version: None,
                errors: Vec::new(),
                skipped: Vec::new(),
            },
            EndpointResult {
                endpoint: Endpoint {
                    url: "wss://c.example".to_owned(),
                    network: "kusama".to_owned(),
                    zone: "eu-central".to_owned(),
                    provider: None,
                },
                block_zero_secs: None,
                get_block_secs: None,
                block_height: Some(7),
                connect_secs: None,
                version: None,
                errors: Vec::new(),
                skipped: Vec::new(),
            },
        ];

        let heights = max_block_heights_by_network(&results);
        assert_eq!(heights.get("polkadot"), Some(&15));
        assert_eq!(heights.get("kusama"), Some(&7));
    }

    #[tokio::test]
    async fn missing_zero_hash_does_not_create_blockzero_error() {
        let endpoint = Endpoint {
            url: "wss://example.invalid".to_owned(),
            network: "polkadot-bulletin".to_owned(),
            zone: "eu-central".to_owned(),
            provider: None,
        };
        let result = check_endpoint(
            endpoint,
            HashMap::new(),
            Timeouts {
                connect_secs: 0,
                block_zero_secs: 0,
                get_block_secs: 0,
                block_height_secs: 0,
                version_secs: 0,
            },
        )
        .await;

        assert!(!result.errors.iter().any(|err| err.kind == "blockzero"));
        assert!(result.skipped.contains(&"blockzero"));
    }

    #[test]
    fn skipped_blockzero_renders_as_zero_error_value() {
        let result = EndpointResult {
            endpoint: Endpoint {
                url: "wss://bulletin.example".to_owned(),
                network: "polkadot-bulletin".to_owned(),
                zone: "us-east".to_owned(),
                provider: None,
            },
            block_zero_secs: None,
            get_block_secs: None,
            block_height: None,
            connect_secs: None,
            version: None,
            errors: Vec::new(),
            skipped: vec!["blockzero"],
        };

        let metrics = render_prometheus("us-east", 123, 1.0, &[result]);
        assert!(metrics.contains(
            "rpc_error{wss=\"wss://bulletin.example\",network=\"polkadot-bulletin\",zone=\"us-east\",error=\"blockzero\"} 0.000000 123"
        ));
    }

    #[test]
    fn normalizes_single_dash_zone_alias() {
        let args = normalize_args([
            OsString::from("rpc-monitor"),
            OsString::from("-zone=eu-central"),
            OsString::from("--dry-run"),
        ]);
        assert_eq!(args[1], "--zone=eu-central");
        assert_eq!(args[2], "--dry-run");

        let args = normalize_args([
            OsString::from("rpc-monitor"),
            OsString::from("-zone"),
            OsString::from("us-east"),
        ]);
        assert_eq!(args[1], "--zone");
        assert_eq!(args[2], "us-east");
    }

    #[test]
    fn computes_month_lengths_for_reports() {
        assert_eq!(days_in_month(2026, 1).unwrap(), 31);
        assert_eq!(days_in_month(2026, 4).unwrap(), 30);
        assert_eq!(days_in_month(2024, 2).unwrap(), 29);
        assert_eq!(days_in_month(2026, 2).unwrap(), 28);
    }

    #[test]
    fn parses_report_month_period() {
        let period = ReportPeriod::from_month("2026-02").unwrap();
        assert_eq!(period.days, 28);
        assert_eq!(
            format_report_datetime(period.start),
            "2026-02-01 00:00:00 UTC"
        );
        assert_eq!(
            format_report_datetime(period.end),
            "2026-02-28 23:59:59 UTC"
        );
        assert_eq!(period.prom_range(), "28d");
    }

    #[test]
    fn parses_short_report_period_from_dutch_date_boundaries() {
        let args = ShortReportCommand {
            start: "01-02-2026".to_owned(),
            end: "20-06-2026".to_owned(),
            label: None,
        };
        let period = ReportPeriod::from_short_args(&args).unwrap();

        assert_eq!(period.prom_range(), "12096000s");
        assert_eq!(
            format_report_datetime(period.start),
            "2026-02-01 00:00:00 UTC"
        );
        assert_eq!(
            format_report_datetime(period.end),
            "2026-06-21 00:00:00 UTC"
        );
    }

    #[test]
    fn parses_short_report_period_from_boundaries() {
        let args = ShortReportCommand {
            start: "2026-02-01".to_owned(),
            end: "2026-02-01".to_owned(),
            label: None,
        };
        let period = ReportPeriod::from_short_args(&args).unwrap();

        assert_eq!(period.prom_range(), "86400s");
        assert_eq!(period.days, 1);
    }

    #[test]
    fn rejects_invalid_short_report_boundaries() {
        let args = ShortReportCommand {
            start: "20-06-2026".to_owned(),
            end: "01-02-2026".to_owned(),
            label: None,
        };
        assert!(ReportPeriod::from_short_args(&args).is_err());
        assert!(parse_report_boundary("2026/02/01", "start").is_err());
    }

    #[test]
    fn normalizes_report_versions() {
        assert_eq!(normalize_version("1.16.3-abcdef12"), "1.16.3");
        assert_eq!(normalize_version("1.16.3"), "Invalid");
        assert_eq!(normalize_version("parity-polkadot"), "Invalid");
    }

    #[test]
    fn strips_report_endpoint_url_to_scheme_and_host() {
        assert_eq!(
            report_endpoint_url(
                "wss://spectrum-03.simplystaking.xyz/cG9sa2Fkb3QtMDMtOTFkMmYwZGYtcG9sa2Fkb3Q/LjwBJpV3dIKyWQ/polkadot/mainnet/"
            ),
            "wss://spectrum-03.simplystaking.xyz"
        );
        assert_eq!(
            report_endpoint_url("ws://example.com:9944/path"),
            "ws://example.com:9944"
        );
    }

    #[test]
    fn parses_endpoint_host_and_port() {
        let endpoint = endpoint_url_parts("wss://example.com/path").unwrap();
        assert_eq!(endpoint.scheme, "wss");
        assert_eq!(endpoint.host, "example.com");
        assert_eq!(endpoint.port, 443);

        let endpoint = endpoint_url_parts("ws://example.com:9944").unwrap();
        assert_eq!(endpoint.scheme, "ws");
        assert_eq!(endpoint.host, "example.com");
        assert_eq!(endpoint.port, 9944);

        let endpoint = endpoint_url_parts("wss://[2001:db8::1]:443/path").unwrap();
        assert_eq!(endpoint.scheme, "wss");
        assert_eq!(endpoint.host, "2001:db8::1");
        assert_eq!(endpoint.port, 443);
    }
}
