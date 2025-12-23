# 002: Deterministic Matcher Loop

**Status:** planned  
**Last Updated:** 2025

---

## Goal and Scope

### What Will Exist When Complete

1. **OrderBook Structure**

   - Single-instrument order book (one book = one instrument)
   - Bid side (buyers) and ask side (sellers) as separate price-sorted collections
   - Price-time priority matching (best price first, then FIFO within price)

2. **Deterministic Matcher Loop**

   - Takes incoming order and matches against opposite side of book
   - Price compatibility: buy_price >= sell_price for matching
   - Partial fill handling (order can match multiple counterparties)
   - Creates `FillEvent` for each match (with correct buyer/seller order IDs)
   - Emits `QuoteEvent` when best bid/ask changes

3. **Order State Management Fix**
   - `Order::fill()` refactored to only update order state (no `FillEvent` creation)
   - Matcher creates `FillEvent` when pairing two orders
   - Proper validation of fill quantities

### Explicitly Out of Scope

- Lock-free data structures (uses current `Vec`-based `PriceLevel`, will refactor later)
- Multiple instruments (single-instrument order book)
- Order types beyond Limit orders (Market, IOC, FOK come later)
- Cancellation logic (separate feature)
- Self-match prevention (separate feature)
- Event serialization/publishing (events are in-memory only)

### Deliverables Checklist

- [ ] `OrderBook` struct with bid/ask sides
- [ ] Price-time priority ordering (best price first, FIFO within price)
- [ ] `Order::fill()` refactored to return `()` instead of `FillEvent`
- [ ] Matcher loop that pairs buy and sell orders
- [ ] Partial fill handling (order matches multiple counterparties)
- [ ] `FillEvent` creation with correct buyer/seller IDs
- [ ] `QuoteEvent` emission when best bid/ask changes
- [ ] Unit tests for order book operations
- [ ] Integration tests for matching scenarios

---

## Context and Justification

### Why This Work Matters Architecturally

**Core Matching Logic:**

- This is the heart of the matching engine – without it, we have no trading functionality
- Validates the event infrastructure (`FillEvent`, `QuoteEvent`) in real scenarios
- Establishes the matching algorithm that all future order types will build upon

**Design Decisions:**

- Single-instrument order book simplifies MVP and matches common practice
- Price-time priority is the standard matching algorithm (best price wins, then time priority)
- Matcher creates events (not `Order::fill()`) because events represent matches between two orders

### What This Enables Next

- Limit order matching (foundation for all order types)
- Market order support (can use same matcher with market price)
- IOC/FOK order types (extend matcher with time-in-force logic)
- Performance optimisation (lock-free structures can replace `Vec` without changing matcher logic)
- Event publishing (matcher emits events that can be consumed by publishers)

### Alternatives Considered and Why Rejected

1. **Multi-instrument order book with symbol field**

   - Rejected: Adds complexity, overhead (symbol checks in hot path), not needed for MVP
   - Can add later if needed (single-instrument is standard practice)

2. **`Order::fill()` creates `FillEvent`**

   - Rejected: `FillEvent` represents a match between two orders, not a single order being filled
   - Matcher should create events when pairing orders

3. **Pro-rata matching (instead of price-time)**
   - Rejected: Price-time is standard, simpler, and sufficient for MVP
   - Pro-rata can be added later as alternative matching algorithm

### Assumptions and Constraints to Validate

- [ ] Single-instrument order book is sufficient for MVP
- [ ] Price-time priority matching is correct algorithm
- [ ] Current `Vec`-based `PriceLevel` is acceptable for initial implementation
- [ ] Partial fills are handled correctly (order can match multiple counterparties)
- [ ] Best bid/ask calculation is correct for `QuoteEvent` emission

---

## Interfaces and Contracts

### Public APIs to Add/Change

**New Module: `src/order_book.rs`**

```rust
pub struct OrderBook {
    bids: BTreeMap<Px, PriceLevel>,  // Buy side (highest price first)
    asks: BTreeMap<Px, PriceLevel>,  // Sell side (lowest price first)
}

impl OrderBook {
    pub fn new() -> Self;
    pub fn add_order(&mut self, order: Order) -> NyquestroResult<Vec<FillEvent>>;
    pub fn get_best_bid(&self) -> Option<(Px, Qty)>;
    pub fn get_best_ask(&self) -> Option<(Px, Qty)>;
}
```

**Modified: `src/order.rs`**

```rust
impl Order {
    // Changed: returns () instead of FillEvent
    pub fn fill(&mut self, fill_amount: Qty) -> NyquestroResult<()>;
}
```

