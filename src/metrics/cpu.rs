// CPU Metrics Collector
// =====================
//
// This module will parse /proc/stat to calculate CPU usage percentages.
//
// Phase 2 will implement:
//   - Reading /proc/stat and parsing the cumulative tick counters
//   - Computing CPU usage % by comparing two snapshots over time
//   - Per-core and total CPU breakdowns (user, system, idle, iowait)
//   - A background sampling task that updates every 2 seconds
//
// For now, this is a stub that defines the data structures we'll use.
// Having the struct definitions early lets us verify the module structure compiles.

use serde::Serialize;

/// CPU usage percentages for a single CPU (or the "total" aggregate).
///
/// All values are percentages (0.0 to 100.0).
/// They should sum to approximately 100.0.
///
/// Why these specific fields?
///   - user: Time running user-space processes (your code)
///   - system: Time running kernel code (syscalls, drivers)
///   - idle: Time doing nothing
///   - iowait: Time waiting for disk/network I/O to complete
///   These are the most useful breakdown for diagnosing performance issues.
///   /proc/stat has more fields (nice, irq, softirq, steal, guest) but
///   these four cover 99% of monitoring use cases.
#[derive(Debug, Clone, Serialize)]
pub struct CpuUsage {
    /// Which CPU this represents: "total", "cpu0", "cpu1", etc.
    pub cpu: String,
    /// Percentage of time in user mode
    pub user_percent: f64,
    /// Percentage of time in kernel/system mode
    pub system_percent: f64,
    /// Percentage of time idle
    pub idle_percent: f64,
    /// Percentage of time waiting for I/O
    pub iowait_percent: f64,
}

/// Snapshot of all CPU metrics at a point in time.
///
/// Contains the total (aggregate across all cores) plus per-core breakdowns.
#[derive(Debug, Clone, Serialize)]
pub struct CpuMetrics {
    pub total: CpuUsage,
    pub per_core: Vec<CpuUsage>,
}

// Phase 2 will add:
//   pub fn read_proc_stat() -> Result<RawCpuCounters, ...>
//   pub fn calculate_usage(prev: &RawCpuCounters, curr: &RawCpuCounters) -> CpuMetrics
//   pub async fn cpu_sampling_task(shared_state: Arc<Mutex<CpuMetrics>>)