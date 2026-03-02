use crate::metrics::{
    cpu::CpuMetrics,
    disk::DiskMetrics,
    memory::MemoryMetrics,
    network::InterfaceMetrics,
    system::SystemMetrics,
};
use std::fmt::Write;

/// Format CPU metrics in Prometheus exposition format.
pub fn format_cpu(metrics: &CpuMetrics, out: &mut String) {
    write_help_type(out, "pi_cpu_usage_percent", "CPU usage percentage", "gauge");

    // Total CPU
    write_labeled(out, "pi_cpu_usage_percent", &[("cpu", "total"), ("mode", "user")], metrics.total.user_percent);
    write_labeled(out, "pi_cpu_usage_percent", &[("cpu", "total"), ("mode", "system")], metrics.total.system_percent);
    write_labeled(out, "pi_cpu_usage_percent", &[("cpu", "total"), ("mode", "idle")], metrics.total.idle_percent);
    write_labeled(out, "pi_cpu_usage_percent", &[("cpu", "total"), ("mode", "iowait")], metrics.total.iowait_percent);

    // Per-core
    for core in &metrics.per_core {
        write_labeled(out, "pi_cpu_usage_percent", &[("cpu", &core.cpu), ("mode", "user")], core.user_percent);
        write_labeled(out, "pi_cpu_usage_percent", &[("cpu", &core.cpu), ("mode", "system")], core.system_percent);
        write_labeled(out, "pi_cpu_usage_percent", &[("cpu", &core.cpu), ("mode", "idle")], core.idle_percent);
        write_labeled(out, "pi_cpu_usage_percent", &[("cpu", &core.cpu), ("mode", "iowait")], core.iowait_percent);
    }

    out.push('\n');
}

/// Format memory metrics.
pub fn format_memory(mem: &MemoryMetrics, out: &mut String) {
    write_metric(out, "pi_memory_total_bytes", "Total memory in bytes", "gauge", mem.total_bytes as f64);
    write_metric(out, "pi_memory_free_bytes", "Free memory in bytes", "gauge", mem.free_bytes as f64);
    write_metric(out, "pi_memory_available_bytes", "Available memory in bytes", "gauge", mem.available_bytes as f64);
    write_metric(out, "pi_memory_used_bytes", "Used memory in bytes", "gauge", mem.used_bytes as f64);
    write_metric(out, "pi_memory_buffers_bytes", "Buffer memory in bytes", "gauge", mem.buffers_bytes as f64);
    write_metric(out, "pi_memory_cached_bytes", "Cached memory in bytes", "gauge", mem.cached_bytes as f64);
    write_metric(out, "pi_memory_swap_total_bytes", "Total swap in bytes", "gauge", mem.swap_total_bytes as f64);
    write_metric(out, "pi_memory_swap_free_bytes", "Free swap in bytes", "gauge", mem.swap_free_bytes as f64);
    out.push('\n');
}

/// Format system metrics (load, uptime, processes).
pub fn format_system(sys: &SystemMetrics, out: &mut String) {
    write_help_type(out, "pi_load_average", "System load average", "gauge");
    write_labeled(out, "pi_load_average", &[("period", "1m")], sys.load_1);
    write_labeled(out, "pi_load_average", &[("period", "5m")], sys.load_5);
    write_labeled(out, "pi_load_average", &[("period", "15m")], sys.load_15);

    write_metric(out, "pi_uptime_seconds", "System uptime in seconds", "counter", sys.uptime_seconds);
    write_metric(out, "pi_processes_running", "Number of running processes", "gauge", sys.processes_running as f64);
    write_metric(out, "pi_processes_total", "Total number of processes", "gauge", sys.processes_total as f64);
    out.push('\n');
}

