use nix::sys::statvfs::statvfs;
use serde::Serialize;
use std::fs;

#[derive(Debug, Clone, Serialize)]
pub struct DiskMetrics {
    pub mount_point: String,
    pub device: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
}

/// Read disk usage for all real filesystems.
pub fn read_disk_metrics() -> Result<Vec<DiskMetrics>, String> {
    let content = fs::read_to_string("/proc/mounts")
        .map_err(|e| format!("Failed to read /proc/mounts: {}", e))?;
    parse_and_stat_mounts(&content)
}

/// Parse /proc/mounts and call statvfs() on each real filesystem.
pub fn parse_and_stat_mounts(content: &str) -> Result<Vec<DiskMetrics>, String> {
    let mut disks = Vec::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let device = parts[0];
        let mount_point = parts[1];
        let fs_type = parts[2];

        // Only stat real block devices and tmpfs (which uses RAM)
        if !should_include(device, fs_type) {
            continue;
        }

        if let Ok(stat) = statvfs(mount_point) {
            let block_size = stat.block_size() as u64;
            let total = stat.blocks() * block_size;
            let free = stat.blocks_available() * block_size;
            let used = total.saturating_sub(free);

            disks.push(DiskMetrics {
                mount_point: mount_point.to_string(),
                device: device.to_string(),
                total_bytes: total,
                used_bytes: used,
                free_bytes: free,
            });
        }
    }

    Ok(disks)
}

/// Filter to real block devices and tmpfs. Skip virtual filesystems.
fn should_include(device: &str, fs_type: &str) -> bool {
    device.starts_with("/dev/") || fs_type == "tmpfs"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_include() {
        assert!(should_include("/dev/root", "ext4"));
        assert!(should_include("/dev/sda1", "ext4"));
        assert!(should_include("tmpfs", "tmpfs"));
        assert!(!should_include("proc", "proc"));
        assert!(!should_include("sysfs", "sysfs"));
        assert!(!should_include("devtmpfs", "devtmpfs"));
        assert!(!should_include("devpts", "devpts"));
    }
}