**New: Matcher Logic (internal to `OrderBook`)**

- `match_order()` – core matching algorithm
- `find_matching_orders()` – finds compatible orders on opposite side
- `create_fill_event()` – creates `FillEvent` with correct buyer/seller IDs
- `update_quote_events()` – determines if best bid/ask changed

### Invariants That Must Hold

- Best bid price <= best ask price (no crossed book)
- Orders on bid side have `Side::Buy`
- Orders on ask side have `Side::Sell`
- `FillEvent` always has different buyer and seller order IDs
- `FillEvent` quantity > 0
- Order `remaining_quantity` never goes negative
- Price-time priority: best price first, then FIFO within price

### Must-Not-Break Contracts

- Existing `Order` API (except `fill()` return type change)
- Existing `PriceLevel` API
- Existing event types (`FillEvent`, `QuoteEvent`)
- `NyquestroResult<T>` type alias

---

## Impacted Areas

### Files/Modules Likely to Change

- [ ] `src/order.rs` – refactor `fill()` method
- [ ] `src/order_book.rs` – **NEW FILE** – order book structure and matcher
- [ ] `src/lib.rs` – export `order_book` module
- [ ] `src/main.rs` – update to use new `Order::fill()` signature
- [ ] `tests/matcher_tests.rs` – **NEW FILE** – matcher integration tests

### Data Model Changes

- No database/serialization changes (all in-memory)
- Order book structure is new (but uses existing `Order` and `PriceLevel`)

### API/CLI Changes

- `Order::fill()` return type changes: `NyquestroResult<FillEvent>` → `NyquestroResult<()>`
- Breaking change for any code calling `Order::fill()` (currently only `main.rs`)

---

## Incremental Implementation Plan

### Step 1: Refactor Order::fill()

**Intent:** Remove `FillEvent` creation from `Order::fill()`, make it only update order state  
**Expected Behaviour:** `Order::fill()` validates fill amount, updates `remaining_quantity` and `status`, returns `()`. No `FillEvent` creation.  
**Verification:** `cargo check` passes. `main.rs` updated to handle new signature. Existing tests updated.

**Checklist:**

- [ ] Change `Order::fill()` return type to `NyquestroResult<()>`
- [ ] Remove `FillEvent` creation from `Order::fill()`
- [ ] Add validation: fill_amount <= remaining_quantity
- [ ] Update `src/main.rs` to handle new signature
- [ ] Update any tests that call `Order::fill()`

### Step 2: Create OrderBook Structure

**Intent:** Create basic `OrderBook` with bid/ask sides  
**Expected Behaviour:** `OrderBook` has `bids` and `asks` as `BTreeMap<Px, PriceLevel>`. Can create new order book.  
**Verification:** `cargo check` passes. Can instantiate `OrderBook`.

**Checklist:**

- [ ] Create `src/order_book.rs` module
- [ ] Define `OrderBook` struct with bid/ask sides
- [ ] Implement `OrderBook::new()`
- [ ] Export module in `src/lib.rs`
- [ ] Basic unit test for creation

### Step 3: Implement Order Addition (No Matching Yet)

**Intent:** Add orders to correct side of book, validate side matches  
**Expected Behaviour:** `add_order()` places order on correct side (bid or ask) based on `order.side`. Validates order side matches book side. Returns empty `Vec<FillEvent>` for now.  
**Verification:** Orders added to correct side. Can retrieve orders from book.

**Checklist:**

- [ ] Implement `add_order()` that routes to bid/ask side
- [ ] Validate order side matches book side
- [ ] Use existing `PriceLevel::add_order()` for price level management
- [ ] Return empty `Vec<FillEvent>` (matching not implemented yet)
- [ ] Unit tests for order addition

### Step 4: Implement Best Bid/Ask Queries

**Intent:** Calculate best bid (highest buy price) and best ask (lowest sell price)  
**Expected Behaviour:** `get_best_bid()` returns highest price on bid side with total quantity. `get_best_ask()` returns lowest price on ask side with total quantity. Returns `None` if side is empty.  
**Verification:** Correct best prices returned. Handles empty book correctly.

**Checklist:**

- [ ] Implement `get_best_bid()` (highest price in bids map)
- [ ] Implement `get_best_ask()` (lowest price in asks map)
- [ ] Return `Option<(Px, Qty)>` (price and total quantity at that level)
- [ ] Unit tests for best bid/ask calculation

### Step 5: Implement Price Compatibility Check

