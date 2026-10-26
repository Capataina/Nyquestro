# Free Data Sources

Hard constraint: **Nyquestro stays zero-cost forever.** No subscriptions, no per-call billing, no academic gating that requires an institutional email. Every "real-data" upgrade has to slot into this constraint.

This note is the inventory of free real-market-data sources and which ones are realistic to consume from this project.

## 1. Current Understanding

The single best free source for high-quality limit-order-book data is **crypto exchange WebSockets**. They give you what NASDAQ ITCH gives you on equities — full L2 / L3 depth updates, per-trade prints, quote changes — but with zero gating and 24/7 markets.

### Recommendation, in priority order

| Source | Cost | Real-time? | L2 depth? | L3 (per-order)? | Auth needed? | Notes |
|--------|------|------------|-----------|------------------|--------------|-------|
| **Coinbase Advanced Trade WebSocket** | Free | ✅ | ✅ (`level2` channel) | Limited | No | The cleanest path. Public market-data channels need no auth. Snapshot + delta protocol. |
| **Binance WebSocket** | Free | ✅ | ✅ (`@depth` stream) | No | No | Higher message rate than Coinbase. Stable. Geo-restricted in some jurisdictions. |
| **Kraken WebSocket v2** | Free | ✅ | ✅ (`book` channel) | No | No | Solid second choice; cleaner protocol than Binance. |
| **Bybit WebSocket** | Free | ✅ | ✅ | No | No | Derivatives-flavoured; useful if we ever do futures. |
| **OKX WebSocket** | Free | ✅ | ✅ | No | No | Good Asia-hours coverage. |
| **Bitstamp WebSocket** | Free | ✅ | ✅ | No | No | EU-flavoured. |
| **LOBSTER** academic samples | Free | ❌ historical | ✅ | ✅ | None for samples | NASDAQ-derived limit-order-book CSVs for AAPL/AMZN/GOOG/INTC/MSFT etc. at multiple depth levels. **The single most useful free real-equity-data source for this project.** See dedicated section below. |
| **Databento** free clips | Free trial / signup | Historical clips | ✅ | ✅ (raw ITCH) | Account signup | Lets you pull small free clips of `XNAS.ITCH` (NASDAQ TotalView-ITCH binary) without subscription. Sign-up required, no card. |
| **NASDAQ ITCH 5.0 (direct from NASDAQ)** | Not directly free | — | — | — | NASDAQ creds | Historical SFTP requires NASDAQ credentials per the ITCHFTP spec; the binary spec PDF itself is free, the *data* is not. Use LOBSTER (above) or Databento (above) instead. |
| **Polygon.io** free tier | Free (rate-limited) | Delayed 15min | ✅ aggregates | ❌ | API key | Useful for OHLCV; not real-time depth. |
| **Alpaca Markets** | Free with brokerage account | ✅ | ✅ IEX-only | ❌ | Account signup | US equities through IEX feed. Free tier doesn't carry SIP. |
| **IEX Cloud** | Free (rate-limited) | ✅ IEX | Limited | ❌ | API key | IEX-routed flow only. |
| **Yahoo Finance via `yfinance`** | Free | Delayed | ❌ | ❌ | No | OHLCV only; insufficient for matching-engine validation. |
| **OpenBB Terminal** | Free | Aggregates above | Aggregates | ❌ | Per-source | A meta-source; useful for analytics, not for raw depth. |
| **CFTC Commitment of Traders** | Free | Weekly | ❌ | ❌ | No | Futures positioning data; out-of-scope for matching engine. |
| **FRED (Federal Reserve)** | Free | Daily | ❌ | ❌ | No | Macro / treasury rates; out-of-scope. |

### Why crypto is the right primary

