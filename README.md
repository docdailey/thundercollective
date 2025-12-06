# thundercollective

**Ultrafast 2-node collectives for M3 Ultra + Thunderbolt 5 RDMA**
The fastest consumer-grade AI training fabric on earth in 2025.

```
Mac Studio #1 (M3 Ultra 512 GB) ←TB5→ Sonnet Echo SE I T5 + ConnectX-6 Dx ←→ Linux head node
Mac Studio #2 (M3 Ultra 512 GB) ←TB5→ Sonnet Echo SE I T5 + ConnectX-6 Dx ←→ Linux head node
```

Real-world target: **7.0 – 7.4 GB/s bidirectional all-reduce** over a single TB5 link
→ < 8 % comms overhead on 34B QLoRA, < 4 % on 70B sharded inference

## Current status (December 2025)

| Version | Status | What it does |
|--------|--------|--------------|
| `v0.1.0` | Frozen forever | Pure TCP 2-node ping-pong + ring-allreduce. Localhost baseline: ~3 GB/s |
| `v0.2.0` | **Current** | `Fabric` trait + `src/fabric/tcp.rs` |
| `v0.3.0` | Next | RDMA backend (`async-rdma` or raw `ibverbs`) + `GradientBucket` |
| `v0.4.0` | Future | Candle `ProcessGroupThunder` integration |

## Quick start

```bash
git clone https://github.com/docdailey/thundercollective
cd thundercollective
cargo build --release

# Terminal 1
./target/release/thundercollective --rank 0 --world-size 2 \
  --addr 127.0.0.1:5000 --addr 127.0.0.1:5001 --mode ping-pong

# Terminal 2
./target/release/thundercollective --rank 1 --world-size 2 \
  --addr 127.0.0.1:5000 --addr 127.0.0.1:5001 --mode ping-pong
```

## Modes

- `ping-pong` - Bidirectional bandwidth test (~3 GB/s TCP localhost)
- `allreduce` - In-place sum reduction across ranks (~1.2 GB/s TCP localhost)

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
- [x] v0.2.0 – `Fabric` trait abstraction
- [ ] v0.3.0 – RDMA backend + `GradientBucket<F: Fabric>` for batched all-reduce
- [ ] v0.4.0 – Candle integration (34B QLoRA, 70B sharded inference)
- [ ] v0.5.0 – Multi-link TB5 bonding (25+ GB/s)

## License

Apache-2.0 — fork it, ship it, make the world faster.

Built by people who got tired of waiting for Apple to give us RDMA.

Now go order the metal.
