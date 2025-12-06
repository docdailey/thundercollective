use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    rank: u32,
    #[arg(long)]
    world_size: u32,
    #[arg(long)]
    addr: Vec<SocketAddr>,
    #[arg(long, default_value = "ping-pong")]
    mode: String,
    #[arg(long, default_value_t = 64 * 1024 * 1024)]
    size: usize,
    #[arg(long, default_value_t = 1000)]
    iters: u32,
}

struct TcpTransport {
    rank: u32,
    world_size: u32,
    peers: Vec<TcpStream>,
}

impl TcpTransport {
    async fn new(rank: u32, world_size: u32, addrs: &[SocketAddr]) -> Result<Self> {
        assert_eq!(world_size, 2, "v0.1 only supports exactly 2 ranks");
        assert_eq!(world_size as usize, addrs.len());

        let mut peers = Vec::with_capacity(1);
        let listener = TcpListener::bind(addrs[rank as usize]).await?;

        for (i, &addr) in addrs.iter().enumerate() {
            let i = i as u32;
            if i == rank {
                continue;
            }

            let stream = if i > rank {
                let (s, _) = listener.accept().await?;
                s
            } else {
                TcpStream::connect(addr).await?
            };
            stream.set_nodelay(true)?;
            peers.push(stream);
        }

        Ok(Self {
            rank,
            world_size,
            peers,
        })
    }

    async fn send(&mut self, data: &[u8]) -> Result<()> {
        self.peers[0].write_all(data).await?;
        Ok(())
    }

    async fn recv(&mut self, buf: &mut [u8]) -> Result<()> {
        self.peers[0].read_exact(buf).await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut transport = TcpTransport::new(args.rank, args.world_size, &args.addr).await?;

    match args.mode.as_str() {
        "ping-pong" => {
            let mut buf = vec![0u8; args.size];
            if args.rank == 0 {
                buf.fill(0x42);
                let start = std::time::Instant::now();
                for _ in 0..args.iters {
                    transport.send(&buf).await?;
                    transport.recv(&mut buf).await?;
                }
                let elapsed = start.elapsed().as_secs_f64();
                let gbps = (args.size as f64 * args.iters as f64 * 2.0 / elapsed) / 1e9;
                println!(
                    "Ping-pong {} bytes x {} iters -> {:.2} GB/s",
                    args.size, args.iters, gbps
                );
            } else {
                for _ in 0..args.iters {
                    transport.recv(&mut buf).await?;
                    transport.send(&buf).await?;
                }
            }
        }

        "ring-allreduce" => {
            let mut buf = vec![args.rank as u8; args.size];
            let mut tmp = vec![0u8; args.size];

            let start = std::time::Instant::now();
            // world_size == 2 -> degenerates to ping-pong + local add
            for _ in 0..args.world_size {
                transport.send(&buf).await?;
                transport.recv(&mut tmp).await?;

                for (a, b) in buf.iter_mut().zip(tmp.iter()) {
                    *a = a.wrapping_add(*b);
                }
            }
            let elapsed = start.elapsed().as_secs_f64();
            println!(
                "rank {} ring-allreduce done in {:.3}s, final[0] = {}",
                args.rank, elapsed, buf[0]
            );
        }

        _ => eprintln!("unknown mode: {}", args.mode),
    }

    Ok(())
}
