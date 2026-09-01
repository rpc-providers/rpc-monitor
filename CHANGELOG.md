# Changelog

All notable changes to RPC Monitor are recorded here. Releases use
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [1.1.0] - 2026-09-01

### Added

- `mon-ap-southeast.rpc-providers.net` is documented as an additional public
  monitor endpoint and included in the deploy target list.
- The public README again links the Telegram alerting group.

### Changed

- Error messages now include `observed_at` as a UTC ISO-8601 timestamp in both
  the visible journald `MESSAGE` and structured logging fields.
- Full `report` supports explicit `--start` and `--end` boundaries in addition
  to `--month`, using the same date handling as `report short`.
- Full reports now use latest-block retrieval (`rpc_getblock` and
  `rpc_error{error="getblock"}`) for block latency and uptime, so non-archive
  Bulletin-style nodes no longer show `N/A` just because block-zero checks are
  skipped.

## [1.0.0] - 2026-06-21

### Added

- Rust-based websocket RPC monitoring with Prometheus textfile output.
- Full reports and date-driven `report short` RPC-call reports.
- Per-IP diagnostic checks and native journald logging.

### Compatibility

- First stable release.
