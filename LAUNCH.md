# Launch Manifest: Thunder Collective v0.3

**Status:** HARDWARE BRING-UP IN PROGRESS
**Date:** December 2025
**Objective:** Validate 7.0+ GB/s RDMA link between Apple Silicon and Linux Head Node via Thunderbolt 5.

---

## 1. System Configuration

| Node | Hardware | OS | Interface | Role |
|:-----|:---------|:---|:----------|:-----|
| **Rank 0** | Mac Studio (M3 Ultra) | macOS Sequoia / Asahi | TB5 → Sonnet → CX-6 | Inference / Shard |
| **Rank 1** | Linux Head Node | Ubuntu 24.04 | PCIe Gen4 x16 → CX-6 | Training / Gateway |
| **Interconnect** | Thunderbolt 5 (80 Gbps) tunneling PCIe Gen4 x4 |

---

## 2. Phase I: Physical Link Integrity

**Goal:** Confirm TB5 negotiation at full PCIe Gen4 x4 width.

- [ ] **LSPCI Check (Mac/Asahi Side)**
  - *Command:* `sudo lspci -vv | grep -A 20 "Mellanox" | grep "LnkSta"`
  - *Target:* `Speed 16GT/s, Width x4`
  - *Actual:* `[WAITING FOR HARDWARE]`

- [ ] **boltctl Authorization**
  - *Command:* `boltctl list` / `boltctl authorize <UUID>`
  - *Status:* `[WAITING FOR HARDWARE]`

> **Note:** If Width is x1 or Speed is 8GT/s, reseat the OWC cable or check Sonnet enclosure power.

---

## 3. Phase II: The "TCP Tax" (macOS Native)

**Goal:** Saturate the link using v0.2.0 multi-stream TCP to prove the wire works before attempting RDMA.

- [ ] **Throughput Test (4-Stream Stripe)**
  - *Command:* `./target/release/thundercollective --mode ping-pong --num-streams 4`
  - *Target:* > 5.5 GB/s (44 Gbps)
  - *Result:* `[WAITING FOR LOGS]`

```text
[PLACEHOLDER FOR TERMINAL OUTPUT]
Rank 0: Connected.
Rank 1: Connected.
Benchmarking...
...
Result: 5.82 GB/s (Saturated)
```

---

## 4. Phase III: The Escape Hatch (RDMA / Asahi)

**Goal:** Bypass the kernel. The main event.

- [ ] **Verbs Device Check**
  - *Command:* `ibv_devinfo`
  - *Target:* `transport: InfiniBand (0)` or `transport: Ethernet (0)`
  - *Status:* `PORT_ACTIVE`

- [ ] **Bandwidth Test (ib_write_bw)**
  - *Command:* `ib_write_bw -F --report_gbits -q 4 <PEER_IP>`
  - *Target:* > 55 Gbps (Raw) / > 7.0 GB/s (Payload)
  - *Result:* `[WAITING FOR SCREENSHOT]`

> **Success Criteria:** If this hits >6.8 GB/s, we have successfully created the world's fastest Thunderbolt networking stack.

---

## 5. Visual Proof

- **Photo:** The Rig (Mac Studio + Sonnet Echo + Glowing CX-6 LEDs)
  - `[INSERT PHOTO]`
- **Screenshot:** `htop` on Linux showing 0% CPU usage during 7 GB/s transfer (RDMA magic)
  - `[INSERT SCREENSHOT]`

---

## 6. Verdict

- [ ] **PASS:** Hardware validated. Merge v0.3 to `master`.
- [ ] **FAIL:** Debug checklist:
  - Cable issue? Try different TB5 cable
  - PCIe negotiation? Check `lspci` width/speed
  - Driver issue? Check `dmesg | grep mlx`

**Next Steps:** Ship `GradientBucket` implementation and Candle integration.

---

---

# Social Media Launch Materials

Ready-to-use posts for sharing thundercollective after v0.3 ships.

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

v0.2.0 is live with a `Fabric` trait abstraction and multi-stream TCP backend:

- ping-pong benchmark: **4.3 GB/s** with `--num-streams 4`
- ring-allreduce implementation
- `--json` output for easy scripting

Target over TB5 RDMA: **7+ GB/s** (based on ib_write_bw benchmarks with CX-6)

**Hardware plan**

The idea is to terminate TB5 in a Mellanox/NVIDIA NIC and run RDMA over it:

- 2 × Sonnet Echo SE I T5 (Thunderbolt 5 → PCIe enclosure)
- 2 × ConnectX-6 Dx 100GbE (~$275 used on eBay)
- 2 × TB5 0.8m cables

Total ≈ $1.8k for the "fabric" between two hosts. There's a HARDWARE_BRINGUP.md in the repo that documents the kernel flags, boltctl/lspci/ibv_devices checks, and the ib_write_bw sanity benchmark.

**Roadmap**

- [x] v0.1.0 – 2-node TCP collectives, frozen forever
- [x] v0.2.0 – Fabric trait + multi-stream TCP (4.3 GB/s)
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

✅ 4.3 GB/s ping-pong (multi-stream TCP)
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
• 4.3 GB/s over multi-stream TCP (v0.2.0)
• Target: 7+ GB/s over TB5 RDMA
• Hardware cost: ~$1.8k for the full fabric

Why not just use cloud GPUs? Because 1TB of unified memory lets you fine-tune 405B models locally, and I'd rather own the hardware than rent it.

Open source (Apache-2.0): https://github.com/docdailey/thundercollective

#MachineLearning #Rust #DistributedSystems #AppleSilicon