- **Truly free.** No academic gating, no API key required for public market-data channels.
- **Real-time.** Sub-second order-book updates. Same quality you'd get from a paid equity feed.
- **Full L2.** Snapshot + delta protocol; reconstruct the book exactly.
- **24/7.** No market-hours problem; can demo any time of day.
- **High activity.** Major venues do $10–50B daily volume each.
- **Cross-venue arbitrage demos.** Easy to run two WebSocket connections (Coinbase + Binance) and show price differences in real-time — that's a Tier-2 portfolio demo unto itself.

### Why ITCH samples are still worth pursuing

- **Equity-microstructure literacy.** A candidate who has touched real ITCH parses interview-stronger than one who has only handled JSON WebSockets.
- **L3 (per-order, not just per-level).** Most crypto venues only give L2 deltas; LOBSTER and ITCH give per-order add/cancel/execute messages. The matching engine's full FIFO-within-level invariant can only be validated against L3 data.
- **Equities are what most HFT firms talk about.** Even though the engineering generalises, the language of US equity microstructure (ITCH 5.0 message types, `MarketCenter` codes, ARCA vs NASDAQ vs IEX routing) is the lingua franca.

### LOBSTER — the practical equity-data path

Verified May 2026: [data.lobsterdata.com](https://data.lobsterdata.com/) provides free sample CSVs without an institutional email. The samples are reconstructed from NASDAQ Historical TotalView-ITCH archives — same data underneath, friendlier format on top.

**Resources:**
- [LOBSTER home](https://data.lobsterdata.com/) — main entry point
- [Sample files page](https://data.lobsterdata.com/info/DataSamples.php) — direct download links for the free samples
- [Data structure documentation](https://data.lobsterdata.com/info/DataStructure.php) — the CSV column spec
- [LOBSTER reconstruction paper (PDF)](https://data.lobsterdata.com/info/docs/LobsterReport.pdf) — what they're actually computing

**What you get per sample day per ticker:**
- `<ticker>_<date>_<starttime>_<endtime>_message_<level>.csv` — every order-book event (add, cancel, modify, execute) with timestamp, type, order id, size, price, direction.
- `<ticker>_<date>_<starttime>_<endtime>_orderbook_<level>.csv` — the resulting top-N levels of the book after each event, where N is 1, 5, 10, 30, or 50.

**Why this is gold for our matching engine:** the canonical end-to-end correctness test is "feed the messages through our engine; compare our reconstructed top-N book against LOBSTER's `orderbook` CSV row-for-row." If they match, our engine reproduces NASDAQ behaviour faithfully on real flow. That's a single-paragraph CV bullet that's hard to argue with.

**Scope of free samples (subject to LOBSTER's current policy):** typically ~5 large-cap tickers, 1 trading day per ticker, depth levels 1 / 5 / 10. Plenty for a validation harness.

**Format quirks worth knowing before parsing:**
- Timestamps are seconds-since-midnight (with fractional seconds), not ns since epoch. Convert at the boundary.
- Direction `1 = buy, -1 = sell` (different from our internal `Side`).
- Event type encoding: `1 = submit, 2 = partial cancel, 3 = full cancel, 4 = visible execute, 5 = hidden execute, 7 = trading halt`.
- Hidden-order executions (type 5) appear in the `message` file but reference orders that were never visible in the book — these are the intentional discrepancies a public-market matching engine cannot reproduce. Document and skip rather than try to match them.

### Reference Rust + ITCH parsers (for cross-validation only)

When we eventually parse raw ITCH 5.0 binary frames (per `plans/itch-replay-harness.md` Phase 2), several open-source parsers exist for cross-validation. **We will not vendor any of these — the matching-engine project is from-scratch by design.** They exist only as oracles for "did I parse this message correctly":

- [shawfdong/itch5parser](https://github.com/shawfdong/itch5parser) — has C, Go, *and Rust* parsers in one repo. Most useful as a Rust reference.
- [bbalouki/itch](https://github.com/bbalouki/itch) — another ITCH 5.0 parser.
- [ZhexiongLiu/Nasdaq-ITCH-5.0](https://github.com/ZhexiongLiu/Nasdaq-ITCH-5.0) — Python parser.
- [stefan-jansen/machine-learning-for-trading — ITCH notebook](https://github.com/stefan-jansen/machine-learning-for-trading/blob/main/02_market_and_fundamental_data/01_NASDAQ_TotalView-ITCH_Order_Book/01_parse_itch_order_flow_messages.ipynb) — canonical "parse ITCH" Jupyter walkthrough; useful as an oracle for what each message type should produce.

**The official ITCH 5.0 spec PDF is free** at [nasdaqtrader.com/.../NQTVITCHSpecification.pdf](https://www.nasdaqtrader.com/content/technicalsupport/specifications/dataproducts/NQTVITCHSpecification.pdf) — that's all you need to write a parser, no sample data required to start.

## 2. Rationale

The free-data constraint is real money: a Bloomberg Terminal seat is $30k/year, a Refinitiv subscription is comparable, and even budget tier-2 vendors (Databento, Polygon paid) cost $50–500/month. Holding the line on zero cost means picking sources strategically:

- **For "the engine works on real data":** crypto WebSocket. Hooks up in a day.
- **For "the engine handles equity microstructure":** ITCH samples. One weekend to write a parser.
- **For "I understand cross-venue":** two crypto WebSockets running simultaneously.

## 3. What Was Tried

Nothing yet. Synthetic flow is the current default.

## 4. Guiding Principles

- **No subscriptions, ever.** Cancellable trials don't count. Free-tier with rate limits is acceptable as long as the project still functions when the limit hits.
- **Public, no-auth feeds preferred.** Anything that requires an account introduces friction for someone cloning the repo. Coinbase/Binance/Kraken public market-data channels qualify.
- **Document the source.** When we pull a sample file, commit a `data/README.md` describing which exchange, which day, which symbol, and where the file came from.
- **Don't ship copyrighted samples in the repo.** Reference scripts that fetch the data on demand. ITCH samples in particular are sometimes provided under restrictive licenses; check before committing.
- **Crypto first, equities second.** Crypto is free + clean + real-time. Equities are higher-prestige but harder to source.

## 5. Trade-offs and Constraints

- **No SIP-quality equity data.** Without paying, we cannot get Consolidated Tape or full NBBO across all US venues. IEX-only data through Alpaca is technically free, but IEX is ~3% of US equity volume.
- **WebSocket rate limits.** Coinbase Advanced Trade limits unauthenticated public connections to a few hundred subscriptions per IP. Binance to ~5 connections per IP. Sufficient for our purposes; not sufficient for production HFT.
- **No co-located feeds.** We will see the data with internet round-trip latency added (~50ms typical). The matching engine's *internal* latency is unaffected; the external timing is for pretty graphs only.
- **Market hours and downtime.** US equities are 9:30 AM – 4:00 PM ET, Mon–Fri. Crypto is 24/7. If we want a "always works for the demo" experience, crypto wins.

## 6. Open Questions

- **Which crypto venue first?** Coinbase has the cleanest API, Binance has higher volume. Recommend starting with Coinbase. Tracked in `plans/live-crypto-feed.md`.
- **Will LOBSTER's per-file license allow committing sample data to the repo?** Verified May 2026 that the samples are downloadable without institutional email; the per-file license terms still need a read before committing any sample file. Default to "fetch script", not "committed data".
- **Does Databento's free-clip access drop ITCH binary directly, or does it transform first?** Worth a quick test before relying on it as the "raw ITCH" source.
- **Is there a free L3 feed for crypto?** dYdX historically exposed order-level data through its WebSocket; check whether the v4 protocol still does.

## 7. Related Systems and Notes

- `notes/hft-firm-priorities.md` — why this matters (firms watch microstructure metrics that depend on real data).
- `plans/live-crypto-feed.md` — concrete plan for hooking up a WebSocket feed.
- `plans/itch-replay-harness.md` — concrete plan for ITCH replay.
