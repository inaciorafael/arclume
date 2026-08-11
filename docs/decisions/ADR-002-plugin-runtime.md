# ADR-002: Plugin runtime

Status: proposed.

Preferred direction is WASM/WASI with capability-mediated host functions, optionally hosted in a separate process. JavaScript offers better immediate DX but a robust embedded sandbox is difficult; native Rust plugins have ABI, distribution and isolation costs; a process protocol gives isolation at higher memory and IPC cost.

No runtime will be selected before measuring startup, call latency, memory, cancellation and crash recovery.
