# Changelog

All notable changes to RPC Monitor are recorded here. Releases use
[Semantic Versioning](https://semver.org/).

## [1.2.0] - 2026-11-01

### Added

- Dated tender catalogue and per-endpoint tender memberships, with validation
  of IDs, references, duplicate entries and UTC date boundaries.
- Separate `rpc_endpoint_info` and `rpc_tender_active` metadata gauges;
  existing measurement labels and history are unchanged. The reserved
  `__current` membership follows active tenders automatically.
- Full and short reports support repeatable `--tender` and `--all-endpoints`.
  Dated membership clips and merges eligible intervals without double-counting
  overlaps; statistics use eligible samples. Rows show applicable tender IDs
  after Network, and the heading lists the selected tenders for the report period.
  The title and introductory metric descriptions retain the previous report style.
  Provider/chain usage is explicitly identified as shared, not slot-level.

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