/// Format network interface metrics.
pub fn format_network(interfaces: &[InterfaceMetrics], out: &mut String) {
    write_help_type(out, "pi_network_receive_bytes_total", "Total bytes received", "counter");
    for iface in interfaces {
        write_labeled(out, "pi_network_receive_bytes_total", &[("interface", &iface.name)], iface.rx_bytes as f64);
    }

    write_help_type(out, "pi_network_transmit_bytes_total", "Total bytes transmitted", "counter");
    for iface in interfaces {
        write_labeled(out, "pi_network_transmit_bytes_total", &[("interface", &iface.name)], iface.tx_bytes as f64);
    }

    write_help_type(out, "pi_network_receive_packets_total", "Total packets received", "counter");
    for iface in interfaces {
        write_labeled(out, "pi_network_receive_packets_total", &[("interface", &iface.name)], iface.rx_packets as f64);
    }

    write_help_type(out, "pi_network_transmit_packets_total", "Total packets transmitted", "counter");
    for iface in interfaces {
        write_labeled(out, "pi_network_transmit_packets_total", &[("interface", &iface.name)], iface.tx_packets as f64);
    }

    write_help_type(out, "pi_network_receive_errors_total", "Total receive errors", "counter");
    for iface in interfaces {
        write_labeled(out, "pi_network_receive_errors_total", &[("interface", &iface.name)], iface.rx_errors as f64);
    }

    out.push('\n');
}

/// Format disk metrics.
pub fn format_disk(disks: &[DiskMetrics], out: &mut String) {
    write_help_type(out, "pi_disk_total_bytes", "Total disk space in bytes", "gauge");
    for d in disks {
        write_labeled(out, "pi_disk_total_bytes", &[("mountpoint", &d.mount_point), ("device", &d.device)], d.total_bytes as f64);
    }

    write_help_type(out, "pi_disk_used_bytes", "Used disk space in bytes", "gauge");
    for d in disks {
        write_labeled(out, "pi_disk_used_bytes", &[("mountpoint", &d.mount_point), ("device", &d.device)], d.used_bytes as f64);
    }

    write_help_type(out, "pi_disk_free_bytes", "Free disk space in bytes", "gauge");
    for d in disks {
        write_labeled(out, "pi_disk_free_bytes", &[("mountpoint", &d.mount_point), ("device", &d.device)], d.free_bytes as f64);
    }

    out.push('\n');
}

// ── Helpers ──────────────────────────────────────────────────────

/// Write a simple metric with HELP, TYPE, and a single value line.
fn write_metric(out: &mut String, name: &str, help: &str, metric_type: &str, value: f64) {
    write_help_type(out, name, help, metric_type);
    let _ = writeln!(out, "{} {}", name, format_value(value));
}

/// Write # HELP and # TYPE header lines.
fn write_help_type(out: &mut String, name: &str, help: &str, metric_type: &str) {
    let _ = writeln!(out, "# HELP {} {}", name, help);
    let _ = writeln!(out, "# TYPE {} {}", name, metric_type);
}

/// Write a metric line with labels: name{label1="val1",label2="val2"} value
fn write_labeled(out: &mut String, name: &str, labels: &[(&str, &str)], value: f64) {
    let _ = write!(out, "{}{{", name);
    for (i, (k, v)) in labels.iter().enumerate() {
        if i > 0 {
            let _ = write!(out, ",");
        }
        let _ = write!(out, "{}=\"{}\"", k, v);
    }
    let _ = writeln!(out, "}} {}", format_value(value));
}

/// Format a float cleanly: integers show as "123", floats as "12.34".
fn format_value(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::cpu::{CpuMetrics, CpuUsage};

    #[test]
    fn test_format_cpu() {
        let metrics = CpuMetrics {
            total: CpuUsage {
                cpu: "total".into(),
                user_percent: 5.5,
                system_percent: 2.3,
                idle_percent: 92.0,
                iowait_percent: 0.2,
            },
            per_core: vec![],
        };
        let mut out = String::new();
        format_cpu(&metrics, &mut out);

        assert!(out.contains("# HELP pi_cpu_usage_percent"));
        assert!(out.contains("# TYPE pi_cpu_usage_percent gauge"));
        assert!(out.contains(r#"pi_cpu_usage_percent{cpu="total",mode="user"} 5.5"#));
        assert!(out.contains(r#"pi_cpu_usage_percent{cpu="total",mode="idle"} 92"#));
    }

    #[test]
    fn test_format_value_integers() {
        assert_eq!(format_value(100.0), "100");
        assert_eq!(format_value(0.0), "0");
    }

    #[test]
    fn test_format_value_floats() {
        assert_eq!(format_value(23.45), "23.45");
        assert_eq!(format_value(0.01), "0.01");
    }
}