**Intent:** Determine if two orders can match based on price  
**Expected Behaviour:** Buy order at price P can match sell order at price Q if P >= Q. Sell order at price P can match buy order at price Q if P <= Q.  
**Verification:** Price compatibility logic is correct for all scenarios.

**Checklist:**

- [ ] Helper function `can_match(buy_price, sell_price) -> bool`
- [ ] Handles edge cases (exact price match, price improvement)
- [ ] Unit tests for price compatibility

### Step 6: Implement Core Matching Loop

**Intent:** Match incoming order against opposite side, find compatible orders  
**Expected Behaviour:** For incoming order, find orders on opposite side that can match (price compatibility). Iterate through price levels in priority order (best price first). Within each price level, iterate orders FIFO.  
**Verification:** Correct orders found for matching. Price-time priority respected.

**Checklist:**

- [ ] `find_matching_orders()` – finds compatible orders on opposite side
- [ ] Respects price-time priority (best price first, then FIFO)
- [ ] Returns iterator or collection of matching orders
- [ ] Unit tests for matching order discovery

### Step 7: Implement Fill Event Creation

**Intent:** Create `FillEvent` with correct buyer/seller IDs when two orders match  
**Expected Behaviour:** Given two orders (one buy, one sell), determine which is buyer and which is seller based on `side` field. Create `FillEvent` with correct `buyer_order_id` and `seller_order_id`. Use match price (typically the resting order's price for price-time priority).  
**Verification:** `FillEvent` has different buyer/seller IDs. Correct price used.

**Checklist:**

- [ ] `create_fill_event(buy_order, sell_order, price, quantity) -> FillEvent`
- [ ] Correctly assigns buyer/seller IDs based on order sides
- [ ] Uses match price (resting order price for price-time priority)
- [ ] Unit tests for fill event creation

### Step 8: Implement Partial Fill Handling

**Intent:** Handle cases where order matches multiple counterparties or partial quantities  
**Expected Behaviour:** If incoming order quantity > matched order quantity, continue matching with next order. If matched order quantity > incoming order quantity, partially fill matched order. Update both orders' `remaining_quantity`. Create multiple `FillEvent`s if needed.  
**Verification:** Partial fills work correctly. Orders updated correctly. Multiple fills created when needed.

**Checklist:**

- [ ] Handle incoming order > matched order quantity (continue matching)
- [ ] Handle matched order > incoming order quantity (partial fill matched order)
- [ ] Update both orders' `remaining_quantity` correctly
- [ ] Remove fully filled orders from book
- [ ] Create `FillEvent` for each match
- [ ] Unit tests for partial fill scenarios

### Step 9: Integrate Matching into add_order()

**Intent:** Complete `add_order()` to actually match orders  
**Expected Behaviour:** When order added, match against opposite side. Create `FillEvent`s for all matches. Update order states. Remove fully filled orders. Return all `FillEvent`s created.  
**Verification:** Orders match correctly. All `FillEvent`s returned. Order states updated.

**Checklist:**

- [ ] Integrate matching loop into `add_order()`
- [ ] Match incoming order against opposite side
- [ ] Create and collect all `FillEvent`s
- [ ] Update order states (call `Order::fill()` on both orders)
- [ ] Remove fully filled orders from book
- [ ] Return `Vec<FillEvent>`

### Step 10: Implement QuoteEvent Emission

**Intent:** Emit `QuoteEvent` when best bid or ask changes  
**Expected Behaviour:** After matching, check if best bid or ask changed. If changed, create `QuoteEvent` for the new best price/quantity. Include side (Buy for bid, Sell for ask) and timestamp.  
**Verification:** `QuoteEvent` emitted when best bid/ask changes. Correct price/quantity/side.

**Checklist:**

- [ ] Track previous best bid/ask
- [ ] Compare with new best bid/ask after matching
- [ ] Create `QuoteEvent` if changed
- [ ] Include correct side (Buy for bid, Sell for ask)
- [ ] Return `QuoteEvent`s along with `FillEvent`s (or separate method)
- [ ] Unit tests for quote event emission

---

## Testing and Validation

### Unit Tests

- [ ] `Order::fill()` validates fill amount <= remaining quantity
- [ ] `Order::fill()` updates remaining_quantity correctly
- [ ] `Order::fill()` updates status correctly
- [ ] `OrderBook::new()` creates empty book
- [ ] `add_order()` places buy orders on bid side
- [ ] `add_order()` places sell orders on ask side
- [ ] `add_order()` rejects order with wrong side
- [ ] `get_best_bid()` returns highest price on bid side
- [ ] `get_best_ask()` returns lowest price on ask side
- [ ] `get_best_bid()` returns None for empty bid side
- [ ] `get_best_ask()` returns None for empty ask side
- [ ] Price compatibility: buy @ $100 matches sell @ $100
- [ ] Price compatibility: buy @ $100 matches sell @ $99
- [ ] Price compatibility: buy @ $100 does not match sell @ $101
- [ ] Fill event creation: correct buyer/seller IDs
- [ ] Fill event creation: correct price

### Integration Tests

- [ ] Simple match: buy @ $100 matches sell @ $100
- [ ] Price improvement: buy @ $100 matches sell @ $99 (buyer gets better price)
- [ ] Partial fill: buy 10 matches sell 5 (buy order partially filled)
- [ ] Multiple matches: buy 10 matches sell 3, then sell 4, then sell 3
- [ ] No match: buy @ $100 does not match sell @ $101
- [ ] Best bid/ask updates after matching
- [ ] QuoteEvent emitted when best bid changes
- [ ] QuoteEvent emitted when best ask changes
- [ ] Fully filled orders removed from book
- [ ] Partially filled orders remain in book

### Validation Commands

- `cargo test` – all tests pass
- `cargo check` – no compilation errors
- `cargo clippy` – no linter warnings

---

## Risks, Edge Cases, and Failure Modes

### Failure Modes

- [ ] **Order side mismatch** – order added to wrong side (buy order to ask side)

  - Detection: Validation in `add_order()`
  - Mitigation: Return error, reject order

- [ ] **Fill amount exceeds remaining quantity** – trying to fill more than available

  - Detection: Validation in `Order::fill()`
  - Mitigation: Return error, reject fill

- [ ] **Same order ID for buyer and seller in FillEvent**

  - Detection: Matcher should never pair order with itself
  - Mitigation: Matching logic ensures different orders

- [ ] **Negative remaining quantity**

  - Detection: Validation in `Order::fill()`
  - Mitigation: Saturating subtraction or error

- [ ] **Crossed book** – best bid > best ask
  - Detection: Should not happen with proper matching
  - Mitigation: Matching should prevent this

### Edge Cases

- Empty order book (no matches possible)
- Order matches entire opposite side (multiple fills)
- Order partially fills multiple counterparties
- Exact price match (buy @ $100, sell @ $100)
- Price improvement (buy @ $100 matches sell @ $99)
- Very large quantities (u32::MAX)
- Very high prices (u64::MAX cents)

### Detection Signals

- Unit tests catch invalid fill amounts
- Unit tests verify correct buyer/seller IDs
- Integration tests verify matching logic
- Integration tests verify quote event emission

### Mitigations

- Comprehensive validation in `Order::fill()`
- Matching logic ensures different orders paired
- Price compatibility checks prevent invalid matches
- Best bid/ask validation prevents crossed book

---

## Exit Criteria

### Correctness

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] `cargo check` passes with no errors
- [ ] Orders match correctly (price-time priority)
- [ ] `FillEvent` always has different buyer/seller IDs
- [ ] Partial fills handled correctly
- [ ] Best bid/ask calculated correctly

