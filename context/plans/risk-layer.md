# Plan: Real-Time Risk Layer

## Header

- **Status:** Planned (not started)
- **Scope:** A thin pre-trade risk guard sitting between submission and the matching engine. Implements fat-finger checks, position limits, rolling VaR circuit breaker, per-session throttle, and a fail-safe-on-error policy.
- **Why this matters:** A matching engine without a risk layer is a liability. Recruiters at HFT firms specifically ask about this in the first 15 minutes — it's a well-known shortlist topic. The project's README already mentions it; the implementation has been deferred.
- **Exit rule:** complete when (a) every order passes through `RiskGuard::check` before reaching the book, (b) a configurable risk policy can be attached at engine startup, (c) violations are surfaced as `OrderEvent::Rejected` with specific reasons, (d) tests cover the four canonical violation modes.

## Implementation Structure

### Modules / files affected

- `src/risk/` (new):
  - `mod.rs`
  - `guard.rs` — `RiskGuard` and the per-check methods
  - `policy.rs` — `RiskPolicy` config struct
  - `position.rs` — per-session position tracker
  - `throttle.rs` — token-bucket throttle per session
  - `var.rs` — rolling-variance circuit breaker
- `src/types.rs` — add `SessionID` (analogous to `OrderID`).
- `src/events/lifecycle.rs` — add `OrderRejectionReason::FatFingerPrice/Quantity`, `PositionLimit`, `Throttle`, `VarBreach`.
- `tests/risk_test.rs` (new).

### Layered architecture

```
            ┌────────────────┐
   submit ─►│   RiskGuard    │──reject──► OrderEvent::Rejected (specific reason)
            │  (this layer)  │
            └────┬───────────┘
                 │ accept
                 ▼
            ┌────────────────┐
            │  Market /      │──fills──► engine output as today
            │  OrderBook     │
            └────────────────┘
```

`RiskGuard` is a wrapper around `Market`. The `App` holds a `RiskGuard` instead of a bare `Market`. Calls from the simulator go through it.

### `RiskPolicy` shape

```rust
pub struct RiskPolicy {
    pub fat_finger: FatFingerPolicy,
    pub position: PositionPolicy,
    pub throttle: ThrottlePolicy,
    pub var: VarPolicy,
}
pub struct FatFingerPolicy {
    pub max_price_dev_bps: u32,    // reject if price > mid * (1 + dev_bps/10000)
    pub max_quantity: u32,
}
pub struct PositionPolicy {
    pub max_long: i64,
    pub max_short: i64,
}
pub struct ThrottlePolicy {
    pub max_orders_per_sec: u32,
}
pub struct VarPolicy {
    pub window_secs: u32,
    pub max_realised_var_cents: u64, // halts trading if breached
}
```

Sensible defaults: ±10% from mid, max qty 10000, 1000 orders/sec/session, VaR window 30s, max-VaR 500 cents.

## Algorithm / System Sections

### A) Fat-finger

**Playbook:**
- [ ] Read `Market::microprice(symbol)` for the order's symbol (or `mid` if microprice unavailable).
- [ ] Compute `dev_bps = abs(order.price.cents() - mid_cents) / mid_cents * 10000`.
- [ ] Reject if `dev_bps > policy.max_price_dev_bps` or `order.quantity > policy.max_quantity`.

### B) Position limits

**Playbook:**
- [ ] `PositionTracker` per session: `i64` running net. Buy adds, sell subtracts (signed by quantity * fill direction).
- [ ] On every fill, update the tracker for both buyer and seller sessions.
- [ ] On submit, project the worst-case post-fill position (`current ± full_quantity`); reject if it would breach.

### C) Per-session throttle

**Playbook:**
- [ ] Token bucket per session: `tokens: f64`, refill rate `policy.max_orders_per_sec` tokens/sec, capacity equal to refill rate (1-second burst window).
- [ ] On submit: take 1 token; reject if bucket empty.

### D) Rolling-variance VaR

**Playbook:**
- [ ] Track recent fill prices in a ring buffer covering `window_secs`.
- [ ] Compute realised variance σ² over the window.
- [ ] If `σ² × scaling > policy.max_realised_var_cents`, halt the affected session (or the entire engine — policy choice).
- [ ] Halts emit `OrderEvent::Rejected { reason: VarBreach }` for subsequent submits until the window's tail rolls past the breach.

### E) Fail-safe-on-error

If any risk check itself errors (e.g. position tracker poisoned, VaR window state corrupted), the default action is **reject**, not pass-through. This is the pattern real risk systems use — when in doubt, refuse.

## Integration Points

- `App` constructs `RiskGuard::new(policy, market)`; the simulator's submits route through `guard.submit_limit(symbol, order, session_id)` instead of directly into `market.submit_limit`.
- The simulator gains a `session_id` per generated order. Default is "DEFAULT" session for synthetic flow; a future multi-session config can stress-test throttle policies.
- The dashboard adds a "Risk" pane (or a row in the engine pane) showing rejections-by-reason.

## Debugging / Verification

- Test that rejecting a fat-finger order does *not* mutate the book.
- Test that throttle resets correctly across the second boundary.
- Test that VaR breach correctly resolves once the breach falls out of the window.
- Test that position limit projects the *worst-case* post-fill, not just current.

## Completion Criteria

- [ ] `src/risk/` exists and `RiskGuard` wraps `Market`.
- [ ] `tests/risk_test.rs` covers the four canonical violation modes (fat-finger, position, throttle, VaR).
- [ ] Dashboard renders a per-reason rejection counter.
- [ ] `systems/risk.md` (new system file) documents the layer.
- [ ] `architecture.md` is updated to show `RiskGuard` between submission and `Market`.
- [ ] README's "Risk Layer" section is no longer aspirational.
- [ ] This file is archived once all the above are checked.
