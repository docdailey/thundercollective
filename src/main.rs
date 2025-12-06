mod fabric;

use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;

use fabric::{tcp::TcpFabric, Fabric, ReduceOp};

#[derive(Parser, Debug)]
#[command(name = "thundercollective")]
#[command(about = "Ultrafast 2-node collectives for M3 Ultra + Thunderbolt 5")]
struct Args {
    #[arg(long)]
    rank: usize,
    #[arg(long)]
    world_size: usize,
    #[arg(long)]
    addr: Vec<SocketAddr>,
    #[arg(long, default_value = "ping-pong")]
    mode: String,
    #[arg(long, default_value_t = 64 * 1024 * 1024)]
    size: usize,
    #[arg(long, default_value_t = 1000)]
    iters: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Create fabric - swap TcpFabric for RdmaFabric when hardware arrives
    let fabric = TcpFabric::new(args.rank, args.world_size, &args.addr).await?;

    match args.mode.as_str() {
        "ping-pong" => run_ping_pong(&fabric, &args).await?,
        "allreduce" => run_allreduce(&fabric, &args).await?,
        _ => eprintln!("unknown mode: {}. Use 'ping-pong' or 'allreduce'", args.mode),
    }

    Ok(())
}

async fn run_ping_pong<F: Fabric>(fabric: &F, args: &Args) -> Result<()> {
    let mut buf = vec![0u8; args.size];
    let peer = if fabric.rank() == 0 { 1 } else { 0 };

    if fabric.rank() == 0 {
        buf.fill(0x42);
        let start = std::time::Instant::now();
        for _ in 0..args.iters {
            fabric.send(peer, &buf).await?;
            fabric.recv(peer, &mut buf).await?;
        }
        let elapsed = start.elapsed().as_secs_f64();
        let gbps = (args.size as f64 * args.iters as f64 * 2.0 / elapsed) / 1e9;
        println!(
            "Ping-pong {} bytes x {} iters -> {:.2} GB/s",
            args.size, args.iters, gbps
        );
    } else {
        for _ in 0..args.iters {
            fabric.recv(peer, &mut buf).await?;
            fabric.send(peer, &buf).await?;
        }
        println!("rank {} ping-pong complete", fabric.rank());
    }

    Ok(())
}

async fn run_allreduce<F: Fabric>(fabric: &F, args: &Args) -> Result<()> {
    let mut buf = vec![fabric.rank() as u8; args.size];

    let start = std::time::Instant::now();
    for _ in 0..args.iters {
        fabric.allreduce(&mut buf, ReduceOp::Sum).await?;
    }
    let elapsed = start.elapsed().as_secs_f64();

    let gbps = (args.size as f64 * args.iters as f64 * 2.0 / elapsed) / 1e9;
    println!(
        "rank {} allreduce {} bytes x {} iters -> {:.2} GB/s, final[0] = {}",
        fabric.rank(),
        args.size,
        args.iters,
        gbps,
        buf[0]
    );

    Ok(())
}
