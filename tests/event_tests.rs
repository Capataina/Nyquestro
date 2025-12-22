#[cfg(test)]
mod event_tests {
    use nyquestro::errors::{ErrorSeverity, NyquestroError, severity};
    use nyquestro::events::fill_event::FillEvent;
    use nyquestro::events::order_event::{OrderEvent, OrderRejectionReason};
    use nyquestro::events::quote_event::QuoteEvent;
    use nyquestro::types::{OrderID, Px, Qty, Side, Ts};

    // ============================================================================
    // FillEvent Tests
    // ============================================================================

    #[test]
    fn test_fill_event_construction_valid() {
        let buyer_id = OrderID::new(1).unwrap();
        let seller_id = OrderID::new(2).unwrap();
        let price = Px::new_from_dollars(100.50).unwrap();
        let quantity = Qty::new(50);
        let timestamp = Ts::now();

        let fill_event = FillEvent::new(buyer_id, seller_id, price, quantity, timestamp)
            .expect("Valid FillEvent should be created");

        assert_eq!(fill_event.get_buyer_order_id(), buyer_id);
        assert_eq!(fill_event.get_seller_order_id(), seller_id);
        assert_eq!(fill_event.get_price(), price);
        assert_eq!(fill_event.get_quantity(), quantity);
        assert_eq!(fill_event.get_timestamp(), timestamp);
    }

