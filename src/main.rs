mod metrics;
mod server;

use metrics::cpu;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let port = 9100;
    println!("Pi-Monitor v{}", env!("CARGO_PKG_VERSION"));

    // Start CPU background sampler
    let cpu_state = cpu::new_shared_metrics();
    tokio::spawn(cpu::cpu_sampling_task(cpu_state.clone(), Duration::from_secs(2)));

    println!("Listening on 0.0.0.0:{}", port);
    if let Err(e) = server::run(port, cpu_state).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}