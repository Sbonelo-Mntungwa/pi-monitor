use serde::Serialize;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RawCpuCounters {
    pub name: String,
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
    pub steal: u64,
    pub guest: u64,
    pub guest_nice: u64,
}

impl RawCpuCounters {
    /// Excludes guest/guest_nice — they're already counted in user/nice.
    pub fn total_ticks(&self) -> u64 {
        self.user + self.nice + self.system + self.idle
            + self.iowait + self.irq + self.softirq + self.steal
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CpuUsage {
    pub cpu: String,
    pub user_percent: f64,
    pub system_percent: f64,
    pub idle_percent: f64,
    pub iowait_percent: f64,
}

impl Default for CpuUsage {
    fn default() -> Self {
        CpuUsage {
            cpu: "total".to_string(),
            user_percent: 0.0,
            system_percent: 0.0,
            idle_percent: 100.0,
            iowait_percent: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CpuMetrics {
    pub total: CpuUsage,
    pub per_core: Vec<CpuUsage>,
}

impl Default for CpuMetrics {
    fn default() -> Self {
        CpuMetrics {
            total: CpuUsage::default(),
            per_core: Vec::new(),
        }
    }
}

/// Parse a single "cpu" line from /proc/stat into raw tick counters.
fn parse_cpu_line(line: &str) -> Option<RawCpuCounters> {
    let mut parts = line.split_whitespace();
    let name = parts.next()?;

    if !name.starts_with("cpu") {
        return None;
    }

    let values: Vec<u64> = parts.filter_map(|s| s.parse().ok()).collect();
    if values.len() < 7 {
        return None;
    }

    Some(RawCpuCounters {
        name: name.to_string(),
        user: values[0],
        nice: values[1],
        system: values[2],
        idle: values[3],
        iowait: values[4],
        irq: values[5],
        softirq: values[6],
        steal: *values.get(7).unwrap_or(&0),
        guest: *values.get(8).unwrap_or(&0),
        guest_nice: *values.get(9).unwrap_or(&0),
    })
}

/// Read and parse /proc/stat from the filesystem.
pub fn read_proc_stat() -> Result<Vec<RawCpuCounters>, String> {
    let content = fs::read_to_string("/proc/stat")
        .map_err(|e| format!("Failed to read /proc/stat: {}", e))?;
    parse_proc_stat(&content)
}

/// Parse /proc/stat content string. Separated from read_proc_stat() for testability.
pub fn parse_proc_stat(content: &str) -> Result<Vec<RawCpuCounters>, String> {
    let counters: Vec<RawCpuCounters> = content.lines().filter_map(parse_cpu_line).collect();

    if counters.is_empty() {
        return Err("No CPU lines found in /proc/stat".to_string());
    }
    if counters[0].name != "cpu" {
        return Err(format!("Expected first line to be 'cpu', got '{}'", counters[0].name));
    }

    Ok(counters)
}

/// Compare two snapshots and compute CPU usage percentages from the diff.
pub fn calculate_usage(prev: &[RawCpuCounters], curr: &[RawCpuCounters]) -> CpuMetrics {
    let mut usages: Vec<CpuUsage> = Vec::new();

    for curr_cpu in curr {
        let prev_cpu = prev.iter().find(|p| p.name == curr_cpu.name);
        let usage = match prev_cpu {
            Some(p) => compute_single_usage(p, curr_cpu),
            None => CpuUsage {
                cpu: display_name(&curr_cpu.name),
                ..CpuUsage::default()
            },
        };
        usages.push(usage);
    }

    let total = usages.first().cloned().unwrap_or_default();
    let per_core = usages.into_iter().skip(1).collect();
    CpuMetrics { total, per_core }
}

/// Compute usage percentage for a single CPU by diffing two readings.
fn compute_single_usage(prev: &RawCpuCounters, curr: &RawCpuCounters) -> CpuUsage {
    let user_diff = curr.user.saturating_sub(prev.user)
        + curr.nice.saturating_sub(prev.nice);
    let system_diff = curr.system.saturating_sub(prev.system)
        + curr.irq.saturating_sub(prev.irq)
        + curr.softirq.saturating_sub(prev.softirq);
    let idle_diff = curr.idle.saturating_sub(prev.idle);
    let iowait_diff = curr.iowait.saturating_sub(prev.iowait);
    let steal_diff = curr.steal.saturating_sub(prev.steal);

    let total_diff = user_diff + system_diff + idle_diff + iowait_diff + steal_diff;

    if total_diff == 0 {
        return CpuUsage {
            cpu: display_name(&prev.name),
            ..CpuUsage::default()
        };
    }

    let t = total_diff as f64;
    CpuUsage {
        cpu: display_name(&prev.name),
        user_percent: round2((user_diff as f64 / t) * 100.0),
        system_percent: round2((system_diff as f64 / t) * 100.0),
        idle_percent: round2((idle_diff as f64 / t) * 100.0),
        iowait_percent: round2((iowait_diff as f64 / t) * 100.0),
    }
}

/// "cpu" → "total", "cpu0" → "cpu0".
fn display_name(name: &str) -> String {
    if name == "cpu" { "total".to_string() } else { name.to_string() }
}

fn round2(val: f64) -> f64 {
    (val * 100.0).round() / 100.0
}

pub type SharedCpuMetrics = Arc<Mutex<CpuMetrics>>;

/// Create shared state initialized with default (idle) metrics.
pub fn new_shared_metrics() -> SharedCpuMetrics {
    Arc::new(Mutex::new(CpuMetrics::default()))
}

/// Background task: samples /proc/stat every `interval` and updates shared state.
pub async fn cpu_sampling_task(shared: SharedCpuMetrics, interval: Duration) {
    let mut previous: Option<Vec<RawCpuCounters>> = None;

    loop {
        match read_proc_stat() {
            Ok(current) => {
                if let Some(ref prev) = previous {
                    let metrics = calculate_usage(prev, &current);
                    if let Ok(mut guard) = shared.lock() {
                        *guard = metrics;
                    }
                }
                previous = Some(current);
            }
            Err(e) => eprintln!("Failed to read CPU stats: {}", e),
        }
        tokio::time::sleep(interval).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_PROC_STAT: &str = "\
cpu  10 0 460 22971 17 0 2 0 0 0
cpu0 0 0 108 5789 5 0 0 0 0 0
cpu1 1 0 217 5543 1 0 0 0 0 0
cpu2 8 0 30 5855 1 0 0 0 0 0
cpu3 0 0 102 5782 9 0 2 0 0 0
intr 78183 0 0 292
ctxt 10383
btime 0
processes 130
procs_running 2
procs_blocked 0
softirq 37976 3485 2691 0 57 0 0 27776 2968 0 999";

    #[test]
    fn test_parse_real_proc_stat() {
        let counters = parse_proc_stat(SAMPLE_PROC_STAT).unwrap();
        assert_eq!(counters.len(), 5);
        assert_eq!(counters[0].name, "cpu");
        assert_eq!(counters[0].user, 10);
        assert_eq!(counters[0].system, 460);
        assert_eq!(counters[0].idle, 22971);
        assert_eq!(counters[1].name, "cpu0");
        assert_eq!(counters[4].name, "cpu3");
    }

    #[test]
    fn test_total_ticks() {
        let counters = parse_proc_stat(SAMPLE_PROC_STAT).unwrap();
        assert_eq!(counters[0].total_ticks(), 23460);
    }

    #[test]
    fn test_non_cpu_lines_skipped() {
        let counters = parse_proc_stat(SAMPLE_PROC_STAT).unwrap();
        assert!(counters.iter().all(|c| c.name.starts_with("cpu")));
    }

    #[test]
    fn test_calculate_usage() {
        let prev = vec![RawCpuCounters {
            name: "cpu".into(), user: 50, nice: 0, system: 100, idle: 1000,
            iowait: 0, irq: 0, softirq: 0, steal: 0, guest: 0, guest_nice: 0,
        }];
        let curr = vec![RawCpuCounters {
            name: "cpu".into(), user: 55, nice: 0, system: 110, idle: 1180,
            iowait: 0, irq: 0, softirq: 0, steal: 0, guest: 0, guest_nice: 0,
        }];

        let m = calculate_usage(&prev, &curr);
        assert!((m.total.user_percent - 2.56).abs() < 0.1);
        assert!((m.total.system_percent - 5.13).abs() < 0.1);
        assert!((m.total.idle_percent - 92.31).abs() < 0.1);
    }

    #[test]
    fn test_zero_diff_returns_idle() {
        let s = vec![RawCpuCounters {
            name: "cpu".into(), user: 50, nice: 0, system: 100, idle: 1000,
            iowait: 0, irq: 0, softirq: 0, steal: 0, guest: 0, guest_nice: 0,
        }];
        let m = calculate_usage(&s, &s);
        assert_eq!(m.total.idle_percent, 100.0);
    }

    #[test]
    fn test_missing_fields_default_to_zero() {
        let c = parse_cpu_line("cpu  100 0 200 5000 10 0 5").unwrap();
        assert_eq!(c.steal, 0);
        assert_eq!(c.guest, 0);
    }
}