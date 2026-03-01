// HTTP Server Module
// ===================
//
// This module will implement the HTTP server that serves metrics.
//
// Phase 3 will implement:
//   - Tokio-based async HTTP server using hyper
//   - Route dispatch: GET /metrics, /json, /health, /
//   - Prometheus exposition format output
//   - JSON output via serde
//   - Embedded HTML dashboard
//
// Submodules:
//   routes.rs      — Request routing and handler functions
//   prometheus.rs  — Formatting metrics in Prometheus text format

// We'll uncomment these as we build them in Phase 3.
// pub mod routes;
// pub mod prometheus;
