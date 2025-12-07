# thundercollective

**Universal high-speed fabric for heterogeneous AI clusters**

A tiny Rust library that turns any RDMA-capable hardware into a unified training fabric. Started as "poor man's NVLink" for two Mac Studios, evolved into a hardware-agnostic collective that works across Apple Silicon, AMD, NVIDIA, and x86.

```
Mac Studio (M3 Ultra 512 GB) ←TB5→ Sonnet + CX-6 ←→ Linux head node
Framework 16 (AMD)           ←TB5→ Sonnet + CX-6 ←───┘
MI300X server                ←PCIe→ CX-7 400GbE  ←───┘
```

Real-world target: **7.0 – 7.4 GB/s** per TB5 link, **400 Gbit/s** on datacenter NICs
→ Same `Fabric` trait, same training code, any hardware

## Current status (December 2025)

| Version | Status | What it does |
|--------|--------|--------------|
| `v0.1.0` | Frozen forever | Pure TCP 2-node ping-pong + ring-allreduce. Localhost baseline: ~3 GB/s |
| `v0.2.0` | **Current** | `Fabric` trait + multi-stream TCP striping (4.3 GB/s with `--num-streams 4`) |
| `v0.3.0` | Next | RDMA backend (`async-rdma` or raw `ibverbs`) + `GradientBucket` |
| `v0.4.0` | Future | Candle `ProcessGroupThunder` integration |

## Quick start

```bash
git clone https://github.com/docdailey/thundercollective
cd thundercollective
cargo build --release

# Terminal 1 (use --num-streams 4 for higher throughput)
./target/release/thundercollective --rank 0 --world-size 2 \
  --addr 127.0.0.1:5000 --addr 127.0.0.1:5010 --mode ping-pong --num-streams 4

# Terminal 2
./target/release/thundercollective --rank 1 --world-size 2 \
  --addr 127.0.0.1:5000 --addr 127.0.0.1:5010 --mode ping-pong --num-streams 4
```

Note: With `--num-streams N`, rank 0 listens on ports 5000, 5001, ..., 5000+N-1. Leave a gap between rank addresses (e.g., 5000 and 5010) to avoid port collisions.

## Modes

- `ping-pong` - Bidirectional bandwidth test
- `allreduce` - In-place sum reduction across ranks

## Benchmarks (localhost, 64 MB × 50 iters)

| Mode | Streams | Throughput |
|------|---------|------------|
| ping-pong | 1 | 2.8 GB/s |
| ping-pong | 4 | **4.3 GB/s** |
| allreduce | 4 | 1.8 GB/s |

On CX-6 Ethernet mode (TB5), expect **5.5–6 GB/s**. Full RDMA (v0.3.0) targets **7+ GB/s**.

## Hardware BOM – "7 GB/s kit" (Dec 2025 prices)

| Qty | Item                                    | Price (USD) | Link |
|-----|-----------------------------------------|-------------|------|
| 2   | Sonnet Echo SE I T5 (TB5 PCIe enclosure)| $449        | [sonnetstore.com](https://www.sonnetstore.com/products/echo-se-i-t5) |
| 2   | Mellanox ConnectX-6 Dx 100GbE dual-port | ~$275 used  | eBay "MCX621102AC-ADAT" |
| 2   | OWC Thunderbolt 5 cable 0.8 m 240 W     | $129        | [owc.com](https://www.owc.com/solutions/thunderbolt-5-cable) |
| **Total** |                                      | **~$1,836** | |

Start with **one** of each (~$918) to validate before duplicating.

## Architecture

```
src/
├── main.rs           # CLI + benchmarks
└── fabric/
    ├── mod.rs        # Fabric trait definition
    ├── tcp.rs        # TCP backend (baseline)
    └── rdma.rs       # RDMA backend (v0.3.0)
```

The `Fabric` trait abstracts transport:

```rust
#[async_trait]
pub trait Fabric: Send + Sync {
    async fn send(&self, peer: usize, buf: &[u8]) -> Result<()>;
    async fn recv(&self, peer: usize, buf: &mut [u8]) -> Result<usize>;
    async fn allreduce(&self, buf: &mut [u8], op: ReduceOp) -> Result<()>;
    fn rank(&self) -> usize;
    fn world_size(&self) -> usize;
}
```

Swap `TcpFabric` for `RdmaFabric` — same API, 2x+ bandwidth.

## Roadmap

- [x] v0.1.0 – TCP baseline (frozen)
- [x] v0.2.0 – `Fabric` trait abstraction + multi-stream TCP striping
- [ ] v0.3.0 – RDMA backend + `GradientBucket<F: Fabric>` for batched all-reduce
- [ ] v0.4.0 – Candle integration (34B QLoRA, 70B sharded inference)
- [ ] v0.5.0 – Multi-link TB5 bonding (25+ GB/s)

## Design Notes

**Why 2-node?**
Two M3 Ultra Mac Studios with 512GB unified memory each = 1TB total. That's enough to fine-tune 405B models or run 1T+ MoE inference. More nodes add complexity without proportional benefit at this scale.

**Why Thunderbolt 5?**
TB5 gives 80 Gbps (10 GB/s theoretical) with PCIe tunneling. Unlike USB4/TB4, it can actually saturate a ConnectX-6 NIC. No enterprise switches, no rack, no electrician.

**Why RDMA over IP bonding?**
RDMA (via ibverbs) bypasses the kernel network stack entirely. Zero-copy, kernel-bypass, hardware offload. IP bonding caps at ~3 GB/s with CPU overhead; RDMA hits 7+ GB/s with negligible CPU.

**Why not just use NCCL?**
NCCL assumes NVIDIA GPUs with NVLink/InfiniBand. Apple Silicon has neither. We need a from-scratch collective that speaks Metal + RDMA.

## Supported Hardware

The `Fabric` trait is hardware-agnostic. Any machine with an RDMA-capable NIC can join:

| Machine | Connection | Expected Bandwidth |
|---------|------------|-------------------|
| Mac Studio / MacBook Pro (M3/M4 Ultra) | TB5 → Sonnet + CX-6 → Linux | 7.0 – 7.4 GB/s |
| Mac mini (M4 Pro) | TB5 → Sonnet + CX-6 → Linux | 7+ GB/s |
| Framework Laptop 16 (AMD) | TB5 → Sonnet + CX-6 or direct PCIe | 7+ GB/s |
| Any x86 Linux desktop/server | Direct PCIe slot + CX-6/7 | 7 – 200+ GB/s |
| AMD MI300X / MI325X | Native 400 Gbit RoCEv2 | 400 Gbit/s |
| NVIDIA Grace Hopper | NVLink + CX-7 | 900 GB/s NVLink domain |

**Mix and match.** Apple for inference, AMD for training, NVIDIA for whatever—all on the same fabric:

```rust
// This runs on ANY of the above machines
let fabric: Box<dyn Fabric> = match backend {
    "tcp"  => Box::new(TcpFabric::new(...).await?),
    "rdma" => Box::new(RdmaFabric::new(...).await?),
};

fabric.allreduce(&mut gradients, ReduceOp::Sum).await?;
```

Same training loop. Same `GradientBucket`. Different metal.

## Hardware Bring-up

Wiring TB5 + ConnectX-6 for the first time? See [HARDWARE_BRINGUP.md](./HARDWARE_BRINGUP.md) for the step-by-step checklist.

**Do not write RDMA code until `ib_write_bw` shows 7+ GB/s.**

## License

Apache-2.0 — fork it, ship it, make the world faster.

Built by people who got tired of waiting for Apple to give us RDMA.

Now go order the metal.
