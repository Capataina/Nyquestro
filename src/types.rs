//! Core domain primitives.
//!
//! Each type is a thin wrapper around a numeric value, with the invariants
//! enforced at construction time. The wrappers exist so the compiler treats
//! `OrderID`, `Px`, `Qty`, and `Ts` as mutually incompatible — accidentally
//! passing a price where a quantity is expected is a type error, not a
//! runtime bug.

use std::fmt;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};

use crate::errors::{NyquestroError, NyquestroResult};

// ─── Symbol ─────────────────────────────────────────────────────────────────

/// 8-byte ASCII-packed instrument identifier. Encodes a string of up to 8
/// printable characters into a single `u64` so the type stays `Copy`,
/// `Ord`, and 8-byte aligned. The packing is big-endian, so lexicographic
/// `Ord` on the underlying `u64` matches lexicographic order on the string.
///
/// Example: `Symbol::from_const("AAPL")` packs to `0x4141504C00000000`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Symbol(u64);

impl Symbol {
    /// Construct from a packed 8-byte big-endian buffer. Mirrors what
    /// [`Symbol::as_bytes`] returns. Used by the live-feed bridge to map
    /// venue product ids (e.g. "BTC-USD") into a `Symbol` at runtime
    /// without going through the `&str`-validating constructor.
    #[inline]
    pub const fn from_const_bytes(bytes: [u8; 8]) -> Self {
        Symbol(u64::from_be_bytes(bytes))
    }

    /// Const-context constructor. Truncates to 8 bytes silently, which is
    /// fine for compile-time string literals the author knows are short.
    pub const fn from_const(s: &str) -> Self {
        let bytes = s.as_bytes();
        let mut buf = [0u8; 8];
        let mut i = 0;
        while i < bytes.len() && i < 8 {
            buf[i] = bytes[i];
            i += 1;
        }
        Symbol(u64::from_be_bytes(buf))
    }

    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Decode the packed bytes into a UTF-8 `&str` slice. The slice
    /// length is the number of leading non-zero bytes.
    pub fn as_bytes(&self) -> [u8; 8] {
        self.0.to_be_bytes()
    }
}

impl FromStr for Symbol {
    type Err = NyquestroError;
    /// Construct from a `&str`. Rejects empty input and strings longer
    /// than 8 bytes. Non-ASCII bytes pass through; the type does not
    /// enforce printable-only.
    fn from_str(s: &str) -> NyquestroResult<Self> {
        let bytes = s.as_bytes();
        if bytes.is_empty() || bytes.len() > 8 {
            return Err(NyquestroError::InvalidSymbol);
        }
        let mut buf = [0u8; 8];
        buf[..bytes.len()].copy_from_slice(bytes);
        Ok(Symbol(u64::from_be_bytes(buf)))
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buf = self.as_bytes();
        let end = buf.iter().position(|&b| b == 0).unwrap_or(8);
        match std::str::from_utf8(&buf[..end]) {
            Ok(s) => f.write_str(s),
            Err(_) => write!(f, "0x{:016X}", self.0),
        }
    }
}

// ─── OrderID ────────────────────────────────────────────────────────────────

/// Unique identifier for an order. Zero is reserved as a sentinel and not
/// representable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OrderID(u64);

impl OrderID {
    pub fn new(id: u64) -> NyquestroResult<Self> {
        if id == 0 {
            Err(NyquestroError::InvalidOrderId)
        } else {
            Ok(OrderID(id))
        }
    }

    #[inline]
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl fmt::Display for OrderID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

// ─── Side ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    #[inline]
    pub const fn opposite(self) -> Self {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }

    #[inline]
    pub const fn is_buy(self) -> bool {
        matches!(self, Side::Buy)
    }

    #[inline]
    pub const fn is_sell(self) -> bool {
        matches!(self, Side::Sell)
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Side::Buy => "BUY",
            Side::Sell => "SELL",
        })
    }
}

// ─── Px ─────────────────────────────────────────────────────────────────────

/// Price in integer cents. Float arithmetic is never used for price comparison
/// — only for human-friendly display via [`Px::to_dollars`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Px(u64);

impl Px {
    /// Construct from raw cents. Zero is rejected.
    pub fn from_cents(cents: u64) -> NyquestroResult<Self> {
        if cents == 0 {
            Err(NyquestroError::InvalidPrice { cents: 0 })
        } else {
            Ok(Px(cents))
        }
    }

    /// Construct from a dollar value, rounding to the nearest cent.
    /// Rejects NaN, infinities, zero, and negative values.
    pub fn from_dollars(dollars: f64) -> NyquestroResult<Self> {
        if !dollars.is_finite() || dollars <= 0.0 {
            return Err(NyquestroError::InvalidPriceFloat { value: dollars });
        }
        let scaled = (dollars * 100.0).round();
        if scaled <= 0.0 || scaled > u64::MAX as f64 {
            return Err(NyquestroError::InvalidPriceFloat { value: dollars });
        }
        Ok(Px(scaled as u64))
    }

