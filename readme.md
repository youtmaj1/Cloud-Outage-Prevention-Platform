# Predictive Cloud Outage Prevention Platform (PCOPP)

## System Definition
A kernel-native anomaly detection system that predicts node-level cloud outages 20–60 minutes before they occur by analyzing low-level OS telemetry (syscalls, IRQs, memory fragmentation) rather than lagging indicators (CPU%, HTTP 500s).

## The Problem
* **Latency:** Standard observability tools (Datadog, Prometheus) detect outages *after* services fail.
* **Visibility:** Root causes often originate in kernel space (e.g., D-state thread pileups) which user-space agents miss.
* **Noise:** Alert fatigue from static thresholds prevents proactive remediation.

## Architecture
                                      +------------------+
                                      |   Alerts / API   |
                                      +--------+---------+
                                               ^
                                               |
[ Kernel Space ]                       +-------+-------+
| eBPF Probes  | --(Ring Buffer)-->    | Inference Engine |
| (Tracepoints)|                       | (TinyML / GPU)|
+------+-------+                       +-------+-------+
       |                                       ^
       |                                       |
+------+-------+      +-------------+   +------+-------+
|  Rust Agent  | ---> | FastAPI Ingest|-->|  PostgreSQL  |
| (Aggregation)|      | (Validation)|   | (Time Series)|
+--------------+      +-------------+   +--------------+

## Non-Goals
* **Root Cause *Resolution*:** IT predicts and alerts; IT does not auto-restart pods (yet).
* **General Purpose APM:** IT does not trace HTTP requests or application logs.
* **Real-time Debugging:** This is a prediction engine, not a live debugger.