mod signals;

use serde::Serialize;
use uuid::Uuid;
use chrono::Utc;
use tokio::time::{sleep, Duration};

// 1. Define Structs that MATCH the JSON Schema exactly
// This is the Rust-side implementation of the Contract
#[derive(Serialize)]
struct TelemetryPayload {
    meta: MetaData,
    memory: MemoryData,
    scheduler: SchedulerData,
    io: IoData,
}

#[derive(Serialize)]
struct MetaData {
    node_id: String,
    timestamp_utc: String,
    kernel_version: String,
}

#[derive(Serialize)]
struct MemoryData {
    mem_frag_index: f64,
    oom_kill_count: u32,
}

#[derive(Serialize)]
struct SchedulerData {
    load_avg_1m: f64,
    procs_blocked: u32,
}

#[derive(Serialize)]
struct IoData {
    dirty_pages_bytes: u64,
}

// 2. The Main Loop
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let node_uuid = Uuid::new_v4().to_string();
    let ingest_url = "http://localhost:8001/ingest";

    println!("🚀 Agent starting. Node ID: {}", node_uuid);

    let mut backoff = Duration::from_secs(1);

    loop {
        // 1. COLLECT REAL DATA
        // If collection fails (e.g., permissions), we log it but don't crash
        let signals = match signals::collect() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("⚠️ Failed to read kernel signals: {}", e);
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };
        
        let payload = TelemetryPayload {
            meta: MetaData {
                node_id: node_uuid.clone(),
                timestamp_utc: Utc::now().to_rfc3339(),
                kernel_version: "5.15.0-generic".to_string(), // Real kernel
            },
            memory: MemoryData {
                mem_frag_index: signals.mem_frag_index,
                oom_kill_count: 0, // TODO: Read from /proc/vmstat
            },
            scheduler: SchedulerData {
                load_avg_1m: signals.load_avg_1m,
                procs_blocked: signals.procs_blocked,
            },
            io: IoData {
                dirty_pages_bytes: signals.dirty_pages_bytes,
            },
        };

        // Debug: Print signals
        println!("📊 Signals - Frag: {:.3}, Blocked: {}, Load: {:.2}, Dirty: {}",
                 signals.mem_frag_index, signals.procs_blocked, signals.load_avg_1m, signals.dirty_pages_bytes);

        // 4. Transmit
        match client.post(ingest_url).json(&payload).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    println!("✅ Sent payload. Status: {}", resp.status());
                    backoff = Duration::from_secs(1); // Reset backoff on success
                    sleep(Duration::from_secs(5)).await; // Normal heartbeat
                } else {
                    eprintln!("❌ Server rejected: {}", resp.status());
                    sleep(backoff).await; // Backoff delay
                    backoff = (backoff * 2).min(Duration::from_secs(60)); // Exponential backoff, cap at 60s
                }
            }
            Err(e) => {
                eprintln!("❌ Connection failed: {}", e);
                sleep(backoff).await; // Backoff on connection error
                backoff = (backoff * 2).min(Duration::from_secs(60));
            }
        }
    }
}