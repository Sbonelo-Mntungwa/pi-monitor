use serde::Serialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize)]
pub struct MemoryMetrics {
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub buffers_bytes: u64,
    pub cached_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_free_bytes: u64,
}

/// Read and parse /proc/meminfo from the filesystem.
pub fn read_memory_metrics() -> Result<MemoryMetrics, String> {
    let content = fs::read_to_string("/proc/meminfo")
        .map_err(|e| format!("Failed to read /proc/meminfo: {}", e))?;
    parse_meminfo(&content)
}

/// Parse /proc/meminfo content string. Separated from read for testability.
pub fn parse_meminfo(content: &str) -> Result<MemoryMetrics, String> {
    let fields = parse_fields(content);

    let total = get_field_kb(&fields, "MemTotal")?;
    let free = get_field_kb(&fields, "MemFree")?;
    let available = get_field_kb(&fields, "MemAvailable").unwrap_or(free);
    let buffers = get_field_kb(&fields, "Buffers").unwrap_or(0);
    let cached = get_field_kb(&fields, "Cached").unwrap_or(0);
    let swap_total = get_field_kb(&fields, "SwapTotal").unwrap_or(0);
    let swap_free = get_field_kb(&fields, "SwapFree").unwrap_or(0);

    // used = total - free - buffers - cached
    // This matches `free` command output and htop's calculation.
    let used = total.saturating_sub(free).saturating_sub(buffers).saturating_sub(cached);

    Ok(MemoryMetrics {
        total_bytes: total * 1024,
        free_bytes: free * 1024,
        available_bytes: available * 1024,
        used_bytes: used * 1024,
        buffers_bytes: buffers * 1024,
        cached_bytes: cached * 1024,
        swap_total_bytes: swap_total * 1024,
        swap_free_bytes: swap_free * 1024,
    })
}

/// Parse "Key: 12345 kB" lines into a map of key → kB value.
fn parse_fields(content: &str) -> HashMap<String, u64> {
    let mut map = HashMap::new();
    for line in content.lines() {
        if let Some((key, rest)) = line.split_once(':') {
            let value: u64 = rest
                .split_whitespace()
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            map.insert(key.to_string(), value);
        }
    }
    map
}

/// Look up a required field from the parsed map.
fn get_field_kb(fields: &HashMap<String, u64>, key: &str) -> Result<u64, String> {
    fields.get(key).copied().ok_or_else(|| format!("Missing field: {}", key))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_MEMINFO: &str = "\
MemTotal:         425688 kB
MemFree:          390580 kB
MemAvailable:     390564 kB
Buffers:             484 kB
Cached:             2776 kB
SwapCached:            0 kB
Active:             3176 kB
Inactive:            584 kB
Active(anon):        512 kB
Inactive(anon):        0 kB
Active(file):       2664 kB
Inactive(file):      584 kB
Unevictable:           0 kB
Mlocked:               0 kB
SwapTotal:             0 kB
SwapFree:              0 kB
Dirty:                 4 kB
Writeback:             0 kB
AnonPages:           516 kB
Mapped:             2080 kB
Shmem:                 0 kB
KReclaimable:       4496 kB
Slab:              15080 kB
SReclaimable:       4496 kB
SUnreclaim:        10584 kB
KernelStack:        1744 kB
PageTables:          284 kB
CommitLimit:      212844 kB
Committed_AS:       2792 kB
VmallocTotal:   261087232 kB
VmallocUsed:        4256 kB
Percpu:              576 kB
CmaTotal:          65536 kB
CmaFree:           61096 kB";

    #[test]
    fn test_parse_real_meminfo() {
        let m = parse_meminfo(SAMPLE_MEMINFO).unwrap();
        assert_eq!(m.total_bytes, 425688 * 1024);
        assert_eq!(m.free_bytes, 390580 * 1024);
        assert_eq!(m.available_bytes, 390564 * 1024);
        assert_eq!(m.buffers_bytes, 484 * 1024);
        assert_eq!(m.cached_bytes, 2776 * 1024);
    }

    #[test]
    fn test_used_calculation() {
        let m = parse_meminfo(SAMPLE_MEMINFO).unwrap();
        // used = total - free - buffers - cached
        let expected = (425688 - 390580 - 484 - 2776) * 1024;
        assert_eq!(m.used_bytes, expected);
    }

    #[test]
    fn test_swap_zero() {
        let m = parse_meminfo(SAMPLE_MEMINFO).unwrap();
        assert_eq!(m.swap_total_bytes, 0);
        assert_eq!(m.swap_free_bytes, 0);
    }
}