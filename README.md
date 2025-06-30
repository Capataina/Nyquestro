# Nyquestro

## Project Description
**Nyquestro** is a lock-free, limit-order-book matching engine written in safe Rust.  
The goal is to explore ultra-low-latency market-microstructure design by building every layer—from atomic price buckets to real-time market-data fan-out—without resorting to `unsafe` blocks or OS locks.

---

## Technologies & Dependencies

### **🦀 Core Technologies**
- **Rust 2024 Edition** – starting point; everything else will grow organically

### **📦 External Dependencies**
- _TBD_ – no external crates yet

---

## Features & Roadmap

### **🔧 Core Infrastructure & Foundations**
- [ ] Type-safe primitives – `OrderId`, `Side`, `Px`, `Qty`, `Ts`
- [ ] Flat-combining slab allocator – O(1) inserts/removes, epoch GC
- [ ] Atomic price buckets – intrusive FIFO lists per price level
- [ ] Deterministic matcher loop – price-time sweep with partial-fill handling
- [ ] Immutable event frames – zero-allocation structs for quotes & fills
- [ ] Engine error enum – recoverable vs fatal classifications
- [ ] Config loader – TOML / env with hot-reload signal

### **📡 Ingress & Market-Data Gateways**
- [ ] Ergonomic JSON CLI – local smoke tests, example payloads
- [ ] FIX 4.2 / 4.4 TCP acceptor – tag=value parser, heartbeat, resend logic
- [ ] Binary UDP gateway – little-endian framing, SO_REUSEPORT sharding
- [ ] ITCH-Lite multicast publisher – depth 0/1 snapshots & incremental updates
- [ ] WebSocket bridge – canned demo UI for workshops
- [ ] gRPC control plane – query stats, toggle risk checks

### **⚡ Order Management & Matching**
- [ ] Limit, Market, IOC, FOK support
- [ ] Cancel / Cancel-Replace – atomic modify flow
- [ ] Minimum tick-size enforcement
- [ ] Self-match prevention – “cancel oldest” / “cancel newest” policies
- [ ] Iceberg orders – displayed vs total quantity tracking
- [ ] Pegged orders – midpoint / last-sale peg logic
- [ ] Cross-asset matching – shard-aware book routing

### **🚀 Concurrency & Performance**
- [ ] SP/MC ring bus – cache-padded cursors, wait-free consumer pop
- [ ] NUMA-aware thread pinning – auto topology detect & affinity set
- [ ] Prefetch & branch-hint macros – `core::intrinsics::assume` helpers
- [ ] Batch cancel sweep – vectorised cancels with single CAS per level
- [ ] Lock-free free-list – recycled order nodes to avoid allocator churn
- [ ] AF_XDP ingress prototype – zero-copy packet RX path _(stretch)_
- [ ] SIMD price comparison – 4-wide compare in sweep inner loop _(stretch)_

### **🔒 Risk & Compliance Guard-Rails**
- [ ] Fat-finger limits – price & size deviation thresholds per session
- [ ] Position & PnL tracker – real-time inventory bounds
- [ ] Kill-switch VAR monitor – rolling variance window with circuit breaker
- [ ] Order throttles – per-IP / per-session rate caps
- [ ] Audit-trail journal – immutable append-only event log (JSON-lines)
- [ ] FIX drop-copy stream – outbound mirror for compliance

### **📊 Observability & Diagnostics**
- [ ] Structured tracing spans – microsecond-grained timing
- [ ] Latency HDR histogram – max, p99, p99.9 export
- [ ] Flamegraph scripts – one-liner `./scripts/profile.sh`
- [ ] Cache-miss counters – `perf stat` integration
- [ ] Hardware timestamp hooks – PTP-enabled NIC tracepoints _(stretch)_
- [ ] Prometheus metrics – gauges for depth, throughput, latency

### **🛠️ Bench & Test Harness**
- [ ] Synthetic order-flow replayer – Nasdaq L2 ITCH → engine feed
- [ ] Determinism test suite – replay vs golden output hash
- [ ] Property-based tests – QuickCheck on price-time ordering
- [ ] Fuzz harness – libFuzzer corpus for malformed FIX / UDP frames
- [ ] CI matrix – MSRV check, clippy, fmt, criterion micro-bench

### **🛡️ Security & Resilience**
- [ ] Input sanitisation – rigid frame length & field validation
- [ ] Memory-safety audit – `cargo-miri`, `cargo-tarpaulin`
- [ ] Crash-only design – idempotent recovery on restart
- [ ] Graceful shutdown – SIGTERM drains ring bus, flushes journals
- [ ] Process isolation guide – systemd unit & seccomp profile

### **🌐 Stretch Goals & Research Paths**
- [ ] WebAssembly back-tester – compile core to WASM, browser replay UI
- [ ] GPU order-book prototype – CUDA warp-level build experiment
- [ ] On-chain DEX adapter – Solana / Ethereum bridge demo
- [ ] eBPF uprobe telemetry – near-zero-overhead per-instruction heat maps
- [ ] Dynamic WASM risk plug-ins – hot-swap policy engine

### **🤝 DevOps & Community**
- [ ] GitHub Actions pipeline – clippy, fmt, benches, docker image
- [ ] Dual MIT / Apache-2 licence – invites commercial contribution
- [ ] GitHub Discussions & templates – bug, feature, question
- [ ] Code-owners & review rules – lock core crates, open gateway crates
- [ ] Crates.io publishing – `nyquestro-core`, `nyquestro-gw`, `nyquestro-proto`
- [ ] Docker compose demo – engine + WebSocket UI + Grafana dashboard
- [ ] Annotated blog series – data-structure deep dives, latency tricks
