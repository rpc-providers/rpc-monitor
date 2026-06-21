# RPC Monitor

RPC Monitor checks Substrate websocket RPC endpoints from one or more
geographical monitoring zones and writes Prometheus textfile metrics. The
monitor is intended to run from systemd timer units on regional monitor nodes.

Public dashboard:

- [RPC Providers Grafana dashboard](https://monitor.rpc-providers.net/)

Public metric endpoints:

- [mon-eu-central.rpc-providers.net](http://mon-eu-central.rpc-providers.net/)
- [mon-us-east.rpc-providers.net](http://mon-us-east.rpc-providers.net/)

## What It Measures

Each configured endpoint is checked over its websocket JSON-RPC interface.

- `rpc_connect`: DNS, TCP, TLS and websocket handshake duration.
- `rpc_getblockzero`: duration of `chain_getBlock` for the configured genesis block hash.
- `rpc_getblock`: duration of `chain_getBlock` for the latest block.
- `rpc_blockheight`: latest observed block height from `chain_getHeader`.
- `rpc_blockdrift`: difference between this endpoint's observed height and the highest observed height for the same network during the same run.
- `rpc_version`: node version from `system_version`, exposed as a label.
- `rpc_error`: per-check error state.
- `rpc_script`: total monitor run duration.

`rpc_error` values:

- `0`: OK
- `1`: error

`rpc_getblockzero` is only measured when the endpoint network has a matching
entry in `[zero_hashes]`. If no zero hash is configured, the block-zero check is
treated as OK in `rpc_error` instead of erroring. This is useful for
non-archive endpoints or chains where genesis block retrieval is not required.

## Repository Layout

- `src/main.rs`: monitor application.
- `monitor.toml`: monitor configuration.
- `deploy/rpc-monitor.service`: systemd oneshot service.
- `deploy/rpc-monitor.timer`: systemd timer, default 15 minute interval.
- `Cargo.toml`: Rust package definition.

Recommended production paths:

- Application directory: `/opt/rpc-monitor`
- Binary: `/usr/local/bin/rpc-monitor
- Config: `/opt/rpc-monitor/monitor.toml`
- Prometheus textfile output: `/var/www/prom/index.txt`
- Lock file: `/tmp/rpc-monitor.lock`
- Logs: `journalctl -u rpc-monitor.service`

## Build

Install a stable Rust toolchain, then build the release binary:

```bash
cd /opt/rpc-monitor
cargo build --release
```

The release binary will be written to:

```text
/opt/rpc-monitor/target/release/rpc-monitor
```

Check the installed build version:

```bash
/opt/rpc-monitor/target/release/rpc-monitor --version
```

Releases use [Semantic Versioning](https://semver.org/). The initial stable
release is `v1.0.0`; use `vMAJOR.MINOR.PATCH` Git tags for later releases.

## Configuration

Configure the monitor directly in `monitor.toml`.

Top-level settings:

```toml
output_path = "/var/www/prom/index.txt"
lock_path = "/tmp/rpc-monitor.lock"
concurrency = 8
```

Timeouts:

```toml
[timeouts]
connect_secs = 5
block_zero_secs = 20
get_block_secs = 10
block_height_secs = 10
version_secs = 10
```

Report settings:

```toml
[report]
prometheus_url = "http://localhost:9090"

[report.network_to_chain]
polkadot = "polkadot"
kusama = "ksmcc3"
polkadot-assethub = "asset-hub-polkadot"
kusama-assethub = "asset-hub-kusama"
```

`report.network_to_chain` maps the monitor's logical `network` labels to the
`chain` labels used by `substrate_rpc_calls_started` in Prometheus. Endpoint
`provider` values are used as the Prometheus `job` label for RPC call totals.

Genesis block hashes used for `rpc_getblockzero`:

```toml
[zero_hashes]
polkadot = "0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3"
kusama = "0xb0a8d493285c2df73290dfb7e61f870f17b41801197a149ca93654499ea3dafe"
```

Endpoint entries:

```toml
[[endpoints]]
url = "wss://polkadot-rpc.n.dwellir.com"
network = "polkadot"
zone = "eu-central"
provider = "Dwellir"
```

Field meanings:

- `url`: websocket endpoint to check.
- `network`: logical network name used in Prometheus labels and drift grouping.
- `zone`: monitor zone this endpoint belongs to.
- `provider`: optional provider label for humans and future tooling.

The same `monitor.toml` can be deployed to all monitor nodes. Select the local
zone at startup with `--zone` or `RPC_MONITOR_ZONE`.

## Running Manually

Run a normal check for one zone:

```bash
/opt/rpc-monitor/target/release/rpc-monitor \
  --config /opt/rpc-monitor/monitor.toml \
  --zone eu-central
```

Dry run, printing Prometheus output to stdout:

```bash
/opt/rpc-monitor/target/release/rpc-monitor \
  --config /opt/rpc-monitor/monitor.toml \
  --zone us-east \
  --dry-run
```

Override the output path:

```bash
/opt/rpc-monitor/target/release/rpc-monitor \
  --config /opt/rpc-monitor/monitor.toml \
  --zone eu-central \
  --output /tmp/rpc-monitor-index.txt
```

Environment variables are also supported:

```bash
RPC_MONITOR_CONFIG=/opt/rpc-monitor/monitor.toml \
RPC_MONITOR_ZONE=eu-central \
RPC_MONITOR_OUTPUT=/var/www/prom/index.txt \
/opt/rpc-monitor/target/release/rpc-monitor
```

## Reports

Generate a Markdown report for the last 30 days:

```bash
/opt/rpc-monitor/target/release/rpc-monitor \
  --config /opt/rpc-monitor/monitor.toml \
  report
```

Generate a Markdown report for a calendar month:

```bash
/opt/rpc-monitor/target/release/rpc-monitor \
  --config /opt/rpc-monitor/monitor.toml \
report --month 2026-05
```

Generate the compact tender-style report for an arbitrary date range:

```bash
/opt/rpc-monitor/target/release/rpc-monitor \
  --config /opt/rpc-monitor/monitor.toml \
  report short --start 01-02-2026 --end 20-06-2026
```

`report short` contains only the endpoint, zone, network and RPC-call total.
It accepts `DD-MM-YYYY`, `YYYY-MM-DD`, or RFC3339 timestamps. Date-only
values are interpreted in UTC: `start` begins at midnight and `end` includes
the entire given calendar day. RFC3339 values are used as exact boundaries.
Endpoint paths and query tokens are omitted from the table; for example,
`wss://spectrum-03.simplystaking.xyz/...` is shown as
`wss://spectrum-03.simplystaking.xyz`.

For example, `--start 01-02-2026 --end 20-06-2026` queries the inclusive
period from `2026-02-01T00:00:00Z` through `2026-06-21T00:00:00Z`.

Month reports use the actual calendar month length for Prometheus ranges, so
February uses 28 or 29 days, 30-day months use 30 days and 31-day months use 31
days. Uptime is calculated from `avg_over_time(rpc_error{error="blockzero"})`
rather than a fixed sample count. Endpoints with no report data in the selected
period are omitted, which keeps historical reports from showing newly added
endpoints as all-`N/A` rows. The report table shows compact endpoint URLs as
`scheme://host`; the full configured URL is still used for Prometheus queries.

## IP Checks

Use `--check-ips` to resolve each endpoint and test every individual IP address.
For each IP it opens the TCP connection, performs TLS validation for the
original endpoint hostname, and completes the websocket handshake:

```bash
/opt/rpc-monitor/target/release/rpc-monitor \
  --config /opt/rpc-monitor/monitor.toml \
  --zone eu-central \
  --check-ips
```

Check all zones:

```bash
/opt/rpc-monitor/target/release/rpc-monitor \
  --config /opt/rpc-monitor/monitor.toml \
  --check-ips \
  --all-zones
```

Output columns:

```text
endpoint    zone    network    ip    port    connect    ssl    websocket    duration_seconds    error
```

For `wss://` endpoints all three check columns should be `ok`. For `ws://`
endpoints, `ssl` is `-` because TLS is not used.

Example:

```text
wss://ksm-rpc.stakeworld.io    eu-central    kusama    2606:4700:20::681a:642    443    ok    ok    ok    0.168568
```

If a row fails, the remaining checks are marked `-` and the row is printed in
red in an interactive terminal.

This is meant for diagnosing DNS, IPv4/IPv6, routing, TLS/SNI and websocket
upgrade issues per individual resolved address.

## systemd Timer

Install the service and timer:

```bash
cd /opt/rpc-monitor
cp deploy/rpc-monitor.service /etc/systemd/system/
cp deploy/rpc-monitor.timer /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now rpc-monitor.timer
```

The service is a oneshot unit. The timer runs it every 15 minutes:

```ini
[Timer]
OnBootSec=2min
OnUnitActiveSec=15min
AccuracySec=30s
Persistent=true
```

Set the zone for a node in `/etc/systemd/system/rpc-monitor.service`:

```ini
Environment=RPC_MONITOR_ZONE=eu-central
ExecStart=/opt/rpc-monitor/target/release/rpc-monitor --config /opt/rpc-monitor/monitor.toml --zone ${RPC_MONITOR_ZONE}
```

After changing the service file:

```bash
systemctl daemon-reload
systemctl restart rpc-monitor.timer
```

Useful commands:

```bash
systemctl list-timers rpc-monitor.timer
systemctl start rpc-monitor.service
systemctl status rpc-monitor.service
journalctl -u rpc-monitor.service -n 100 --no-pager
journalctl -u rpc-monitor.service -p err -n 100 --no-pager
```

## Logging

Runtime logs go to journald when started through systemd.

Example log lines:

```text
INFO rpc_monitor: starting rpc monitor run zone=eu-central endpoints=13
ERROR rpc_monitor: rpc monitor check failed: check=connect wss=wss://rpc-kusama.example network=kusama zone=eu-central reason=timeout after 5s
INFO rpc_monitor: monitor run complete duration_secs=55.0
```

Check journal priorities:

```bash
journalctl -u rpc-monitor.service -o json -n 20 --no-pager | jq '{priority: .PRIORITY, message: .MESSAGE}'
```

Error events also expose structured journald fields such as `F_ERROR`, `F_WSS`,
`F_NETWORK`, `F_ZONE`, and `F_REASON`.

Example Promtail journal scrape for errors only:

```yaml
scrape_configs:
  - job_name: rpc-monitor-errors
    journal:
      max_age: 12h
      labels:
        job: rpc-monitor
        host: monitor
    relabel_configs:
      - source_labels: ['__journal__systemd_unit']
        regex: 'rpc-monitor.service'
        action: keep
      - source_labels: ['__journal_priority']
        regex: '[0-3]'
        action: keep
      - source_labels: ['__journal_priority_keyword']
        target_label: level
      - source_labels: ['__journal__systemd_unit']
        target_label: unit
      - source_labels: ['__journal__hostname']
        target_label: hostname
```

Remove the `__journal_priority` relabel rule if Loki should receive both INFO
and ERROR lines.

## Prometheus Scraping

The monitor writes Prometheus text format to `output_path`. Serve that file via
nginx or another static HTTP server.

Example Prometheus scrape configuration:

```yaml
global:
  scrape_interval: 10m
  evaluation_interval: 10m

scrape_configs:
  - job_name: "rpc-mon"
    metrics_path: "/"
    static_configs:
      - targets: ["mon-eu-central.rpc-providers.net:80"]
      - targets: ["mon-us-east.rpc-providers.net:80"]
```

Example queries:

```promql
rpc_connect{zone=~"$zone",network=~"$network",wss=~"$wss"}
rpc_getblock{zone=~"$zone",network=~"$network",wss=~"$wss"}
rpc_getblockzero{zone=~"$zone",network=~"$network",wss=~"$wss"}
rpc_blockdrift{zone=~"$zone",network=~"$network",wss=~"$wss"}
rpc_error{zone=~"$zone",network=~"$network",wss=~"$wss"}
```

## Adding Or Changing Endpoints

Production deployment is a change and therefore requires explicit user
confirmation before it is performed. Preparing and validating the change on
`monitor.rpc-providers.net` is allowed; copying configuration or binaries to a
`mon-*` host is not.

1. Edit `monitor.toml`.
2. Add or update the relevant `[[endpoints]]` entry.
3. Deploy the updated config as `/opt/rpc-monitor/monitor.toml` on monitor nodes.
4. Run a dry run for each affected zone.
5. Restart or wait for the systemd timer.

Validation examples:

```bash
/opt/rpc-monitor/target/release/rpc-monitor --config /opt/rpc-monitor/monitor.toml --zone eu-central --dry-run
/opt/rpc-monitor/target/release/rpc-monitor --config /opt/rpc-monitor/monitor.toml --zone us-east --dry-run
```

## Troubleshooting

Check the last run:

```bash
journalctl -u rpc-monitor.service -n 100 --no-pager
```

Check only errors:

```bash
journalctl -u rpc-monitor.service -p err -n 100 --no-pager
```

Check whether the timer is active:

```bash
systemctl list-timers rpc-monitor.timer
```

Run one manual measurement without writing the production output file:

```bash
/opt/rpc-monitor/target/release/rpc-monitor \
  --config /opt/rpc-monitor/monitor.toml \
  --zone eu-central \
  --output /tmp/rpc-monitor-test.txt
```

Check DNS/IP reachability:

```bash
/opt/rpc-monitor/target/release/rpc-monitor \
  --config /opt/rpc-monitor/monitor.toml \
  --zone eu-central \
  --check-ips
```

Common causes of endpoint errors:

- `connect`: DNS, TCP, TLS, websocket handshake, IPv4/IPv6 or routing problem.
- `blockzero`: genesis block retrieval failed or endpoint is not archive-capable.
- `getblock`: latest block retrieval failed.
- `height`: latest header retrieval failed.
- `version`: `system_version` failed.

For endpoints where block-zero retrieval is not relevant, leave the network out
of `[zero_hashes]`; the monitor will skip the block-zero request and emit
`rpc_error{error="blockzero"} 0`.
