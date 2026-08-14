# Changelog

All notable changes to RPC Monitor are recorded here. Releases use
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- HTTP JSON-RPC monitoring: endpoints can be checked over `https://` with
  `transport = "http"` (HTTP/3-optimistic with HTTP/2/HTTP/1.1 fallback) or
  `transport = "http3"` (forced QUIC), alongside the default websocket checks.
- EVM (`eth-rpc`) endpoint support via `flavor = "eth"`: checks use Ethereum
  JSON-RPC methods (`eth_getBlockByNumber`, `web3_clientVersion`) so Asset Hub
  EVM proxies can be monitored over HTTP. Block-zero is skipped when the proxy
  has no genesis block. Default `flavor` is `substrate`.
- `transport` label (`ws`, `http`, `http3`) on all emitted metrics so
  dashboards can show websocket and HTTP values together.
- `rpc_httpversion` metric reporting the HTTP version an endpoint actually
  answered on (`http_version="h3"|"h2"|"http1.1"`), so an
  HTTP/3-optimistic check shows whether QUIC won or the fallback was used.
- HTTP (https) entries for every member endpoint whose HTTPS JSON-RPC endpoint
  was verified, so both websocket and http are measured.
- Importable Grafana dashboard mirroring rpc-providers.net with HTTP values
  added: `deploy/grafana/rpc-providers-ws-http.json`.

## [1.0.0] - 2026-06-21

### Added

- Rust-based websocket RPC monitoring with Prometheus textfile output.
- Full reports and date-driven `report short` RPC-call reports.
- Per-IP diagnostic checks and native journald logging.

### Compatibility

- First stable release.
