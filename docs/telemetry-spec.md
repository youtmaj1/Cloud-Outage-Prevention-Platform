# Telemetry Specification v0.1

All metrics are emitted at a 10-second sampling interval (configurable).

## Metadata (Attached to every payload)
* `node_id` (UUID): Unique identifier for the host/VM.
* `kernel_version` (String): e.g., "5.15.0-1041-azure".
* `timestamp_utc` (ISO8601): Collection time.

## Metric Group A: Memory Health
* `mem_frag_index` (Float, 0.0-1.0): External fragmentation index for order-3 pages.
* `pgmajfault_rate` (Int): Major page faults per second (requires disk I/O).
* `oom_kill_count` (Counter): Cumulative OOM kills since boot.

## Metric Group B: Scheduler Health
* `load_avg_1m` (Float): Standard load average.
* `procs_blocked` (Int): Number of processes in `TASK_UNINTERRUPTIBLE` state.
* `context_switch_rate` (Int): Context switches per second.

## Metric Group C: I/O Pressure
* `dirty_pages_bytes` (Int): Amount of memory waiting to be written to disk.
* `disk_io_time_weighted` (Int): Weighted time spent doing I/Os (ms).

## Schema Validation Rules
1. `timestamp_utc` must not be > 5 minutes in the past.
2. `mem_frag_index` must be between 0.0 and 1.0.
3. Negative values for counters are rejected.