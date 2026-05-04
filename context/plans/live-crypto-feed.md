# Plan: Live Crypto Feed

## Header

- **Status:** Planned (not started)
- **Scope:** Connect to a free public crypto WebSocket (Coinbase Advanced Trade), parse the L2 update stream, and feed the matching engine alongside (or instead of) synthetic flow.
- **Why this matters:** "Engine works on real data" is the single most impressive next-step for a portfolio piece, and crypto WebSockets are the only zero-cost path to that demo. Cross-references `notes/free-data-sources.md`.
- **Exit rule:** complete when (a) a `--live coinbase` (or similar) flag drives the dashboard from a live WebSocket feed, (b) the depth-of-book pane shows live BTC-USD or ETH-USD depth, (c) reconnect-on-drop works, (d) the feed degrades gracefully back to synthetic if the network is down.

## Implementation Structure

### Modules / files affected

- `Cargo.toml` — add `tokio = { version = "1", features = ["rt-multi-thread", "macros", "net", "io-util"] }`, `tokio-tungstenite` (WebSocket), `serde_json`.
- `src/feed/` (new):
  - `mod.rs`
  - `coinbase.rs` — Coinbase Advanced Trade WebSocket client + `level2` channel parser
  - `bridge.rs` — translates feed messages into `SimAction`-equivalent actions for the engine
- `src/main.rs` — `--live <venue>` CLI flag.
- `src/ui/app.rs` — async run loop variant when feed mode is active.

### WebSocket protocol (Coinbase Advanced Trade `level2` channel)

- WebSocket URL: `wss://advanced-trade-ws.coinbase.com`.
- Subscribe message: `{"type": "subscribe", "channel": "level2", "product_ids": ["BTC-USD", "ETH-USD", ...]}`.
- Initial response: snapshot with full bid/ask depth.
- Subsequent messages: incremental updates (`{"side": "bid", "event_time": "...", "price_level": "...", "new_quantity": "..."}`).

The protocol is L2 (per-level), not L3. This means we can rebuild the visible book but not the per-order FIFO inside each level. For our matching engine, we treat each level as a single virtual order at the published quantity.

### Translation

| Coinbase event | Engine action |
|----------------|---------------|
| Snapshot row (bid, qty > 0) | Submit a synthetic `Order::new` at that price + size, side=Buy |
| Snapshot row (ask, qty > 0) | Submit at price + size, side=Sell |
| Update row (qty > 0, new level) | Submit at price + size |
| Update row (qty == 0, level cleared) | Cancel the synthetic order at that price |
| Update row (qty changed) | Cancel old + submit new (delta replacement) |

We will need a synthetic `OrderID` allocator per `(symbol, side, price)` so cancels target the right virtual order.

### Async run loop

The current dashboard uses a single-threaded blocking loop. The feed introduces network IO that must not block render. Two options:

- **Option A (simpler):** dedicated tokio runtime in a separate thread; channel-based delivery of feed messages into the existing blocking dashboard loop. The dashboard polls a `mpsc::Receiver` non-blockingly.
- **Option B (idiomatic):** convert the dashboard to fully async (`tokio::select!` between input, render tick, sim tick, feed messages). Bigger refactor.

Recommended: Option A first. Less invasive, same external behaviour.

## Algorithm / System Sections

### A) WebSocket connect + auth

**Playbook:**
- [ ] Connect to `wss://advanced-trade-ws.coinbase.com` with `tokio-tungstenite`.
- [ ] Send subscribe message for the configured product list.
- [ ] No auth required for `level2` public channel.
- [ ] Reconnect on drop with exponential backoff (start 250ms, cap 30s).

### B) Snapshot + delta reconstruction

**Playbook:**
- [ ] On snapshot: clear the synthetic book for that symbol, submit an order per level.
- [ ] On update: route each row to the appropriate cancel/submit per the table above.
- [ ] Track sequence numbers; on gap, request a fresh snapshot.

### C) Bridge to `Market`

**Playbook:**
- [ ] `Bridge::handle_feed_event(event) -> NyquestroResult<()>` calls into `Market::submit_limit` / `Market::cancel`.
- [ ] On feed-induced order: tag with a known synthetic session id (e.g. `SessionID::new("FEED")`) so risk checks treat it differently if needed.

### D) Graceful degradation

**Playbook:**
- [ ] If the feed channel is silent for > 5s, raise a "feed stalled" indicator on the dashboard.
- [ ] On full drop, fall back to synthetic flow with a banner indicating the fallback.

## Integration Points

- `--live coinbase --product BTC-USD` adds a live source.
- `--live coinbase --product BTC-USD --hybrid` merges live feed + synthetic noise (interesting for stress-testing the engine while still demoing real data).
- Multi-instrument support is a prerequisite (per the in-flight implementation in this session).

## Debugging / Verification

- Reconnect test: kill the local network for 10s; the feed should reconnect and re-snapshot cleanly.
- Sequence-gap test: simulate a missed message (drop one delta in the bridge); the bridge should detect the gap via the venue's sequence number and re-snapshot.
- Latency: measure end-to-end feed-event-to-render-frame latency. Target < 50ms p99.

## Completion Criteria

- [ ] `cargo run -- --live coinbase --product BTC-USD` shows live BTC depth in the dashboard.
- [ ] Reconnect-on-drop works.
- [ ] `tests/feed_test.rs` covers: reconnect, sequence gap, JSON parse errors.
- [ ] `systems/feed.md` (new system file) documents the bridge.
- [ ] `architecture.md` mentions the live-feed path as an alternative source.
- [ ] This file is archived once all the above are checked.
