//! `Market` — multi-instrument routing layer.
//!
//! Holds one [`OrderBook`] per [`Symbol`]. New symbols are auto-registered
//! on first submit; existing symbols route to their existing book.

use std::collections::BTreeMap;

use crate::book::order_book::{OrderBook, SubmitResult};
use crate::errors::NyquestroResult;
use crate::events::OrderEvent;
use crate::order::Order;
use crate::types::{OrderID, Px, Qty, Symbol, Ts};

#[derive(Debug, Clone, Default)]
pub struct Market {
    books: BTreeMap<Symbol, OrderBook>,
}

impl Market {
    pub fn new() -> Self {
        Market::default()
    }

    /// Pre-register a symbol. Useful when you want the symbol to appear in
    /// the dashboard even before the first order arrives.
    pub fn register(&mut self, symbol: Symbol) -> &mut OrderBook {
        self.books.entry(symbol).or_insert_with(|| OrderBook::new(symbol))
    }

    pub fn book(&self, symbol: Symbol) -> Option<&OrderBook> {
        self.books.get(&symbol)
    }

    pub fn book_mut(&mut self, symbol: Symbol) -> Option<&mut OrderBook> {
        self.books.get_mut(&symbol)
    }

    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.books.keys()
    }

    pub fn books(&self) -> impl Iterator<Item = (&Symbol, &OrderBook)> {
        self.books.iter()
    }

    pub fn len(&self) -> usize {
        self.books.len()
    }

    pub fn is_empty(&self) -> bool {
        self.books.is_empty()
    }

    /// Submit an order. The book for the order's symbol is auto-registered
    /// if it doesn't already exist.
    pub fn submit_limit(&mut self, order: Order) -> NyquestroResult<SubmitResult> {
        let symbol = order.symbol();
        let book = self
            .books
            .entry(symbol)
            .or_insert_with(|| OrderBook::new(symbol));
        book.submit_limit(order)
    }

    /// Cancel an order. The caller must specify the symbol because order
    /// ids are not globally unique across symbols (different books may
    /// reuse them).
    pub fn cancel(&mut self, symbol: Symbol, id: OrderID, ts: Ts) -> NyquestroResult<OrderEvent> {
        let book = self.books.get_mut(&symbol).ok_or_else(|| {
            crate::errors::NyquestroError::SymbolMismatch {
                expected: symbol.as_u64(),
                actual: 0,
            }
        })?;
        book.cancel(id, ts)
    }

    /// Aggregate top-of-book best bid across all symbols. Returns the
    /// (symbol, price, qty) triple of the symbol with the highest bid.
    pub fn aggregate_best_bid(&self) -> Option<(Symbol, Px, Qty)> {
        self.books
            .iter()
            .filter_map(|(s, b)| b.best_bid().map(|(p, q)| (*s, p, q)))
            .max_by_key(|(_, p, _)| p.cents())
    }

    pub fn aggregate_best_ask(&self) -> Option<(Symbol, Px, Qty)> {
        self.books
            .iter()
            .filter_map(|(s, b)| b.best_ask().map(|(p, q)| (*s, p, q)))
            .min_by_key(|(_, p, _)| p.cents())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{OrderID, Px, Qty, Side, Ts};

    fn buy(symbol: Symbol, id: u64, price: u64, qty: u32, ts: u64) -> Order {
        Order::new(
            OrderID::new(id).unwrap(),
            symbol,
            Side::Buy,
            Px::from_cents(price).unwrap(),
            Qty::new(qty),
            Ts::from_nanos(ts),
        )
        .unwrap()
    }

    #[test]
    fn empty_market() {
        let m = Market::new();
        assert!(m.is_empty());
        assert_eq!(m.len(), 0);
    }

    #[test]
    fn auto_register_on_submit() {
        let mut m = Market::new();
        let aapl = Symbol::from_const("AAPL");
        m.submit_limit(buy(aapl, 1, 15000, 5, 1)).unwrap();
        assert_eq!(m.len(), 1);
        assert!(m.book(aapl).is_some());
    }

    #[test]
    fn separate_books_per_symbol() {
        let mut m = Market::new();
        let aapl = Symbol::from_const("AAPL");
        let msft = Symbol::from_const("MSFT");
        m.submit_limit(buy(aapl, 1, 15000, 5, 1)).unwrap();
        m.submit_limit(buy(msft, 2, 30000, 3, 2)).unwrap();
        assert_eq!(m.len(), 2);
        assert_eq!(m.book(aapl).unwrap().best_bid().unwrap().0.cents(), 15000);
        assert_eq!(m.book(msft).unwrap().best_bid().unwrap().0.cents(), 30000);
    }
}