    #[test]
    fn test_fill_event_rejects_zero_quantity() {
        let buyer_id = OrderID::new(1).unwrap();
        let seller_id = OrderID::new(2).unwrap();
        let price = Px::new_from_dollars(100.0).unwrap();
        let zero_quantity = Qty::new(0);
        let timestamp = Ts::now();

        let result = FillEvent::new(buyer_id, seller_id, price, zero_quantity, timestamp);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), NyquestroError::InvalidQuantity);
    }

    #[test]
    fn test_fill_event_with_different_prices() {
        let buyer_id = OrderID::new(10).unwrap();
        let seller_id = OrderID::new(20).unwrap();

        // Test with various price formats
        let price_cents = Px::new_from_cents(10000).unwrap();
        let price_dollars = Px::new_from_dollars(100.00).unwrap();

        assert_eq!(price_cents, price_dollars);

        let fill1 =
            FillEvent::new(buyer_id, seller_id, price_cents, Qty::new(100), Ts::now()).unwrap();

        let fill2 =
            FillEvent::new(buyer_id, seller_id, price_dollars, Qty::new(100), Ts::now()).unwrap();

        assert_eq!(fill1.get_price(), fill2.get_price());
    }

    #[test]
    fn test_fill_event_copy_semantics() {
        let fill_event = FillEvent::new(
            OrderID::new(1).unwrap(),
            OrderID::new(2).unwrap(),
            Px::new_from_dollars(50.0).unwrap(),
            Qty::new(25),
            Ts::now(),
        )
        .unwrap();

        // Copy should work (no move)
        let fill_event_copy = fill_event;
        let fill_event_another_copy = fill_event;

        assert_eq!(
            fill_event_copy.get_buyer_order_id(),
            fill_event_another_copy.get_buyer_order_id()
        );
        assert_eq!(
            fill_event_copy.get_seller_order_id(),
            fill_event_another_copy.get_seller_order_id()
        );
        assert_eq!(
            fill_event_copy.get_price(),
            fill_event_another_copy.get_price()
        );
        assert_eq!(
            fill_event_copy.get_quantity(),
            fill_event_another_copy.get_quantity()
        );
    }

    #[test]
    fn test_fill_event_equality() {
        let timestamp = Ts::from_nanos(1000000);

        let fill1 = FillEvent::new(
            OrderID::new(1).unwrap(),
            OrderID::new(2).unwrap(),
            Px::new_from_dollars(100.0).unwrap(),
            Qty::new(50),
            timestamp,
        )
        .unwrap();

        let fill2 = FillEvent::new(
            OrderID::new(1).unwrap(),
            OrderID::new(2).unwrap(),
            Px::new_from_dollars(100.0).unwrap(),
            Qty::new(50),
            timestamp,
        )
        .unwrap();

        assert_eq!(fill1, fill2);
    }

    #[test]
    fn test_fill_event_inequality_different_ids() {
        let timestamp = Ts::from_nanos(1000000);

        let fill1 = FillEvent::new(
            OrderID::new(1).unwrap(),
            OrderID::new(2).unwrap(),
            Px::new_from_dollars(100.0).unwrap(),
            Qty::new(50),
            timestamp,
        )
        .unwrap();

        let fill2 = FillEvent::new(
            OrderID::new(3).unwrap(),
            OrderID::new(4).unwrap(),
            Px::new_from_dollars(100.0).unwrap(),
            Qty::new(50),
            timestamp,
        )
        .unwrap();

        assert_ne!(fill1, fill2);
    }

    #[test]
    fn test_fill_event_large_values() {
        let fill_event = FillEvent::new(
            OrderID::new(u64::MAX).unwrap(),
            OrderID::new(u64::MAX - 1).unwrap(),
            Px::new_from_cents(u64::MAX).unwrap(),
            Qty::new(u32::MAX),
            Ts::from_nanos(u64::MAX),
        )
        .unwrap();

        assert_eq!(fill_event.get_quantity().value(), u32::MAX);
    }

    // ============================================================================
    // QuoteEvent Tests
    // ============================================================================

    #[test]
    fn test_quote_event_construction_buy_side() {
        let price = Px::new_from_dollars(99.50).unwrap();
        let quantity = Qty::new(200);
        let side = Side::Buy;
        let timestamp = Ts::now();

        let quote = QuoteEvent::new(price, quantity, side, timestamp)
            .expect("Valid QuoteEvent should be created");

        assert_eq!(quote.get_price(), price);
        assert_eq!(quote.get_quantity(), quantity);
        assert_eq!(quote.get_side(), Side::Buy);
        assert_eq!(quote.get_timestamp(), timestamp);
    }

    #[test]
    fn test_quote_event_construction_sell_side() {
        let price = Px::new_from_dollars(100.50).unwrap();
        let quantity = Qty::new(150);
        let side = Side::Sell;
        let timestamp = Ts::now();

        let quote = QuoteEvent::new(price, quantity, side, timestamp)
            .expect("Valid QuoteEvent should be created");

        assert_eq!(quote.get_price(), price);
        assert_eq!(quote.get_quantity(), quantity);
        assert_eq!(quote.get_side(), Side::Sell);
    }

    #[test]
    fn test_quote_event_rejects_zero_quantity() {
        let price = Px::new_from_dollars(100.0).unwrap();
        let zero_quantity = Qty::new(0);
        let side = Side::Buy;
        let timestamp = Ts::now();

        let result = QuoteEvent::new(price, zero_quantity, side, timestamp);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), NyquestroError::InvalidQuantity);
    }

    #[test]
    fn test_quote_event_copy_semantics() {
        let quote = QuoteEvent::new(
            Px::new_from_dollars(75.25).unwrap(),
            Qty::new(100),
            Side::Buy,
            Ts::now(),
        )
        .unwrap();

        let quote_copy1 = quote;
        let quote_copy2 = quote;

        assert_eq!(quote_copy1.get_price(), quote_copy2.get_price());
        assert_eq!(quote_copy1.get_side(), quote_copy2.get_side());
    }

    #[test]
    fn test_quote_event_equality() {
        let timestamp = Ts::from_nanos(2000000);

        let quote1 = QuoteEvent::new(
            Px::new_from_dollars(100.0).unwrap(),
            Qty::new(50),
            Side::Buy,
            timestamp,
        )
        .unwrap();

        let quote2 = QuoteEvent::new(
            Px::new_from_dollars(100.0).unwrap(),
            Qty::new(50),
            Side::Buy,
            timestamp,
        )
        .unwrap();

        assert_eq!(quote1, quote2);
    }

    #[test]
    fn test_quote_event_inequality_different_sides() {
        let timestamp = Ts::from_nanos(2000000);

        let buy_quote = QuoteEvent::new(
            Px::new_from_dollars(100.0).unwrap(),
            Qty::new(50),
            Side::Buy,
            timestamp,
        )
        .unwrap();

        let sell_quote = QuoteEvent::new(
            Px::new_from_dollars(100.0).unwrap(),
            Qty::new(50),
            Side::Sell,
            timestamp,
        )
        .unwrap();

        assert_ne!(buy_quote, sell_quote);
    }

    // ============================================================================
    // OrderEvent Tests
    // ============================================================================

    #[test]
    fn test_order_event_new_construction() {
        let order_id = OrderID::new(123).unwrap();
        let price = Px::new_from_dollars(150.75).unwrap();
        let quantity = Qty::new(75);
        let side = Side::Buy;
        let timestamp = Ts::now();

        let event = OrderEvent::new(order_id, price, quantity, side, timestamp)
            .expect("Valid OrderEvent::New should be created");

        match event {
            OrderEvent::New {
                order_id: id,
                price: p,
                quantity: q,
                side: s,
                timestamp: t,
            } => {
                assert_eq!(id, order_id);
                assert_eq!(p, price);
                assert_eq!(q, quantity);
                assert_eq!(s, side);
                assert_eq!(t, timestamp);
            }
            _ => panic!("Expected OrderEvent::New"),
        }
    }

    #[test]
    fn test_order_event_new_getters() {
        let order_id = OrderID::new(456).unwrap();
        let price = Px::new_from_dollars(200.0).unwrap();
        let quantity = Qty::new(100);
        let side = Side::Sell;
        let timestamp = Ts::now();

        let event = OrderEvent::new(order_id, price, quantity, side, timestamp).unwrap();

        assert_eq!(event.get_order_id(), order_id);
        assert_eq!(event.get_price(), price);
        assert_eq!(event.get_quantity(), quantity);
        assert_eq!(event.get_side(), side);
        assert_eq!(event.get_timestamp(), timestamp);
    }

    #[test]
    fn test_order_event_cancelled_construction() {
        let order_id = OrderID::new(789).unwrap();
        let price = Px::new_from_dollars(50.0).unwrap();
        let quantity = Qty::new(25);
        let side = Side::Buy;
        let timestamp = Ts::now();

        let event = OrderEvent::Cancelled {
            order_id,
            price,
            quantity,
            side,
            timestamp,
        };

        match event {
            OrderEvent::Cancelled {
                order_id: id,
                price: p,
                quantity: q,
                side: s,
                timestamp: t,
            } => {
                assert_eq!(id, order_id);
                assert_eq!(p, price);
                assert_eq!(q, quantity);
                assert_eq!(s, side);
                assert_eq!(t, timestamp);
            }
            _ => panic!("Expected OrderEvent::Cancelled"),
        }
    }

    #[test]
    fn test_order_event_cancelled_getters() {
        let order_id = OrderID::new(999).unwrap();
        let event = OrderEvent::Cancelled {
            order_id,
            price: Px::new_from_dollars(75.5).unwrap(),
            quantity: Qty::new(30),
            side: Side::Sell,
            timestamp: Ts::now(),
        };

        assert_eq!(event.get_order_id(), order_id);
        assert_eq!(event.get_quantity(), Qty::new(30));
        assert_eq!(event.get_side(), Side::Sell);
    }

    #[test]
    fn test_order_event_rejected_all_reasons() {
        let order_id = OrderID::new(111).unwrap();
        let price = Px::new_from_dollars(100.0).unwrap();
        let quantity = Qty::new(50);
        let side = Side::Buy;
        let timestamp = Ts::now();

        let reasons = [
            OrderRejectionReason::InvalidQuantity,
            OrderRejectionReason::InvalidPrice,
            OrderRejectionReason::InvalidSide,
            OrderRejectionReason::InvalidTimestamp,
            OrderRejectionReason::InvalidOrderID,
            OrderRejectionReason::InvalidOrderStatus,
            OrderRejectionReason::InvalidOrderType,
        ];

        for reason in reasons.iter() {
            let event = OrderEvent::Rejected {
                order_id,
                price,
                quantity,
                side,
                reason: *reason,
                timestamp,
            };

            match event {
                OrderEvent::Rejected { reason: r, .. } => assert_eq!(r, *reason),
                _ => panic!("Expected OrderEvent::Rejected"),
            }

            // Test getters work for rejected events
            assert_eq!(event.get_order_id(), order_id);
            assert_eq!(event.get_price(), price);
        }
    }

    #[test]
    fn test_order_event_rejected_getters() {
        let event = OrderEvent::Rejected {
            order_id: OrderID::new(222).unwrap(),
            price: Px::new_from_dollars(125.0).unwrap(),
            quantity: Qty::new(40),
            side: Side::Buy,
            reason: OrderRejectionReason::InvalidPrice,
            timestamp: Ts::now(),
        };

        assert_eq!(event.get_order_id(), OrderID::new(222).unwrap());
        assert_eq!(event.get_quantity(), Qty::new(40));
        assert_eq!(event.get_side(), Side::Buy);
    }

    #[test]
    fn test_order_event_equality_same_variant() {
        let timestamp = Ts::from_nanos(3000000);

        let event1 = OrderEvent::New {
            order_id: OrderID::new(1).unwrap(),
            price: Px::new_from_dollars(100.0).unwrap(),
            quantity: Qty::new(50),
            side: Side::Buy,
            timestamp,
        };

        let event2 = OrderEvent::New {
            order_id: OrderID::new(1).unwrap(),
            price: Px::new_from_dollars(100.0).unwrap(),
            quantity: Qty::new(50),
            side: Side::Buy,
            timestamp,
        };

        assert_eq!(event1, event2);
    }

    #[test]
    fn test_order_event_inequality_different_variants() {
        let order_id = OrderID::new(1).unwrap();
        let price = Px::new_from_dollars(100.0).unwrap();
        let quantity = Qty::new(50);
        let side = Side::Buy;
        let timestamp = Ts::now();

        let new_event = OrderEvent::New {
            order_id,
            price,
            quantity,
            side,
            timestamp,
        };

        let cancelled_event = OrderEvent::Cancelled {
            order_id,
            price,
            quantity,
            side,
            timestamp,
        };

        assert_ne!(new_event, cancelled_event);
    }

    #[test]
    fn test_order_event_copy_semantics() {
        let event = OrderEvent::New {
            order_id: OrderID::new(1).unwrap(),
            price: Px::new_from_dollars(100.0).unwrap(),
            quantity: Qty::new(50),
            side: Side::Buy,
            timestamp: Ts::now(),
        };

        let event_copy1 = event;
        let event_copy2 = event;

        assert_eq!(event_copy1.get_order_id(), event_copy2.get_order_id());
    }

    // ============================================================================
    // Integration Scenarios
    // ============================================================================

    #[test]
    fn test_event_sequence_order_lifecycle() {
        let order_id = OrderID::new(1000).unwrap();
        let price = Px::new_from_dollars(100.0).unwrap();
        let quantity = Qty::new(100);
        let side = Side::Buy;
        let timestamp1 = Ts::from_nanos(1000000);

        // 1. Order created
        let new_event = OrderEvent::new(order_id, price, quantity, side, timestamp1).unwrap();
        assert!(matches!(new_event, OrderEvent::New { .. }));

        // 2. Order partially filled
        let timestamp2 = Ts::from_nanos(2000000);
        let fill_event = FillEvent::new(
            order_id,
            OrderID::new(2000).unwrap(), // seller
            price,
            Qty::new(50), // partial fill
            timestamp2,
        )
        .unwrap();
        assert_eq!(fill_event.get_quantity(), Qty::new(50));

        // 3. Order cancelled
        let timestamp3 = Ts::from_nanos(3000000);
        let cancelled_event = OrderEvent::Cancelled {
            order_id,
            price,
            quantity,
            side,
            timestamp: timestamp3,
        };
        assert!(matches!(cancelled_event, OrderEvent::Cancelled { .. }));

        // Verify timestamps are in order
        assert!(timestamp1 < timestamp2);
        assert!(timestamp2 < timestamp3);
    }

    #[test]
    fn test_multiple_fills_same_orders() {
        let buyer_id = OrderID::new(1).unwrap();
        let seller_id = OrderID::new(2).unwrap();
        let price = Px::new_from_dollars(100.0).unwrap();

        // Multiple partial fills
        let fill1 = FillEvent::new(
            buyer_id,
            seller_id,
            price,
            Qty::new(25),
            Ts::from_nanos(1000000),
        )
        .unwrap();

        let fill2 = FillEvent::new(
            buyer_id,
            seller_id,
            price,
            Qty::new(25),
            Ts::from_nanos(2000000),
        )
        .unwrap();

        assert_eq!(fill1.get_price(), fill2.get_price());
        assert_eq!(fill1.get_buyer_order_id(), fill2.get_buyer_order_id());
        assert_ne!(fill1.get_timestamp(), fill2.get_timestamp());
    }

    #[test]
    fn test_quote_updates_both_sides() {
        let timestamp = Ts::now();
        let price = Px::new_from_dollars(100.0).unwrap();

        let best_bid = QuoteEvent::new(price, Qty::new(200), Side::Buy, timestamp).unwrap();

        let best_ask = QuoteEvent::new(
            Px::new_from_dollars(100.50).unwrap(),
            Qty::new(150),
            Side::Sell,
            timestamp,
        )
        .unwrap();

        assert_eq!(best_bid.get_side(), Side::Buy);
        assert_eq!(best_ask.get_side(), Side::Sell);
        assert!(best_ask.get_price() > best_bid.get_price()); // Ask > Bid
    }

    #[test]
    fn test_event_timestamp_ordering() {
        let base_time = Ts::from_nanos(1000000);

        let event1 = OrderEvent::new(
            OrderID::new(1).unwrap(),
            Px::new_from_dollars(100.0).unwrap(),
            Qty::new(50),
            Side::Buy,
            base_time,
        )
        .unwrap();

        let event2 = FillEvent::new(
            OrderID::new(1).unwrap(),
            OrderID::new(2).unwrap(),
            Px::new_from_dollars(100.0).unwrap(),
            Qty::new(50),
            Ts::from_nanos(2000000),
        )
        .unwrap();

        assert!(event1.get_timestamp() < event2.get_timestamp());
    }

    // ============================================================================
    // Error Handling Integration
    // ============================================================================

    #[test]
    fn test_fill_event_error_propagation() {
        let result = FillEvent::new(
            OrderID::new(1).unwrap(),
            OrderID::new(2).unwrap(),
            Px::new_from_dollars(100.0).unwrap(),
            Qty::new(0), // Invalid
            Ts::now(),
        );

        assert!(result.is_err());
        let error = result.unwrap_err();

        // Verify error can be classified
        let error_severity = severity(&error);
        assert_eq!(error_severity, ErrorSeverity::Recoverable);
    }

    #[test]
    fn test_quote_event_error_propagation() {
        let result = QuoteEvent::new(
            Px::new_from_dollars(100.0).unwrap(),
            Qty::new(0), // Invalid
            Side::Buy,
            Ts::now(),
        );

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error, NyquestroError::InvalidQuantity);
    }

    // ============================================================================
    // Edge Cases and Boundary Conditions
    // ============================================================================

    #[test]
    fn test_fill_event_minimum_quantity() {
        let fill = FillEvent::new(
            OrderID::new(1).unwrap(),
            OrderID::new(2).unwrap(),
            Px::new_from_dollars(100.0).unwrap(),
            Qty::new(1), // Minimum valid quantity
            Ts::now(),
        )
        .unwrap();

        assert_eq!(fill.get_quantity().value(), 1);
    }

    #[test]
    fn test_quote_event_minimum_quantity() {
        let quote = QuoteEvent::new(
            Px::new_from_dollars(100.0).unwrap(),
            Qty::new(1), // Minimum valid quantity
            Side::Buy,
            Ts::now(),
        )
        .unwrap();

        assert_eq!(quote.get_quantity().value(), 1);
    }

    #[test]
    fn test_order_event_with_extreme_values() {
        let event = OrderEvent::new(
            OrderID::new(u64::MAX).unwrap(),
            Px::new_from_cents(u64::MAX).unwrap(),
            Qty::new(u32::MAX),
            Side::Buy,
            Ts::from_nanos(u64::MAX),
        )
        .unwrap();

        assert_eq!(event.get_quantity().value(), u32::MAX);
    }

    #[test]
    fn test_all_event_types_copy_and_clone() {
        let fill = FillEvent::new(
            OrderID::new(1).unwrap(),
            OrderID::new(2).unwrap(),
            Px::new_from_dollars(100.0).unwrap(),
            Qty::new(50),
            Ts::now(),
        )
        .unwrap();

        let quote = QuoteEvent::new(
            Px::new_from_dollars(100.0).unwrap(),
            Qty::new(50),
            Side::Buy,
            Ts::now(),
        )
        .unwrap();

        let order_event = OrderEvent::new(
            OrderID::new(1).unwrap(),
            Px::new_from_dollars(100.0).unwrap(),
            Qty::new(50),
            Side::Buy,
            Ts::now(),
        )
        .unwrap();

        // All should be Copy (no move, can use after assignment)
        let _fill_copy = fill;
        let _fill_another = fill; // Should still work

        let _quote_copy = quote;
        let _quote_another = quote;

        let _order_copy = order_event;
        let _order_another = order_event;
    }
}
