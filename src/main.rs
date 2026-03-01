mod metrics;
mod server;

fn main() {
    println!("Pi-Monitor v{}", env!("CARGO_PKG_VERSION"));
    println!("Build target: {}", std::env::consts::ARCH);
    println!("Ready to monitor! (Stub — real server coming in Phase 3)");

    match std::fs::read_to_string("/proc/loadavg") {
        Ok(content) => println!("  /proc/loadavg: {}", content.trim()),
        Err(e) => println!("  /proc not available: {} (normal on macOS)", e),
    }
}
