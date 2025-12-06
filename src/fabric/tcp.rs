use anyhow::{anyhow, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use super::{Fabric, ReduceOp};

/// TCP-based fabric implementation.
/// This is the baseline for testing - works everywhere, ~3 GB/s on localhost.
/// RDMA backend will be a drop-in replacement hitting 7+ GB/s.
pub struct TcpFabric {
    rank: usize,
    world_size: usize,
    /// Peers indexed by rank (own rank slot is None)
    peers: Vec<Option<Arc<Mutex<TcpStream>>>>,
}

impl TcpFabric {
    /// Create a new TCP fabric.
    /// All ranks must call this simultaneously with the same addrs list.
    pub async fn new(rank: usize, world_size: usize, addrs: &[SocketAddr]) -> Result<Self> {
        if world_size != 2 {
            return Err(anyhow!("v0.2 only supports exactly 2 ranks"));
        }
        if addrs.len() != world_size {
            return Err(anyhow!(
                "addrs.len() {} != world_size {}",
                addrs.len(),
                world_size
            ));
        }

        let listener = TcpListener::bind(addrs[rank]).await?;
        let mut peers: Vec<Option<Arc<Mutex<TcpStream>>>> = vec![None; world_size];

        for i in 0..world_size {
            if i == rank {
                continue;
            }

            let stream = if i > rank {
                // Accept from higher ranks
                let (s, _) = listener.accept().await?;
                s
            } else {
                // Connect to lower ranks
                TcpStream::connect(addrs[i]).await?
            };
            stream.set_nodelay(true)?;
            peers[i] = Some(Arc::new(Mutex::new(stream)));
        }

        Ok(Self {
            rank,
            world_size,
            peers,
        })
    }

    fn get_peer(&self, peer: usize) -> Result<Arc<Mutex<TcpStream>>> {
        self.peers
            .get(peer)
            .and_then(|p| p.clone())
            .ok_or_else(|| anyhow!("invalid peer rank: {}", peer))
    }
}

#[async_trait::async_trait]
impl Fabric for TcpFabric {
    async fn send(&self, peer: usize, buf: &[u8]) -> Result<()> {
        let stream = self.get_peer(peer)?;
        let mut guard = stream.lock().await;
        guard.write_all(buf).await?;
        Ok(())
    }

    async fn recv(&self, peer: usize, buf: &mut [u8]) -> Result<usize> {
        let stream = self.get_peer(peer)?;
        let mut guard = stream.lock().await;
        guard.read_exact(buf).await?;
        Ok(buf.len())
    }

    async fn allreduce(&self, buf: &mut [u8], op: ReduceOp) -> Result<()> {
        // For world_size == 2, this degenerates to:
        // 1. Exchange buffers with peer (rank 0 sends first, rank 1 recvs first)
        // 2. Apply reduction locally

        let peer = if self.rank == 0 { 1 } else { 0 };
        let mut tmp = vec![0u8; buf.len()];

        // Avoid deadlock: rank 0 sends then recvs, rank 1 recvs then sends
        if self.rank == 0 {
            self.send(peer, buf).await?;
            self.recv(peer, &mut tmp).await?;
        } else {
            self.recv(peer, &mut tmp).await?;
            self.send(peer, buf).await?;
        }

        // Apply reduction
        match op {
            ReduceOp::Sum => {
                for (a, b) in buf.iter_mut().zip(tmp.iter()) {
                    *a = a.wrapping_add(*b);
                }
            }
        }

        Ok(())
    }

    fn rank(&self) -> usize {
        self.rank
    }

    fn world_size(&self) -> usize {
        self.world_size
    }
}
