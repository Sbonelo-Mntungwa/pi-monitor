use serde::Serialize;
use std::fs;

#[derive(Debug, Clone, Serialize)]
pub struct InterfaceMetrics {
    pub name: String,
    pub rx_bytes: u64,
    pub rx_packets: u64,
    pub rx_errors: u64,
    pub rx_dropped: u64,
    pub tx_bytes: u64,
    pub tx_packets: u64,
    pub tx_errors: u64,
    pub tx_dropped: u64,
}

/// Read and parse /proc/net/dev for all network interfaces.
pub fn read_network_metrics() -> Result<Vec<InterfaceMetrics>, String> {
    let content = fs::read_to_string("/proc/net/dev")
        .map_err(|e| format!("Failed to read /proc/net/dev: {}", e))?;
    parse_net_dev(&content)
}

/// Parse /proc/net/dev content. Skips the 2-line header.
pub fn parse_net_dev(content: &str) -> Result<Vec<InterfaceMetrics>, String> {
    let interfaces: Vec<InterfaceMetrics> = content
        .lines()
        .filter_map(parse_interface_line)
        .collect();
    Ok(interfaces)
}

/// Parse a single interface line like "  eth0:    7065      56  166 ..."
fn parse_interface_line(line: &str) -> Option<InterfaceMetrics> {
    let (name, stats) = line.split_once(':')?;
    let name = name.trim().to_string();

    let vals: Vec<u64> = stats.split_whitespace().filter_map(|s| s.parse().ok()).collect();
    if vals.len() < 16 {
        return None;
    }

    // /proc/net/dev columns (per the kernel docs):
    // Receive:  bytes packets errs drop fifo frame compressed multicast
    // Transmit: bytes packets errs drop fifo colls carrier compressed
    Some(InterfaceMetrics {
        name,
        rx_bytes: vals[0],
        rx_packets: vals[1],
        rx_errors: vals[2],
        rx_dropped: vals[3],
        tx_bytes: vals[8],
        tx_packets: vals[9],
        tx_errors: vals[10],
        tx_dropped: vals[11],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_NET_DEV: &str = "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo:       0       0    0    0    0     0          0         0        0       0    0    0    0     0       0          0
  eth0:    7065      56  166    0    0     0          0         0     3672      24    0    0    0     0       0          0";

    #[test]
    fn test_parse_real_net_dev() {
        let interfaces = parse_net_dev(SAMPLE_NET_DEV).unwrap();
        assert_eq!(interfaces.len(), 2);

        let lo = &interfaces[0];
        assert_eq!(lo.name, "lo");
        assert_eq!(lo.rx_bytes, 0);
        assert_eq!(lo.tx_bytes, 0);

        let eth0 = &interfaces[1];
        assert_eq!(eth0.name, "eth0");
        assert_eq!(eth0.rx_bytes, 7065);
        assert_eq!(eth0.rx_packets, 56);
        assert_eq!(eth0.rx_errors, 166);
        assert_eq!(eth0.tx_bytes, 3672);
        assert_eq!(eth0.tx_packets, 24);
    }

    #[test]
    fn test_header_lines_skipped() {
        let interfaces = parse_net_dev(SAMPLE_NET_DEV).unwrap();
        for iface in &interfaces {
            assert!(!iface.name.contains("face"));
            assert!(!iface.name.contains("Inter"));
        }
    }
}