    #[inline]
    pub const fn cents(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn to_dollars(self) -> f64 {
        self.0 as f64 / 100.0
    }
}

impl fmt::Display for Px {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:.2}", self.to_dollars())
    }
}

// ─── Qty ────────────────────────────────────────────────────────────────────

/// Quantity in whole units. Zero is representable (e.g. `remaining_quantity`
/// after a full fill); rejection of zero is enforced at the construction
/// boundaries that need it (`Order::new`, event constructors).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Qty(u32);

impl Qty {
    pub const ZERO: Qty = Qty(0);

    #[inline]
    pub const fn new(value: u32) -> Self {
        Qty(value)
    }

    #[inline]
    pub const fn value(self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Subtract `other` from `self`. Returns `None` when `other > self` —
    /// callers are expected to check explicitly rather than rely on
    /// saturating arithmetic, which has historically masked over-fill bugs.
    #[inline]
    pub fn checked_sub(self, other: Qty) -> Option<Qty> {
        self.0.checked_sub(other.0).map(Qty)
    }

    #[inline]
    pub fn checked_add(self, other: Qty) -> Option<Qty> {
        self.0.checked_add(other.0).map(Qty)
    }
}

impl fmt::Display for Qty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ─── Ts ─────────────────────────────────────────────────────────────────────

/// Nanosecond-resolution timestamp since the UNIX epoch. Stored as `u64`,
/// which wraps in ~584 years from epoch — not a practical concern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ts(u64);

impl Ts {
    /// Wall-clock now. If the system clock is before the UNIX epoch (an
    /// extreme edge case on misconfigured hardware) this returns `Ts(0)`
    /// rather than panicking.
    pub fn now() -> Self {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| Ts(d.as_nanos() as u64))
            .unwrap_or(Ts(0))
    }

    #[inline]
    pub const fn from_nanos(nanos: u64) -> Self {
        Ts(nanos)
    }

    #[inline]
    pub const fn nanos(self) -> u64 {
        self.0
    }

    #[inline]
    pub const fn micros(self) -> u64 {
        self.0 / 1_000
    }

    #[inline]
    pub const fn millis(self) -> u64 {
        self.0 / 1_000_000
    }

    /// Absolute nanoseconds between two timestamps. Symmetric — order does
    /// not matter.
    #[inline]
    pub fn duration_since(self, other: Ts) -> u64 {
        self.0.max(other.0) - self.0.min(other.0)
    }

    pub fn to_utc_datetime(self) -> DateTime<Utc> {
        let when = UNIX_EPOCH + Duration::from_nanos(self.0);
        when.into()
    }
}

impl fmt::Display for Ts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}ns", self.0)
    }
}

// ─── Status ─────────────────────────────────────────────────────────────────

/// Lifecycle state of an order. The state machine is one-way:
///
/// ```text
///                ┌─→ PartiallyFilled ─→ FullyFilled
/// Open  ────────┤                              │
///                └─────────────────────────────┴──→ (terminal)
///                │
///                └──────────────────────────────→ Cancelled (terminal)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Status {
    Open,
    PartiallyFilled,
    FullyFilled,
    Cancelled,
}

impl Status {
    #[inline]
    pub const fn is_active(self) -> bool {
        matches!(self, Status::Open | Status::PartiallyFilled)
    }

