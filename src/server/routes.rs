use crate::metrics::{
    cpu::SharedCpuMetrics,
    disk::{self, DiskMetrics},
    memory::{self, MemoryMetrics},
    network::{self, InterfaceMetrics},
    system::{self, SystemMetrics},
};
use crate::server::prometheus;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};

type BoxBody = Full<Bytes>;

/// Route incoming requests to the appropriate handler.
pub async fn handle_request(
    req: Request<hyper::body::Incoming>,
    cpu_state: SharedCpuMetrics,
) -> Result<Response<BoxBody>, hyper::Error> {
    let response = match req.uri().path() {
        "/health" => health_handler(),
        "/metrics" => metrics_handler(cpu_state),
        "/json" => json_handler(cpu_state),
        _ => not_found(),
    };
    Ok(response)
}

/// GET /health — simple liveness check.
fn health_handler() -> Response<BoxBody> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .body(Full::new(Bytes::from("ok\n")))
        .unwrap()
}

/// GET /metrics — Prometheus exposition format.
fn metrics_handler(cpu_state: SharedCpuMetrics) -> Response<BoxBody> {
    let mut output = String::with_capacity(4096);

    // CPU metrics (from background sampler)
    if let Ok(guard) = cpu_state.lock() {
        prometheus::format_cpu(&guard, &mut output);
    }

    // Memory metrics (read on demand)
    match memory::read_memory_metrics() {
        Ok(mem) => prometheus::format_memory(&mem, &mut output),
        Err(e) => eprintln!("Memory metrics error: {}", e),
    }

    // System metrics (read on demand)
    match system::read_system_metrics() {
        Ok(sys) => prometheus::format_system(&sys, &mut output),
        Err(e) => eprintln!("System metrics error: {}", e),
    }

    // Network metrics (read on demand)
    match network::read_network_metrics() {
        Ok(interfaces) => prometheus::format_network(&interfaces, &mut output),
        Err(e) => eprintln!("Network metrics error: {}", e),
    }

    // Disk metrics (read on demand)
    match disk::read_disk_metrics() {
        Ok(disks) => prometheus::format_disk(&disks, &mut output),
        Err(e) => eprintln!("Disk metrics error: {}", e),
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
        .body(Full::new(Bytes::from(output)))
        .unwrap()
}

/// All metrics bundled for JSON serialization.
#[derive(serde::Serialize)]
struct AllMetrics {
    cpu: crate::metrics::cpu::CpuMetrics,
    memory: Option<MemoryMetrics>,
    system: Option<SystemMetrics>,
    network: Vec<InterfaceMetrics>,
    disk: Vec<DiskMetrics>,
}

/// GET /json — all metrics as a JSON object.
fn json_handler(cpu_state: SharedCpuMetrics) -> Response<BoxBody> {
    let cpu = cpu_state.lock().map(|g| g.clone()).unwrap_or_default();

    let all = AllMetrics {
        cpu,
        memory: memory::read_memory_metrics().ok(),
        system: system::read_system_metrics().ok(),
        network: network::read_network_metrics().unwrap_or_default(),
        disk: disk::read_disk_metrics().unwrap_or_default(),
    };

    let json = serde_json::to_string_pretty(&all).unwrap_or_else(|e| {
        format!(r#"{{"error":"{}"}}"#, e)
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap()
}

fn not_found() -> Response<BoxBody> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/plain")
        .body(Full::new(Bytes::from("not found\n")))
        .unwrap()
}