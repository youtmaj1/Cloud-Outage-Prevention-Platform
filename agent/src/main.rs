use serde::Serialize;
use uuid::Uuid;
use chrono::Utc;
use rand::Rng;
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
        // 3. Generate FAKE compliant data (Day 1 logic)
        // We simulate a healthy node that occasionally spikes
        let mut rng = rand::thread_rng();
        
        let payload = TelemetryPayload {
            meta: MetaData {
                node_id: node_uuid.clone(),
                timestamp_utc: Utc::now().to_rfc3339(),
                kernel_version: "5.15.0-fake-kernel".to_string(),
            },
            memory: MemoryData {
                mem_frag_index: rng.gen_range(0.1..0.6), // Safe range
                oom_kill_count: 0,
            },
            scheduler: SchedulerData {
                load_avg_1m: rng.gen_range(0.5..2.0),
                procs_blocked: if rng.gen_bool(0.05) { 1 } else { 0 }, // 5% chance of D-state
            },
            io: IoData {
                dirty_pages_bytes: rng.gen_range(1024..4096),
            },
        };

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