    #[inline]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Status::FullyFilled | Status::Cancelled)
    }

    /// Whether the transition `self -> next` is valid under the one-way state
    /// machine. Used by `Order` to reject backwards transitions.
    pub const fn can_transition_to(self, next: Status) -> bool {
        matches!(
            (self, next),
            (Status::Open, Status::Open)
                | (Status::PartiallyFilled, Status::PartiallyFilled)
                | (Status::FullyFilled, Status::FullyFilled)
                | (Status::Cancelled, Status::Cancelled)
                | (Status::Open, Status::PartiallyFilled)
                | (Status::Open, Status::FullyFilled)
                | (Status::Open, Status::Cancelled)
                | (Status::PartiallyFilled, Status::FullyFilled)
                | (Status::PartiallyFilled, Status::Cancelled)
        )
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Status::Open => "OPEN",
            Status::PartiallyFilled => "PARTIAL",
            Status::FullyFilled => "FILLED",
            Status::Cancelled => "CANCELLED",
        })
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_round_trip() {
        let s: Symbol = "AAPL".parse().unwrap();
        assert_eq!(format!("{s}"), "AAPL");
        assert_eq!(Symbol::from_const("AAPL"), s);
    }

    #[test]
    fn symbol_rejects_empty_or_too_long() {
        assert!("".parse::<Symbol>().is_err());
        assert!("TOOOOOLONGSYMBOL".parse::<Symbol>().is_err());
        assert!("EIGHTLET".parse::<Symbol>().is_ok());
        assert!("NINELETTR".parse::<Symbol>().is_err());
    }

    #[test]
    fn symbol_lexicographic_ord() {
        assert!(Symbol::from_const("AAPL") < Symbol::from_const("MSFT"));
        assert!(Symbol::from_const("MSFT") < Symbol::from_const("NVDA"));
    }

    #[test]
    fn order_id_rejects_zero() {
        assert!(matches!(
            OrderID::new(0),
            Err(NyquestroError::InvalidOrderId)
        ));
        assert_eq!(OrderID::new(42).unwrap().value(), 42);
    }

    #[test]
    fn side_opposite_is_involution() {
        assert_eq!(Side::Buy.opposite(), Side::Sell);
        assert_eq!(Side::Sell.opposite(), Side::Buy);
        assert_eq!(Side::Buy.opposite().opposite(), Side::Buy);
    }

    #[test]
    fn px_from_cents_rejects_zero() {
        assert!(Px::from_cents(0).is_err());
        assert_eq!(Px::from_cents(1).unwrap().cents(), 1);
        assert_eq!(Px::from_cents(12_345).unwrap().to_dollars(), 123.45);
    }

    #[test]
    fn px_from_dollars_rounds_to_nearest_cent() {
        assert_eq!(Px::from_dollars(10.999).unwrap().cents(), 1100);
        assert_eq!(Px::from_dollars(10.991).unwrap().cents(), 1099);
        assert_eq!(Px::from_dollars(10.005).unwrap().cents(), 1001);
    }

    #[test]
    fn px_from_dollars_rejects_invalid_inputs() {
        assert!(Px::from_dollars(0.0).is_err());
        assert!(Px::from_dollars(-5.0).is_err());
        assert!(Px::from_dollars(f64::NAN).is_err());
        assert!(Px::from_dollars(f64::INFINITY).is_err());
        assert!(Px::from_dollars(0.001).is_err()); // rounds to 0 cents
    }

    #[test]
    fn px_ordering_works_on_cents() {
        assert!(Px::from_cents(100).unwrap() < Px::from_cents(200).unwrap());
        assert!(Px::from_cents(200).unwrap() > Px::from_cents(100).unwrap());
    }

    #[test]
    fn qty_zero_representable_arithmetic_checked() {
        assert!(Qty::ZERO.is_zero());
        assert_eq!(Qty::new(10).checked_sub(Qty::new(3)), Some(Qty::new(7)));
        assert_eq!(Qty::new(3).checked_sub(Qty::new(10)), None); // over-subtract
        assert_eq!(Qty::new(u32::MAX).checked_add(Qty::new(1)), None); // overflow
    }

    #[test]
    fn ts_now_is_post_2020() {
        // 2020-01-01T00:00:00 UTC in nanoseconds since epoch.
        const Y2020_NANOS: u64 = 1_577_836_800 * 1_000_000_000;
        assert!(Ts::now().nanos() > Y2020_NANOS);
    }

    #[test]
    fn ts_duration_since_is_symmetric() {
        let a = Ts::from_nanos(1_000);
        let b = Ts::from_nanos(5_000);
        assert_eq!(a.duration_since(b), 4_000);
        assert_eq!(b.duration_since(a), 4_000);
    }

    #[test]
    fn status_one_way_transitions() {
        assert!(Status::Open.can_transition_to(Status::PartiallyFilled));
        assert!(Status::Open.can_transition_to(Status::FullyFilled));
        assert!(Status::Open.can_transition_to(Status::Cancelled));
        assert!(Status::PartiallyFilled.can_transition_to(Status::FullyFilled));
        assert!(Status::PartiallyFilled.can_transition_to(Status::Cancelled));

        // Backwards / out-of-terminal forbidden.
        assert!(!Status::PartiallyFilled.can_transition_to(Status::Open));
        assert!(!Status::FullyFilled.can_transition_to(Status::Open));
        assert!(!Status::FullyFilled.can_transition_to(Status::PartiallyFilled));
        assert!(!Status::Cancelled.can_transition_to(Status::Open));
    }

    #[test]
    fn status_classification() {
        assert!(Status::Open.is_active());
        assert!(Status::PartiallyFilled.is_active());
        assert!(!Status::FullyFilled.is_active());
        assert!(Status::FullyFilled.is_terminal());
        assert!(Status::Cancelled.is_terminal());
    }
}
