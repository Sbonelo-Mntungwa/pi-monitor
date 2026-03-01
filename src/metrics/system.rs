use serde::Serialize;
use std::fs;

#[derive(Debug, Clone, Serialize)]
pub struct SystemMetrics {
    pub load_1: f64,
    pub load_5: f64,
    pub load_15: f64,
    pub processes_running: u32,
    pub processes_total: u32,
    pub uptime_seconds: f64,
}

/// Read system metrics from /proc/loadavg and /proc/uptime.
pub fn read_system_metrics() -> Result<SystemMetrics, String> {
    let loadavg = fs::read_to_string("/proc/loadavg")
        .map_err(|e| format!("Failed to read /proc/loadavg: {}", e))?;
    let uptime = fs::read_to_string("/proc/uptime")
        .map_err(|e| format!("Failed to read /proc/uptime: {}", e))?;
    parse_system_metrics(&loadavg, &uptime)
}

/// Parse /proc/loadavg and /proc/uptime content strings.
pub fn parse_system_metrics(loadavg: &str, uptime: &str) -> Result<SystemMetrics, String> {
    let (load_1, load_5, load_15, running, total) = parse_loadavg(loadavg)?;
    let uptime_seconds = parse_uptime(uptime)?;

    Ok(SystemMetrics {
        load_1,
        load_5,
        load_15,
        processes_running: running,
        processes_total: total,
        uptime_seconds,
    })
}

/// Parse "0.10 0.04 0.01 1/105 127" into load averages and process counts.
fn parse_loadavg(content: &str) -> Result<(f64, f64, f64, u32, u32), String> {
    let parts: Vec<&str> = content.trim().split_whitespace().collect();
    if parts.len() < 4 {
        return Err(format!("Unexpected /proc/loadavg format: {}", content));
    }

    let load_1: f64 = parts[0].parse().map_err(|e| format!("Bad load_1: {}", e))?;
    let load_5: f64 = parts[1].parse().map_err(|e| format!("Bad load_5: {}", e))?;
    let load_15: f64 = parts[2].parse().map_err(|e| format!("Bad load_15: {}", e))?;

    // "1/105" → running=1, total=105
    let (running, total) = parts[3]
        .split_once('/')
        .ok_or_else(|| format!("Bad process field: {}", parts[3]))?;

    let running: u32 = running.parse().map_err(|e| format!("Bad running count: {}", e))?;
    let total: u32 = total.parse().map_err(|e| format!("Bad total count: {}", e))?;

    Ok((load_1, load_5, load_15, running, total))
}

/// Parse "61.38 238.71" → 61.38 (first value is uptime in seconds).
fn parse_uptime(content: &str) -> Result<f64, String> {
    content
        .trim()
        .split_whitespace()
        .next()
        .ok_or_else(|| "Empty /proc/uptime".to_string())?
        .parse()
        .map_err(|e| format!("Bad uptime value: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_real_data() {
        let m = parse_system_metrics("0.10 0.04 0.01 1/105 127\n", "61.38 238.71\n").unwrap();
        assert!((m.load_1 - 0.10).abs() < 0.001);
        assert!((m.load_5 - 0.04).abs() < 0.001);
        assert!((m.load_15 - 0.01).abs() < 0.001);
        assert_eq!(m.processes_running, 1);
        assert_eq!(m.processes_total, 105);
        assert!((m.uptime_seconds - 61.38).abs() < 0.01);
    }

    #[test]
    fn test_parse_high_load() {
        let m = parse_system_metrics("4.50 3.20 2.10 8/300 9999\n", "86400.00 100000.00\n").unwrap();
        assert!((m.load_1 - 4.50).abs() < 0.001);
        assert_eq!(m.processes_running, 8);
        assert_eq!(m.processes_total, 300);
    }
}