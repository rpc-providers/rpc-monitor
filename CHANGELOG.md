# Changelog

All notable changes to RPC Monitor are recorded here. Releases use
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Dashboard migration utility creates a separate archive-status preview,
  replacing block-zero timings/errors with latest-block metrics and uptime.

### Changed

- Replace genesis-block checks with the historical `:code` storage hash at block
  1000 for all endpoints, including Bulletin, using `state_getStorageHash` to
  avoid historical pallet-key differences and large runtime downloads.
  New `rpc_archive_status` gauge:
  1 archive, 0 explicitly pruned, -1 unknown. Classification is not an error;
  connection/latest-block/height/version errors remain unchanged.
- Stop emitting `rpc_getblockzero` and `rpc_error{error="blockzero"}`. Genesis
  hash lists are no longer needed. `archive_secs` configures the total probe
  deadline; legacy `block_zero_secs` remains accepted as an alias.

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
