# Feed

*Maturity: working · Stability: unstable — first iteration; reconnect / sequence-gap handling are minimal*

## Scope / Purpose

`src/feed/` connects the dashboard to a live public market-data WebSocket — Coinbase Advanced Trade's `level2` channel — so `cargo run -- --live coinbase` shows real BTC-USD/ETH-USD/SOL-USD depth in the terminal instead of synthetic flow.

Two modules:

- **`coinbase`** — connects to `wss://advanced-trade-ws.coinbase.com`, sends a public `level2` subscribe, parses incoming JSON into typed [`FeedEvent`](#feedevent) variants, owns reconnect-with-exponential-backoff. No authentication; no API key; no signup.
- **`bridge`** — translates the per-level `FeedEvent` stream into the engine's per-order `SimAction` stream. Coinbase's L2 protocol says "the bid at $80305 is now 0.5 BTC"; the bridge maintains a virtual `OrderID` per `(symbol, side, price)` cell so updates land as cancel-old-virtual + submit-new-virtual on the matching engine.

The live feed is **mode-exclusive** with the synthetic simulator. In Live mode the per-symbol `MarketSimulator` instances exist but never tick — the bridge feeds flow instead. In Synthetic mode the simulators tick as today; no WebSocket connection is opened.

## Boundaries / Ownership

- **Owns:** `CoinbaseConfig`, `FeedEvent`, the WebSocket client + JSON parser (`coinbase`), the L2-to-virtual-order translator (`bridge`), the symbol mapping `(Symbol → SymbolState idx)`, the `(Symbol, Side, Px) → OrderID` cell tracker, the synthetic-id allocator (starts at `1_000_000_000_000` to avoid colliding with simulator-assigned ids), the `QTY_SCALE = 1e6` constant for fractional-crypto quantity preservation.
- **Does not own:** the matching engine (`book::Market`), the dashboard (`ui::App`) — the feed produces `SimAction`s; everything downstream is shared with the synthetic path.
- **Imported by:** `main.rs` (when `--live coinbase` is parsed) and `examples/live_smoke.rs`. The library never imports `feed` from non-binary code; the feed is a binary-mode-only concern.

## Current Implemented Reality

### Coinbase WebSocket protocol shape

Connection: `wss://advanced-trade-ws.coinbase.com`. Subscribe message:

```json
{"type": "subscribe", "product_ids": ["BTC-USD", "ETH-USD", "SOL-USD"], "channel": "level2"}
```

Server messages relevant to us have `channel == "l2_data"` and an `events` array. Each event has a `type` (`"snapshot"` or `"update"`), a `product_id`, and an `updates` array of `(side, price_level, new_quantity)` triples. Sides are `"bid"` and `"offer"` (we accept `"ask"` as a synonym).

Coinbase sends one large snapshot per subscribed product on connect, then a steady stream of incremental updates. The first observed snapshot for BTC-USD was 25,191 bids × 22,072 asks at $80,305 mid — production-shape data.

### `FeedEvent`

```rust
pub enum FeedEvent {
    Snapshot { symbol: Symbol, bids: Vec<(Px, Qty)>, asks: Vec<(Px, Qty)> },
    Update   { symbol: Symbol, side: Side, price: Px, new_quantity: Qty },
    Status(String),
}
```

`Status` carries connection / subscribe / parse-error messages so the dashboard can surface them in a banner.

### `Bridge` translation

| FeedEvent | SimAction emissions |
|-----------|---------------------|
| `Snapshot { bids, asks }` | For every existing virtual cell on this symbol: `Cancel`. For every non-zero `(price, qty)` in the snapshot: allocate a new id, `Submit(Order)`. |
| `Update { side, price, qty > 0 }` | If a virtual cell already exists at `(symbol, side, price)`: `Cancel` it. Then `Submit(Order)` with the new quantity. |
| `Update { side, price, qty == 0 }` | `Cancel` the virtual cell at `(symbol, side, price)` if any; otherwise drop. |
| `Status(_)` | None (status is for UI only). |

Symbols not in the bridge's pre-registered list are silently dropped.

### Quantity scaling

Coinbase quantities are fractional (e.g. `"0.5"` BTC). Our `Qty` is `u32` of whole units. The bridge multiplies by `QTY_SCALE = 1_000_000` so 0.5 BTC becomes `Qty(500_000)`. This preserves micro-unit precision (~$0.06 at $80k BTC) while staying in `u32` range (max representable is `u32::MAX / 1e6` ≈ 4,294 BTC per individual order — far above any single-trade size on Coinbase).

Display in the dashboard remains the raw scaled integer. Future polish: per-symbol display-divisor so the engine pane shows "37.745 BTC" instead of "37745".

### Threading model

The dashboard's `App` is single-threaded. The feed runtime lives on a **separate OS thread** spawned by `main.rs`:

```text
main thread                          feed thread
─────────────                        ────────────
                                     tokio runtime
                                       │
                                       ▼
ui::run_with_app(app)              run_coinbase()
  │                                   │ WebSocket events
  │                                   ▼
  │                                 Bridge::translate
  │                                   │ FeedAction
  │  std::sync::mpsc::Receiver  ◄────┘ (mpsc::Sender)
  ▼
App::step (drain non-blockingly)
```

The feed thread owns its own Tokio runtime. The two threads communicate via a single `std::sync::mpsc` channel carrying `FeedAction`s. The main loop drains the channel non-blockingly each tick. If the main thread quits, the channel closes and the feed thread terminates.

### Reconnect

`run_coinbase`'s outer loop reconnects forever with exponential backoff: starting at 250ms, doubling on each failure, capped at 30s. A successful subscribe resets the delay. Status messages flow through the same `FeedEvent::Status` path so the dashboard can render reconnection banners.

## Key Interfaces / Data Flow

```rust
// Public surface re-exported from `feed/mod.rs`:
pub struct CoinbaseConfig { pub product_ids: Vec<String> }
pub enum FeedEvent { Snapshot{..}, Update{..}, Status(String) }
pub async fn run_coinbase(cfg: CoinbaseConfig, tx: tokio::sync::mpsc::Sender<FeedEvent>);

pub struct Bridge { /* private state */ }
impl Bridge {
    pub fn new(symbols: Vec<Symbol>) -> Self;
    pub fn translate(&mut self, event: FeedEvent) -> Vec<FeedAction>;
}
pub struct FeedAction {
    pub symbol_idx: usize,
    pub action: SimAction,
}
pub const QTY_SCALE: f64 = 1_000_000.0;
```

The `App` wires the feed via `App::new_live(symbols, feed_rx)` and dispatches via the shared `App::dispatch(idx, SimAction)` path that synthetic flow already uses.

## Implemented Outputs / Artifacts

- Two module files (`feed/mod.rs`, `feed/coinbase.rs`, `feed/bridge.rs`).
- 4 inline unit tests in `bridge` covering: snapshot routes to correct symbol idx, update with non-zero qty emits cancel-then-submit, update with zero qty emits cancel-only, unknown symbol drops silently.
- `examples/live_smoke.rs` — TUI-free smoke test that connects to Coinbase, prints the first 60 events, summarises submit/cancel action counts; verified working end-to-end against production Coinbase as of 2026-05-04.
- `cargo run --release -- --live coinbase` is the canonical demo entry point; same dashboard, real data.

## Known Issues / Active Risks

- **Resyncing on sequence gaps is minimal.** Coinbase emits a `sequence_num` field on each message; we do not currently track or verify it. A dropped delta will desync the bridge's level cache from the venue's true book until the next snapshot. For demo purposes invisible; for any analytical use, sequence-gap detection + re-snapshot trigger is the next-iteration must-have.
- **Quantity scaling is hardcoded to BTC-style 1e6.** ETH and SOL also use 1e6 for consistency; equity-style or higher-precision crypto symbols would need per-symbol scaling. Not currently configurable.
- **No authentication.** Coinbase recommends (not requires) authenticating with a CDP API key for "more reliable connection." Without auth, the connection may be bumped under load. For a personal-use dashboard demo this is fine; if running 24/7 on a server, adding auth would reduce reconnect noise.
- **Rate limit on unauthenticated public connections.** Coinbase enforces a per-IP cap (~hundreds of subscriptions per IP). Three products is well below; running multiple instances of the dashboard from the same machine could trip the limit.
- **`std::mem::replace` dance in `App::step`.** Required to satisfy borrow-checker around `&mut self.mode` while calling `&mut self` methods. Functional but slightly ugly; alternative would be a `MessageBus` trait abstraction over both modes. Acceptable for first iteration.

### Downstream impact

Bugs in the feed would manifest as a stale or wrong dashboard:
- Mistranslated quantity → bars in the depth ladder are sized wrong (off by 6 orders of magnitude).
- Missed snapshot → empty book despite connection.
- Sequence-gap silent failure → drift between displayed depth and venue's true depth (subtle; may go unnoticed).

For tests that depend on determinism (the entire matching-engine test suite), `cargo test` runs against the synthetic simulator only. The live feed is *not* in the test path. Determinism is preserved.

## Partial / In Progress

None — the live feed is feature-complete to its first-iteration scope.

## Planned / Missing / Likely Changes

- **Sequence-gap detection.** Track Coinbase's `sequence_num` per product; on gap, request a fresh snapshot. Currently we drop deltas silently.
- **Per-symbol display divisor.** So the dashboard shows "37.745 BTC" not "37745".
- **`level2_batch` channel option.** Coinbase offers a batched variant with lower message rate; useful for low-bandwidth demos.
- **Multi-venue.** A second feed thread for Binance or Kraken running in parallel. The bridge architecture already supports this — symbols just need distinct names.
- **Auth path.** Optional CDP API key for more reliable connections under load.
- **Reconnect-with-resync.** When reconnecting, re-subscribe and discard any pre-snapshot updates so the bridge starts clean.

## Durable Notes / Discarded Approaches

- **`native-tls` over `rustls`.** Initially used `rustls-tls-webpki-roots` per the design brief in `notes/dashboard-design.md`. rustls 0.23+ requires explicit crypto-provider installation (`aws-lc-rs` or `ring`); the smoke test panicked with "Could not automatically determine the process-level CryptoProvider". Switching to `native-tls` (Security.framework on macOS, OpenSSL on Linux) eliminated the runtime-config dance and worked first-try. Cost: an `openssl` transitive dep on Linux. Acceptable for a demo project.
- **Separate OS thread, not async dashboard.** Considered fully-async dashboard with `tokio::select!` over input + render + sim ticks. Rejected for first iteration: the existing blocking dashboard loop works fine, and a separate thread + `mpsc::channel` is much simpler than refactoring the whole render path. The async refactor is a Tier-3 nice-to-have.
- **Synthetic id allocation starts at `1_000_000_000_000`.** Bridges are not the only thing allocating `OrderID`s in the dashboard — the synthetic simulator does too. Starting at 1e12 prevents ever-colliding ids when both modes' ids might appear in the same engine state (unlikely in practice but cheap insurance).
- **`level2` channel only, not `ticker` or `market_trades`.** The bridge can reconstruct trades from the L2 stream (level qty decreases imply executions) and we already have a tape; subscribing to additional channels would multiply message volume without adding signal. Future iteration may opt into `market_trades` for trade-tape colour (aggressor side comes pre-tagged from the venue).

## Obsolete / No Longer Relevant

None — module is brand-new in this iteration.
