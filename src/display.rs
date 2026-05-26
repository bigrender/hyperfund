use colored::Colorize;
use crate::api::Market;
use crate::strategy::ScanSnapshot;

const SEP: &str = "─────────────────────────────────────────────────────────────";

pub fn print_rates(markets: &[Market], k: usize) {
    let mut sorted = markets.to_vec();
    sorted.sort_by(|a, b| a.funding_rate.partial_cmp(&b.funding_rate).unwrap());

    let ts = chrono_now();
    println!();
    println!("{}", "  HYPERFUND  ·  Hyperliquid Funding Rates".bold().white());
    println!("  {}", ts.dimmed());
    println!("  {}", SEP.dimmed());

    println!();
    println!("{}", "  LONG CANDIDATES  (negative funding → you collect)".green().bold());
    println!("  {:<10} {:>12} {:>12} {:>14}", "COIN", "RATE/HR", "APR", "OI (USD)");
    println!("  {}", SEP.dimmed());

    let longs: Vec<_> = sorted.iter()
        .filter(|m| m.funding_rate < 0.0)
        .take(k)
        .collect();

    if longs.is_empty() {
        println!("  {}", "(없음)".dimmed());
    }
    for m in &longs {
        println!("  {:<10} {:>12} {:>12} {:>14}",
            m.coin.green().bold(),
            format!("{:+.4}%", m.funding_rate * 100.0).green(),
            format!("{:+.1}%", m.funding_annual()).green(),
            format_oi(m.open_interest).white(),
        );
    }

    println!();
    println!("{}", "  SHORT CANDIDATES  (positive funding → you collect)".red().bold());
    println!("  {:<10} {:>12} {:>12} {:>14}", "COIN", "RATE/HR", "APR", "OI (USD)");
    println!("  {}", SEP.dimmed());

    let shorts: Vec<_> = sorted.iter().rev()
        .filter(|m| m.funding_rate > 0.0)
        .take(k)
        .collect();

    if shorts.is_empty() {
        println!("  {}", "(없음)".dimmed());
    }
    for m in &shorts {
        println!("  {:<10} {:>12} {:>12} {:>14}",
            m.coin.red().bold(),
            format!("{:+.4}%", m.funding_rate * 100.0).red(),
            format!("{:+.1}%", m.funding_annual()).red(),
            format_oi(m.open_interest).white(),
        );
    }

    println!();
    println!("  {}", SEP.dimmed());
    println!("  {} markets · use {} for delta-neutral scan",
        markets.len().to_string().white().bold(),
        "hyperfund scan".cyan()
    );
    println!();
}

pub fn print_scan(s: &ScanSnapshot, min_oi: f64) {
    let ts = chrono_now();
    println!();
    println!("{}", "  HYPERFUND  ·  Delta-Neutral Scan".bold().white());
    println!("  {}", ts.dimmed());
    let oi_str = if min_oi >= 1_000_000.0 { format!("min OI: {}M", min_oi as u64 / 1_000_000) } else { "no OI filter".to_string() };
    println!("  capital: ${:.0}  |  K={}  |  {}", s.capital, s.longs.len(), oi_str.dimmed());
    println!("  {}", SEP.dimmed());

    println!();
    println!("{}", "  LONG BASKET".green().bold());
    println!("  {:<10} {:>12} {:>12}", "COIN", "RATE/HR", "APR");
    println!("  {}", SEP.dimmed());
    for m in &s.longs {
        println!("  {:<10} {:>12} {:>12}",
            m.coin.green().bold(),
            format!("{:+.4}%", m.funding_rate * 100.0).green(),
            format!("{:+.1}%", m.funding_annual()).green(),
        );
    }

    println!();
    println!("{}", "  SHORT BASKET".red().bold());
    println!("  {:<10} {:>12} {:>12}", "COIN", "RATE/HR", "APR");
    println!("  {}", SEP.dimmed());
    for m in &s.shorts {
        println!("  {:<10} {:>12} {:>12}",
            m.coin.red().bold(),
            format!("{:+.4}%", m.funding_rate * 100.0).red(),
            format!("{:+.1}%", m.funding_annual()).red(),
        );
    }

    println!();
    println!("  {}", SEP.dimmed());
    let daily_str = format!("${:+.2}/day", s.net_funding_daily);
    let annual_str = format!("{:+.1}% APR", s.net_funding_annual_pct);

    println!("  {} {}  |  {}",
        "Estimated funding yield:".white().bold(),
        if s.net_funding_daily >= 0.0 { daily_str.green().bold() } else { daily_str.red().bold() },
        if s.net_funding_annual_pct >= 0.0 { annual_str.green() } else { annual_str.red() },
    );
    println!("  {} (price residual excluded — run backtest for full picture)",
        "⚠  funding only".yellow()
    );
    println!();
}

fn format_oi(oi: f64) -> String {
    if oi >= 1_000_000_000.0 {
        format!("${:.1}B", oi / 1_000_000_000.0)
    } else if oi >= 1_000_000.0 {
        format!("${:.1}M", oi / 1_000_000.0)
    } else if oi >= 1_000.0 {
        format!("${:.1}K", oi / 1_000.0)
    } else {
        format!("${:.0}", oi)
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02} UTC", h, m, s)
}
