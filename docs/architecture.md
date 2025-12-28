# System Architecture & Boundaries

## 1. Telemetry Agent (The "Eye")
* **Responsibility:** collect kernel signals, aggregate in-memory (1s windows), serialize to Protobuf/JSON, transmit to Ingest.
* **Constraints:** MUST NOT consume > 2% CPU or > 50MB RAM. MUST fail open (crash without taking down the host).
* **Language:** Rust (w/ Aya or Libbpf-rs).

## 2. Ingestion Gateway (The "Gatekeeper")
* **Responsibility:** Authenticate agents, validate schema strictness, buffer for write-efficiency, push to storage.
* **Failure Mode:** If DB is down, return 503 to Agent (Agent handles backoff).
* **Language:** Python (FastAPI).

## 3. Storage Layer (The "Memory")
* **Responsibility:** Store raw telemetry for training (7 days retention) and aggregated trends for inference.
* **Data Flow:** Write-heavy, Read-heavy during training.
* **Technology:** PostgreSQL (TimescaleDB extension recommended).

## 4. Inference Engine (The "Brain")
* **Responsibility:** Fetch recent windows, run simple heuristic models (Day 1) or ML models (Day N), emit "Risk Score" (0.0 - 1.0).
* **Boundary:** Does NOT talk to the Kernel directly. Consumes data only from Storage/Ingest.

## Critical Data Flow
Kernel -> [eBPF] -> Agent -> (HTTP/gRPC) -> Ingest -> Database -> ML Trainer