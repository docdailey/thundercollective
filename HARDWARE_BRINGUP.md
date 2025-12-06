# v0.3.0 – "First Light" Hardware Bring-up Checklist

**DO NOT write rdma.rs before completing this checklist.**

## Prerequisites

- 1× Sonnet Echo SE I T5 (TB5 PCIe enclosure)
- 1× Mellanox ConnectX-6 Dx 100GbE (MCX621102AC-ADAT)
- 1× OWC Thunderbolt 5 cable (0.8m, 240W)
- Linux box with kernel ≥6.6

## Bring-up Sequence

```bash
# 1. Physical setup
# Mac Studio ←TB5 cable→ Sonnet Enclosure ←PCIe→ ConnectX-6 ←ethernet→ Linux

# 2. Authorize Thunderbolt enclosure
sudo boltctl list
sudo boltctl authorize <uuid-of-sonnet>

# 3. Verify NIC detection
lspci | grep -i mellanox
# → must see "ConnectX-6 Dx"

# 4. Verify RDMA device
ibv_devices
# → must show mlx5_0 (or mlx5_1)

# 5. Install perftest
sudo apt install perftest   # Ubuntu/Debian
# or: dnf install perftest  # Fedora

# 6. Run bandwidth test - Server (terminal 1)
ib_write_bw -d mlx5_0 --report_gbits

# 7. Run bandwidth test - Client (terminal 2)
ib_write_bw -d mlx5_0 127.0.0.1 --report_gbits
```

## Expected Results

| Result | Action |
|--------|--------|
| ≥ 7.0 GB/s (56 Gbit/s) | 🍾 Champagne. Proceed to rdma.rs |
| 6.0–7.0 GB/s | Acceptable. Check for thermal throttling |
| 3.0–6.0 GB/s | Investigate. Check PCIe link width (x4 vs x8) |
| < 3.0 GB/s | STOP. Debug IOMMU, TB security, kernel flags |

## Troubleshooting

If bandwidth is low:

```bash
# Check PCIe link
sudo lspci -vv | grep -A 20 Mellanox | grep -i width
# Want: LnkSta: Width x4 (or x8)

# Check IOMMU
cat /proc/cmdline | grep iommu
# Try: intel_iommu=on iommu=pt (or amd_iommu=on)

# Check Thunderbolt security
sudo boltctl
# Ensure device is "authorized"

# Check for thermal throttling
sensors | grep -i mlx
```

## Only After Success

Once `ib_write_bw` shows 7+ GB/s:

1. Open PR for `src/fabric/rdma.rs`
2. Implement `RdmaFabric` using `async-rdma` crate
3. Add `GradientBucket<F: Fabric>` in `src/bucket.rs`
4. Benchmark against TCP baseline

**No Rust RDMA code until step 7 passes.**
