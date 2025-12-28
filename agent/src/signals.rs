use std::fs;
use std::error::Error;

pub struct SystemSignals {
    pub mem_frag_index: f64,
    pub procs_blocked: u32,
    pub dirty_pages_bytes: u64,
    pub load_avg_1m: f64,
}

pub fn collect() -> Result<SystemSignals, Box<dyn Error>> {
    Ok(SystemSignals {
        mem_frag_index: read_fragmentation()?,
        procs_blocked: read_procs_blocked()?,
        dirty_pages_bytes: read_dirty_pages()?,
        load_avg_1m: read_load_avg()?,
    })
}

// 1. Memory Fragmentation (The "Silent Killer")
// Reads /proc/buddyinfo
fn read_fragmentation() -> Result<f64, Box<dyn Error>> {
    let content = fs::read_to_string("/proc/buddyinfo")?;
    
    let mut total_free_pages = 0.0;
    let mut huge_free_pages = 0.0;

    for line in content.lines() {
        // Format: Node 0, Zone Normal 100 50 20 ... (11 numbers)
        // Each number represents count of free blocks at Order N (2^N * 4KB)
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 15 { continue; } // Skip malformed lines

        // The counts start at index 4 (after "Node 0 Zone Normal")
        for (order, count_str) in parts[4..].iter().enumerate() {
            let count: f64 = count_str.parse().unwrap_or(0.0);
            let pages_in_block = 2_u32.pow(order as u32) as f64;
            let total_pages_at_order = count * pages_in_block;

            total_free_pages += total_pages_at_order;

            // We define "Huge" as Order 9 and 10 (2MB and 4MB blocks)
            if order >= 9 {
                huge_free_pages += total_pages_at_order;
            }
        }
    }

    if total_free_pages == 0.0 { return Ok(0.0); }
    
    let frag = huge_free_pages / total_free_pages;
    Ok(frag)
}

// 2. Processes in D-State (Uninterruptible Sleep)
// Reads /proc/stat
fn read_procs_blocked() -> Result<u32, Box<dyn Error>> {
    let content = fs::read_to_string("/proc/stat")?;
    
    for line in content.lines() {
        if line.starts_with("procs_blocked") {
            // Format: procs_blocked 0
            let parts: Vec<&str> = line.split_whitespace().collect();
            return Ok(parts.get(1).unwrap_or(&"0").parse()?);
        }
    }
    Ok(0)
}

// 3. Dirty Pages
// Reads /proc/vmstat
fn read_dirty_pages() -> Result<u64, Box<dyn Error>> {
    let content = fs::read_to_string("/proc/vmstat")?;
    
    for line in content.lines() {
        if line.starts_with("nr_dirty ") {
            // Format: nr_dirty 1234
            let parts: Vec<&str> = line.split_whitespace().collect();
            let dirty: u64 = parts.get(1).unwrap_or(&"0").parse()?;
            // Convert to bytes (assuming 4KB pages)
            return Ok(dirty * 4096);
        }
    }
    Ok(0)
}

// 4. Load Average
// Reads /proc/loadavg
fn read_load_avg() -> Result<f64, Box<dyn Error>> {
    let content = fs::read_to_string("/proc/loadavg")?;
    // Format: 0.00 0.00 0.00 1/123 1234
    let parts: Vec<&str> = content.split_whitespace().collect();
    Ok(parts.get(0).unwrap_or(&"0.0").parse()?)
}