use crate::api::Market;

pub struct ScanSnapshot {
    pub longs: Vec<Market>,   // most negative funding → go long, collect
    pub shorts: Vec<Market>,  // most positive funding → go short, collect
    pub capital: f64,
    pub net_funding_daily: f64,
    pub net_funding_annual_pct: f64,
}

pub fn scan(markets: &[Market], k: usize, capital: f64) -> ScanSnapshot {
    let mut sorted = markets.to_vec();
    sorted.sort_by(|a, b| a.funding_rate.partial_cmp(&b.funding_rate).unwrap());

    let longs: Vec<Market> = sorted.iter().take(k).cloned().collect();
    let shorts: Vec<Market> = sorted.iter().rev().take(k).cloned().collect();

    let per_side = capital / k as f64;

    // Long leg: we're long, so we pay/receive -funding_rate per hour
    // When funding_rate < 0, long receives |funding_rate| per hour
    let long_hourly: f64 = longs.iter().map(|m| -m.funding_rate * per_side).sum();

    // Short leg: we're short, so we pay/receive +funding_rate per hour
    // When funding_rate > 0, short receives funding_rate per hour
    let short_hourly: f64 = shorts.iter().map(|m| m.funding_rate * per_side).sum();

    let net_hourly = long_hourly + short_hourly;
    let net_daily = net_hourly * 24.0;
    let net_annual_pct = net_daily * 365.0 / capital * 100.0;

    ScanSnapshot {
        longs,
        shorts,
        capital,
        net_funding_daily: net_daily,
        net_funding_annual_pct: net_annual_pct,
    }
}
