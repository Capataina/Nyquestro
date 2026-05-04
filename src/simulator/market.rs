//! Configurable synthetic order-flow generator.

use rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::order::Order;
use crate::types::{OrderID, Px, Qty, Side, Symbol, Ts};

// ─── SimAction ──────────────────────────────────────────────────────────────

/// One step of simulator (or feed-bridge) output. The dashboard treats
/// these as the canonical input to the matching engine, regardless of
/// whether they came from synthetic flow or a live data feed.
#[derive(Debug, Clone)]
pub enum SimAction {
    Submit(Order),
    /// "Now would be a good time to cancel something" — caller picks the
    /// actual id from its own resting cache. Used by the synthetic
    /// simulator only.
    CancelHint,
    /// Explicit cancel by id and symbol. Used by the feed bridge to
    /// retract virtual level-orders when a Coinbase L2 update reports a
    /// level cleared or quantity-changed.
    Cancel { symbol: Symbol, order_id: OrderID },
}

#[derive(Debug, Clone)]
pub struct SimConfig {
    /// Symbol this simulator emits orders for.
    pub symbol: Symbol,
    /// Theoretical fair value the OU walk reverts to (in cents).
    pub fair_value_cents: u64,
    /// Mean reversion strength θ (per second). 0.5 = half-life ~1.4s.
    pub theta: f64,
    /// Diffusion σ (cents per √second).
    pub sigma_cents: f64,
    /// Tick size (cents). Prices snap to multiples of this.
    pub tick_cents: u64,
    /// Limit-order arrival intensity per side (events per second).
    pub limit_lambda: f64,
    /// Market-order arrival intensity per side (events per second).
    pub market_lambda: f64,
    /// Cancellation intensity per side (events per second).
    pub cancel_lambda: f64,
    /// Log-normal mean of order size (in log space).
    pub size_log_mean: f64,
    /// Log-normal stddev of order size.
    pub size_log_sigma: f64,
    /// Maximum order size (clipping cap).
    pub size_max: u32,
    /// Distance-decay exponent α; controls how fast intensity drops with
    /// distance from the touch.
    pub price_alpha: f64,
    /// Maximum ticks of price-displacement to consider for limit orders.
    pub price_max_ticks: u32,
}

impl Default for SimConfig {
    fn default() -> Self {
        SimConfig {
            symbol: Symbol::from_const("DEFAULT"),
            fair_value_cents: 10_000, // $100.00
            theta: 0.5,
            sigma_cents: 2.0,
            tick_cents: 1,
            limit_lambda: 30.0,
            market_lambda: 8.0,
            cancel_lambda: 25.0,
            size_log_mean: (20.0_f64).ln(),
            size_log_sigma: 0.6,
            size_max: 500,
            price_alpha: 1.5,
            price_max_ticks: 20,
        }
    }
}


pub struct MarketSimulator {
    cfg: SimConfig,
    rng: ChaCha8Rng,
    /// Mid-price (cents, real-valued so the OU process can drift between ticks).
    mid_real: f64,
    /// Monotonic order id supply.
    next_id: u64,
    /// Wall-clock-ns supply for deterministic order timestamps.
    sim_clock_ns: u64,
}

impl MarketSimulator {
    pub fn new(cfg: SimConfig, seed: u64) -> Self {
        MarketSimulator {
            mid_real: cfg.fair_value_cents as f64,
            cfg,
            rng: ChaCha8Rng::seed_from_u64(seed),
            next_id: 1,
            sim_clock_ns: 0,
        }
    }

    pub fn config(&self) -> &SimConfig {
        &self.cfg
    }

    pub fn mid_cents(&self) -> u64 {
        self.mid_real.round().max(1.0) as u64
    }

    /// Advance simulation by `dt` seconds and emit any actions that
    /// occurred. Aggregates by side. The caller submits/cancels each one.
    pub fn step(&mut self, dt: f64) -> Vec<SimAction> {
        // 1. Drift the OU mid by dt.
        // dX = θ(μ − X)dt + σ √dt · N(0, 1)
        let mu = self.cfg.fair_value_cents as f64;
        let drift = self.cfg.theta * (mu - self.mid_real) * dt;
        let shock = self.cfg.sigma_cents * dt.sqrt() * standard_normal(&mut self.rng);
        self.mid_real += drift + shock;
        self.sim_clock_ns += (dt * 1_000_000_000.0) as u64;

        let mut actions = Vec::new();

        // 2. Sample event counts for this tick (Poisson per channel × per side).
        for side in [Side::Buy, Side::Sell] {
            let lim_n = poisson_sample(&mut self.rng, self.cfg.limit_lambda * dt);
            for _ in 0..lim_n {
                if let Some(o) = self.gen_limit(side) {
                    actions.push(SimAction::Submit(o));
                }
            }
            let mkt_n = poisson_sample(&mut self.rng, self.cfg.market_lambda * dt);
            for _ in 0..mkt_n {
                if let Some(o) = self.gen_market(side) {
                    actions.push(SimAction::Submit(o));
                }
            }
            let cnl_n = poisson_sample(&mut self.rng, self.cfg.cancel_lambda * dt);
            for _ in 0..cnl_n {
                actions.push(SimAction::CancelHint);
            }
        }

        actions
    }

