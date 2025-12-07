# Launch Materials

Ready-to-use posts for sharing thundercollective.

---

## Hacker News

**Title (pick one):**
1. thundercollective – 2-node collectives for M3 Ultra + Thunderbolt 5 RDMA
2. Show HN: thundercollective – "poor man's NVLink" for deskside AI over TB5
3. Show HN: Thunderbolt 5 collectives for 2-node LLM training (thundercollective)

**Text:**

I've been slowly building what I wanted as a "deskside supercomputer" for LLM work: two Mac Studio M3 Ultras with 512 GB unified memory each, plus a cheap but fast interconnect that doesn't require a datacenter or Infiniband.

thundercollective is the first piece of that: a tiny Rust library that implements 2-node collectives (ping-pong and ring all-reduce) and is designed to run over:

- localhost (TCP) today
- Thunderbolt 5 + ConnectX-6 Dx RDMA tomorrow (same binary, different backend)

**Why this exists**

- M3 Ultra has amazing single-node throughput, but no NVLink and no eGPU support.
- TB5 gives you a ~80–120 Gbit fabric to something, you just have to terminate it in a NIC.
- I wanted a minimal, inspectable "poor man's NVLink" for 7–34B LLM training without renting a rack of A100s.

**What works right now (Dec 2025)**

v0.2.0 is live with a `Fabric` trait abstraction and TCP backend:

- ping-pong benchmark
- ring-allreduce implementation
- `--json` output for easy scripting:

```json
{"mode":"ping-pong","rank":0,"size_bytes":67108864,"iters":100,"elapsed_secs":4.58,"gbps":2.93}
```

On localhost TCP (baseline), I get:

```
Ping-pong 67108864 bytes × 100 → 2.9 GB/s
```

Target over TB5 RDMA: **7+ GB/s** (based on ib_write_bw benchmarks with CX-6)

**Hardware plan**

The idea is to terminate TB5 in a Mellanox/NVIDIA NIC and run RDMA over it:

- 2 × Sonnet Echo SE I T5 (Thunderbolt 5 → PCIe enclosure)
- 2 × ConnectX-6 Dx 100GbE (~$275 used on eBay)
- 2 × TB5 0.8m cables

Total ≈ $1.8k for the "fabric" between two hosts. There's a HARDWARE_BRINGUP.md in the repo that documents the kernel flags, boltctl/lspci/ibv_devices checks, and the ib_write_bw sanity benchmark.

**Roadmap**

- [x] v0.1.0 – 2-node TCP collectives, frozen forever
- [x] v0.2.0 – Fabric trait + TCP backend
- [ ] v0.3.0 – RDMA backend over TB5 (ConnectX-6 Dx in Sonnet enclosure)
- [ ] v0.4.0 – Candle ProcessGroupThunder to train 7–34B models across 2 nodes

**Repo:** https://github.com/docdailey/thundercollective

I'd love feedback from:

- People doing small-scale HPC / RDMA at home
- Folks who've already run CX-6 over Thunderbolt or USB4
- Anyone who wants a small, auditable building block instead of yet another giant framework

Right now this is deliberately tiny and "obviously correct." The goal is to get the fabric right, then plug it into real training loops.

---

## X / Bluesky / Mastodon

```
New toy: thundercollective – a tiny Rust library for 2-node LLM training over Thunderbolt 5.

✅ 2.9 GB/s ping-pong baseline (TCP localhost)
🎯 Target: 7+ GB/s over TB5 RDMA with ConnectX-6
🧱 v0.2.0 live with Fabric trait abstraction
🧵 Roadmap: RDMA backend → Candle integration → 34B QLoRA

Code + hardware BOM: https://github.com/docdailey/thundercollective
```

---

## LinkedIn (if you must)

**Building a "poor man's NVLink" for deskside AI training**

I've been working on thundercollective – a minimal Rust library for 2-node distributed training over Thunderbolt 5.

The goal: connect two Mac Studio M3 Ultras (512GB unified memory each = 1TB total) with a fast, cheap interconnect that doesn't require datacenter infrastructure.

Current status:
• 2.9 GB/s baseline over TCP
• Target: 7+ GB/s over TB5 RDMA
• Hardware cost: ~$1.8k for the full fabric

Why not just use cloud GPUs? Because 1TB of unified memory lets you fine-tune 405B models locally, and I'd rather own the hardware than rent it.

Open source (Apache-2.0): https://github.com/docdailey/thundercollective

#MachineLearning #Rust #DistributedSystems #AppleSilicon