### Performance

- [ ] Order addition is O(log n) where n is number of price levels
- [ ] Matching is O(m) where m is number of matching orders
- [ ] No unnecessary allocations in hot path (uses existing `Vec` for now)

### Operability

- [ ] Error messages are descriptive
- [ ] Order states updated correctly
- [ ] Events emitted correctly

### Documentation

- [ ] Plan file updated with completion status
- [ ] `plans/README.md` updated with plan status
- [ ] Code comments explain matching algorithm

---

## Future Considerations

### Follow-on Work

- [ ] Lock-free data structures (replace `Vec` with atomic price buckets)
- [ ] Market orders (use matcher with market price)
- [ ] IOC/FOK order types (extend matcher with time-in-force)
- [ ] Cancellation logic (remove orders from book)
- [ ] Self-match prevention (check order IDs before matching)
- [ ] Pro-rata matching (alternative to price-time priority)
- [ ] Multi-instrument support (symbol field, multiple order books)

### Known Limitations

- Uses `Vec`-based `PriceLevel` (not lock-free, will need refactoring)
- Single-instrument only (no symbol field)
- Limit orders only (no Market, IOC, FOK)
- No cancellation (orders stay in book until filled)
- No self-match prevention (can match order with itself if same ID)

### Deliberate Technical Debt

- `Vec`-based price levels deferred until lock-free structures implemented
- Multi-instrument support deferred (single-instrument is sufficient for MVP)
- Advanced order types deferred (Limit orders are foundation)
- Cancellation deferred (separate feature)