    fn gen_limit(&mut self, side: Side) -> Option<Order> {
        // Distance ticks ~ geometric weighted by 1/(k+1)^α.
        let ticks = sample_distance(&mut self.rng, self.cfg.price_alpha, self.cfg.price_max_ticks);
        let mid = self.mid_cents() as i64;
        let tick = self.cfg.tick_cents as i64;
        // Limits sit on their own side: buys below mid, sells above mid.
        let raw = match side {
            Side::Buy => mid - (ticks as i64) * tick,
            Side::Sell => mid + (ticks as i64) * tick,
        };
        if raw <= 0 {
            return None;
        }
        let price = Px::from_cents(raw as u64).ok()?;
        let qty = self.gen_qty()?;
        let id = self.next_order_id();
        Order::new(id, self.cfg.symbol, side, price, qty, Ts::from_nanos(self.sim_clock_ns)).ok()
    }

    fn gen_market(&mut self, side: Side) -> Option<Order> {
        // Aggressive limit priced "through" the touch by 5 ticks.
        let mid = self.mid_cents() as i64;
        let tick = self.cfg.tick_cents as i64;
        let raw = match side {
            Side::Buy => mid + 5 * tick,
            Side::Sell => mid - 5 * tick,
        };
        if raw <= 0 {
            return None;
        }
        let price = Px::from_cents(raw as u64).ok()?;
        let qty = self.gen_qty()?;
        let id = self.next_order_id();
        Order::new(id, self.cfg.symbol, side, price, qty, Ts::from_nanos(self.sim_clock_ns)).ok()
    }

    fn gen_qty(&mut self) -> Option<Qty> {
        let n = standard_normal(&mut self.rng);
        let log_size = self.cfg.size_log_mean + self.cfg.size_log_sigma * n;
        let raw = log_size.exp().round() as u32;
        let clipped = raw.clamp(1, self.cfg.size_max);
        Some(Qty::new(clipped))
    }

    fn next_order_id(&mut self) -> OrderID {
        let id = self.next_id;
        self.next_id = self.next_id.checked_add(1).unwrap_or(1);
        OrderID::new(id).expect("non-zero")
    }

    /// Re-seed the RNG. Used by `reset` keybind.
    pub fn reseed(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
        self.mid_real = self.cfg.fair_value_cents as f64;
        self.sim_clock_ns = 0;
    }
}

// ─── Helpers ───────────────────────────────────────────────────────────────

/// Box-Muller standard-normal sample.
fn standard_normal<R: RngCore>(rng: &mut R) -> f64 {
    // Avoid log(0) by clamping u1 ≥ 1e-12.
    let u1: f64 = rng.r#gen::<f64>().max(1e-12);
    let u2: f64 = rng.r#gen::<f64>();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

/// Knuth's small-λ Poisson sampler. Returns a non-negative integer count of
/// events occurring with mean `lambda`.
fn poisson_sample<R: Rng + ?Sized>(rng: &mut R, lambda: f64) -> u32 {
    if lambda <= 0.0 {
        return 0;
    }
    // For modest lambda (< 30) this is fast and exact.
    let l = (-lambda).exp();
    let mut k: u32 = 0;
    let mut p = 1.0;
    loop {
        k += 1;
        p *= rng.r#gen::<f64>();
        if p <= l || k > 1_000 {
            return k.saturating_sub(1);
        }
    }
}

/// Sample a tick distance with weight ∝ 1/(k+1)^α, k ∈ [0, max].
fn sample_distance<R: Rng + ?Sized>(rng: &mut R, alpha: f64, max_ticks: u32) -> u32 {
    let weights: Vec<f64> = (0..=max_ticks)
        .map(|k| 1.0 / ((k as f64 + 1.0).powf(alpha)))
        .collect();
    let total: f64 = weights.iter().sum();
    let mut t = rng.r#gen::<f64>() * total;
    for (i, w) in weights.iter().enumerate() {
        t -= w;
        if t <= 0.0 {
            return i as u32;
        }
    }
    max_ticks
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_under_fixed_seed() {
        let mut a = MarketSimulator::new(SimConfig::default(), 42);
        let mut b = MarketSimulator::new(SimConfig::default(), 42);
        let aa = a.step(0.1);
        let bb = b.step(0.1);
        assert_eq!(aa.len(), bb.len());
        // Compare the orders that came out.
        for (x, y) in aa.iter().zip(bb.iter()) {
            match (x, y) {
                (SimAction::Submit(ox), SimAction::Submit(oy)) => {
                    assert_eq!(ox.id(), oy.id());
                    assert_eq!(ox.price(), oy.price());
                    assert_eq!(ox.quantity(), oy.quantity());
                    assert_eq!(ox.side(), oy.side());
                }
                (SimAction::CancelHint, SimAction::CancelHint) => {}
                _ => panic!("action mismatch under fixed seed"),
            }
        }
    }

    #[test]
    fn step_emits_orders_within_expected_band() {
        let mut sim = MarketSimulator::new(SimConfig::default(), 7);
        // 5-second horizon should produce well over zero orders.
        let mut all = Vec::new();
        for _ in 0..50 {
            all.extend(sim.step(0.1));
        }
        let submits = all
            .iter()
            .filter(|a| matches!(a, SimAction::Submit(_)))
            .count();
        assert!(submits > 50, "expected >50 submits, got {submits}");
    }

    #[test]
    fn mid_price_stays_in_reasonable_neighbourhood() {
        let mut sim = MarketSimulator::new(SimConfig::default(), 1);
        for _ in 0..1_000 {
            let _ = sim.step(0.01);
        }
        let mid = sim.mid_cents();
        // OU around 10000 with θ=0.5, σ=2 ⇒ stationary stddev ≈ σ/√(2θ) ≈ 2.
        // Well within ±200 cents.
        assert!(mid > 9_500 && mid < 10_500, "mid drifted to {mid}");
    }
}
