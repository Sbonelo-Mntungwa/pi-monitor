mod metrics;
mod server;

fn main() {
    println!("Pi-Monitor v{}", env!("CARGO_PKG_VERSION"));
    println!("Build target: {}", std::env::consts::ARCH);
    println!("Ready to monitor! (Stub — real server coming in Phase 3)");

    // Quick smoke test: try to read /proc/loadavg to verify /proc is accessible
    match std::fs::read_to_string("/proc/loadavg") {
        Ok(content) => {
            println!("  /proc/loadavg: {}", content.trim());
            println!("  /proc filesystem is accessible — metrics collection will work!");
        }
        Err(e) => {
            // This is expected if running on macOS (which has no /proc).
            // It will succeed on the VM and on the Pi.
            println!("  /proc/loadavg not available: {}", e);
            println!("  (This is normal on macOS — will work on Linux/Pi)");
        }
    }
}