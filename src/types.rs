use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderID(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Px(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Qty(u32);

/// A timestamp representing nanoseconds since the UNIX epoch.
///
/// TS is basically a lightweight wrapper around a u64 value in nanoseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ts(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    Open,
    PartiallyFilled,
    FullyFilled,
    Cancelled,
}

impl OrderID {
    pub fn new(id: u64) -> Result<Self, &'static str> {
        if id == 0 {
            Err("OrderID cannot be zero.")
        } else {
            Ok(OrderID(id))
        }
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Side {
    pub fn opposite(&self) -> Self {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

impl Px {
    pub fn new_from_dollars(dollar_price: f64) -> Result<Self, &'static str> {
        if dollar_price > 0.0 {
            Ok(Px((dollar_price * 100.0) as u64))
        } else {
            Err("Invalid price in dollars, price can't be negative.")
        }
    }

    pub fn new_from_cents(cent_price: u64) -> Result<Self, &'static str> {
        if cent_price > 0 {
            Ok(Px(cent_price))
        } else {
            Err("Invalid price.")
        }
    }

    pub fn to_dollars(&self) -> f64 {
        (self.0 as f64) / 100.0
    }

    pub fn to_cents(&self) -> u64 {
        self.0
    }
}

impl Qty {
    pub fn new(value: u32) -> Self {
        Qty(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }

    pub fn can_subtract(&self, other: Qty) -> bool {
        if self.value() < other.value() {
            false
        } else {
            true
        }
    }

    pub fn saturating_sub(&self, other: Qty) -> Qty {
        let result = if self.value() > other.value() {
            self.value() - other.value()
        } else {
            0
        };

        Qty(result)
    }
}

impl Ts {
    pub fn now() -> Self {
        let time_now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        Ts(time_now)
    }

    pub fn from_nanos(nanos: u64) -> Self {
        Ts(nanos)
    }

    pub fn is_before(&self, timestamp: u64) -> bool {
        if self.0 < timestamp { true } else { false }
    }

    pub fn is_after(&self, timestamp: u64) -> bool {
        if self.0 > timestamp { true } else { false }
    }

    pub fn duration_since(&self, time_in_nanos: u64) -> u64 {
        if self.is_after(time_in_nanos) {
            self.nanos() - time_in_nanos
        } else {
            time_in_nanos - self.nanos()
        }
    }

    pub fn nanos(&self) -> u64 {
        self.0
    }

    pub fn micros(&self) -> u64 {
        self.0 / 1000
    }

    pub fn millis(&self) -> u64 {
        self.0 / 1000000
    }

    pub fn to_utc_datetime(&self) -> DateTime<Utc> {
        let system_time = UNIX_EPOCH + Duration::from_nanos(self.0);
        system_time.into()
    }
}
