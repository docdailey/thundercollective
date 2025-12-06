use anyhow::Result;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReduceOp {
    Sum,
    // Max, Min, Prod can come later
}

/// Core abstraction for network transport.
/// Implementations can be TCP (baseline), RDMA (production), or shared memory (testing).
#[async_trait::async_trait]
pub trait Fabric: Send + Sync {
    /// Send a message to peer (by rank)
    async fn send(&self, peer: usize, buf: &[u8]) -> Result<()>;

    /// Receive message from peer into buf, returns bytes received
    async fn recv(&self, peer: usize, buf: &mut [u8]) -> Result<usize>;

    /// In-place all-reduce across all ranks
    async fn allreduce(&self, buf: &mut [u8], op: ReduceOp) -> Result<()>;

    /// This process's rank
    fn rank(&self) -> usize;

    /// Total number of processes
    fn world_size(&self) -> usize;
}

pub mod tcp;

#[cfg(feature = "rdma")]
pub mod rdma;
