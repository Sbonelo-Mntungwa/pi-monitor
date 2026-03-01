// Metrics Collection Module
// =========================
//
// This module is responsible for reading system metrics from /proc and /sys.
// Each submodule handles one "domain" of metrics:
//
//   cpu.rs     — CPU usage percentages from /proc/stat
//   memory.rs  — Memory breakdown from /proc/meminfo
//   disk.rs    — Disk space from statfs() syscall
//   network.rs — Network counters from /proc/net/dev
//   system.rs  — Uptime, load averages, process count from /proc/loadavg & /proc/uptime
//
// Module Organization:
//   In Rust, a directory with mod.rs acts as the "public API" for that module.
//   This file declares which submodules exist and re-exports their public types
//   so that other parts of the code can do:
//     use crate::metrics::cpu::CpuMetrics;
//   instead of reaching deep into the module tree.

// We'll uncomment these as we build each collector.
// For now, just cpu.rs exists as a stub.

pub mod cpu;
// pub mod memory;
// pub mod disk;
// pub mod network;
// pub mod system;