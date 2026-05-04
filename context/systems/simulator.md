# Simulator

*Maturity: working · Stability: stable*

## Scope / Purpose

`src/simulator/` is the synthetic order-flow generator that drives the dashboard. It produces a realistic stream of submits and cancellation hints the matching engine can chew on. The flow shape is grounded in the Cont–Stoikov–Talreja stochastic LOB model: independent Poisson arrivals per side, log-normal order sizes, and a mean-reverting Ornstein–Uhlenbeck walk on the theoretical fair value.

All randomness flows through one `ChaCha8Rng`, so a fixed seed produces a byte-identical event stream — important for reproducible bug repros and deterministic dashboard demos.

## Boundaries / Ownership

- **Owns:** `MarketSimulator`, `SimConfig`, `SimAction`, the OU drift, the Poisson sampler, the log-normal size generator, the distance-decay sampler, the `next_id` supply.
- **Does not own:** the matching engine itself (lives in `book::order_book`), or the cancellation lookup (`SimAction::CancelHint` just signals "now is a good time to cancel"; the dashboard's `App` picks the actual id from its own resting cache).
- **Imported by:** `ui::app` (one `MarketSimulator` per dashboard instance) and the headless `main.rs`.

## Current Implemented Reality

### Configuration

```rust
pub struct SimConfig {
    pub fair_value_cents: u64,    // 10_000 ($100.00)
    pub theta: f64,                // 0.5 — OU mean-reversion strength (per second)
    pub sigma_cents: f64,          // 2.0 — OU diffusion (cents per √sec)
    pub tick_cents: u64,           // 1
    pub limit_lambda: f64,         // 30/s per side
    pub market_lambda: f64,        // 8/s per side
    pub cancel_lambda: f64,        // 25/s per side
    pub size_log_mean: f64,        // ln(20)
    pub size_log_sigma: f64,       // 0.6
    pub size_max: u32,             // 500
    pub price_alpha: f64,          // 1.5 — distance-decay exponent
    pub price_max_ticks: u32,      // 20
}
```

Defaults follow the design brief in `notes/dashboard-design.md`: ~60 events/s aggregate, log-normal sizes clipped to [1, 500], OU stationary stddev ≈ 2 cents.

### Step semantics

`MarketSimulator::step(dt: f64) -> Vec<SimAction>` does, in order:
1. **OU drift** on `mid_real`: `dX = θ(μ − X) dt + σ √dt · N(0,1)` — Box–Muller standard normal.
2. **Advance the simulator clock** by `dt × 1e9` ns (used as the timestamp for every order generated this step).
3. **Per side**, sample three Poisson counts with `λ × dt`:
   - limit-order arrivals → `gen_limit(side)` → `SimAction::Submit(order)`,
   - market-order arrivals → `gen_market(side)` → `SimAction::Submit(order)` priced 5 ticks through the touch,
   - cancellation arrivals → `SimAction::CancelHint`.

`gen_limit` samples a tick distance with weight ∝ `1 / (k+1)^α`, so most limit orders sit near the touch and density falls off with distance.

### Determinism

`MarketSimulator::new(cfg, seed: u64)` seeds the RNG. The `tests/` integration tests + the inline `deterministic_under_fixed_seed` test both verify byte-identical streams under a fixed seed:

```rust
let mut a = MarketSimulator::new(SimConfig::default(), 42);
let mut b = MarketSimulator::new(SimConfig::default(), 42);
assert_eq!(a.step(0.1), b.step(0.1));   // every Order id, price, quantity, side matches
```

`reseed(seed)` resets both the RNG and the OU mid back to `fair_value_cents`. Used by the dashboard's `r` keypress (Reset).

## Key Interfaces / Data Flow

```rust
pub enum SimAction {
    Submit(Order),
    CancelHint,    // "now is a good time to cancel something"; caller picks the id
}

impl MarketSimulator {
    pub fn new(SimConfig, seed: u64) -> Self;
    pub fn config(&self) -> &SimConfig;
    pub fn mid_cents(&self) -> u64;
    pub fn step(&mut self, dt: f64) -> Vec<SimAction>;
    pub fn reseed(&mut self, seed: u64);
}
```

Driver loop:

```
App::step(dt)
  ├─ if EngineState::Paused: return
  ├─ scaled_dt = dt × app.speed
  ├─ for action in sim.step(scaled_dt):
  │     match action:
  │       Submit(order)   → handle_submit(order)
  │       CancelHint      → handle_cancel_hint()
  └─ mid_history.push_back(sim.mid_cents())
```

`speed` defaults to 1.0; `+`/`-` keys multiply by 1.5 / divide by 1.5, clamped to `[0.1, 50.0]`.

## Implemented Outputs / Artifacts

- The two module files (`simulator/mod.rs`, `simulator/market.rs`).
- 3 inline unit tests: deterministic-under-fixed-seed, step-emits-orders-within-expected-band, mid-price-stays-in-reasonable-neighbourhood.
- Headless `main --no-tui` produces ~750 orders / ~550 fills per 10 simulated seconds with the default config.

## Known Issues / Active Risks

- **Knuth's small-λ Poisson sampler is exact only for modest λ.** With the default `limit_lambda = 30 events/s`, a 50ms tick has λ = 1.5, well within the safe range. If a future user cranks `dt × λ` past ~30 the sampler degrades; we cap iterations at 1000 to avoid runaway loops, but the distribution would be wrong. Worth replacing with a transformed-rejection sampler if `λ × dt > 30` becomes a real config.
- **`gen_market` prices "5 ticks through the touch"** — a hardcoded constant. For most synthetic setups this works, but a config field would be cleaner.
- **Box–Muller wastes one sample.** Each call to `standard_normal` consumes two uniform samples and uses the cosine pair only. Negligible cost; documented for completeness.
- **Cancellations target the wrong order distribution.** `CancelHint` is a signal; `App::handle_cancel_hint` cancels by `(self.total_cancels as usize) % resting_ids.len()` which is round-robin. Real markets cancel proportionally to queue size at a level (Cont's "proportional to liquidity"). The current behaviour skews uniform; visually fine for a demo, distributionally wrong for a research-grade simulator.

### Downstream impact

The simulator drives every visible thing in the dashboard. A bug here that, say, stopped emitting market orders would result in a book that fills slowly (no aggressive crossings), no trade tape, and a flat latency histogram. The `step_emits_orders_within_expected_band` test pins the gross output rate.

## Partial / In Progress

None — the simulator is feature-complete to the design brief.

## Planned / Missing / Likely Changes

- **Hawkes self-exciting overlay** for clustered "news" episodes (mentioned in the design brief; toggleable). Each market order would temporarily multiply `market_lambda` by 2 with 0.5s decay.
- **ITCH-style replay producer** as a swap-in alternative to synthetic generation. The `SimAction` enum is general enough to be the boundary; an ITCH reader would emit the same actions from a recorded feed.
- **Liquidity-proportional cancellations.** Replace round-robin in `App::handle_cancel_hint` with weighted-by-queue-size sampling.
- **Per-order ts derived from the wall clock + `sim_clock_ns`** so simulator-driven orders get realistic ns-precision timestamps.

## Durable Notes / Discarded Approaches

- **One RNG, one seed, deterministic.** Considered per-channel RNGs (one for arrivals, one for sizes, one for prices) but rejected — the cross-channel determinism property is easier to reason about with a single source. The cost is that swapping the order of two `gen` calls inside `step` would change the entire output stream; that's accepted as a small price for clean reproducibility.
- **OU on the *real-valued* mid, not the integer cents.** Cents are derived by `mid_cents() = mid_real.round().max(1.0) as u64`. Integer-only OU would require truncation/rejection of small drift steps.
- **Knuth's algorithm chosen over `Distribution::Poisson` from `rand_distr`.** Avoids pulling another transitive dep. Performance is fine for our λ values.
- **The full simulator is in one module.** Considered splitting into `simulator/{config, rng, distributions, market}.rs` but the entire file is ~250 LOC and splits would just add navigation overhead.

## Obsolete / No Longer Relevant

None — this module was authored fresh.
