pub mod prometheus;
pub mod routes;

use crate::metrics::cpu::SharedCpuMetrics;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Start the HTTP server on the given port. Runs until the process exits.
pub async fn run(port: u16, cpu_state: SharedCpuMetrics) -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _remote_addr) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let cpu = cpu_state.clone();

        // Spawn a new task per connection so one slow client doesn't block others
        tokio::spawn(async move {
            let service = service_fn(move |req| {
                let cpu = cpu.clone();
                async move { routes::handle_request(req, cpu).await }
            });

            if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }
}