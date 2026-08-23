mod api;
mod display;
mod server;
mod strategy;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hyperfund")]
#[command(about = "Hyperliquid funding rate monitor & delta-neutral engine")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Show live funding rates (default)
    Rates {
        /// Number of top longs/shorts to show
        #[arg(short, long, default_value = "5")]
        k: usize,
        /// Minimum open interest in USD (filter thin markets)
        #[arg(long, default_value = "0")]
        min_oi: f64,
        /// Watch mode: refresh every N seconds
        #[arg(short, long)]
        watch: Option<u64>,
    },
    /// Serve funding signals as an x402-paid HTTP API
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Base address receiving USDC payments (0x…)
        #[arg(long)]
        pay_to: String,
    },
    /// Show delta-neutral strategy snapshot
    Scan {
        /// Long/short basket size
        #[arg(short, long, default_value = "5")]
        k: usize,
        /// Capital in USD (for yield estimate)
        #[arg(short, long, default_value = "1000")]
        capital: f64,
        /// Minimum open interest in USD (filter thin markets)
        #[arg(long, default_value = "5000000")]
        min_oi: f64,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Rates { k: 5, min_oi: 0.0, watch: None }) {
        Command::Rates { k, min_oi, watch } => {
            if let Some(secs) = watch {
                loop {
                    run_rates(k, min_oi).await?;
                    tokio::time::sleep(tokio::time::Duration::from_secs(secs)).await;
                    print!("\x1B[2J\x1B[1;1H");
                }
            } else {
                run_rates(k, min_oi).await?;
            }
        }
        Command::Scan { k, capital, min_oi } => {
            run_scan(k, capital, min_oi).await?;
        }
        Command::Serve { port, pay_to } => {
            server::serve(port, &pay_to).await?;
        }
    }

    Ok(())
}

async fn run_rates(k: usize, min_oi: f64) -> anyhow::Result<()> {
    let markets = api::fetch_markets().await?;
    let filtered: Vec<_> = markets.into_iter().filter(|m| m.open_interest >= min_oi).collect();
    display::print_rates(&filtered, k);
    Ok(())
}

async fn run_scan(k: usize, capital: f64, min_oi: f64) -> anyhow::Result<()> {
    let markets = api::fetch_markets().await?;
    let filtered: Vec<_> = markets.into_iter().filter(|m| m.open_interest >= min_oi).collect();
    let snapshot = strategy::scan(&filtered, k, capital);
    display::print_scan(&snapshot, min_oi);
    Ok(())
}
