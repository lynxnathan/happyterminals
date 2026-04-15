//! Property test: `set()` followed by `untracked()` returns last-set value — REACT-01.

#![allow(
    clippy::unwrap_used,
    reason = "proptest collection range `1..64` guarantees non-empty; .last() is safe"
)]

use happyterminals_core::{Signal, create_root};
use proptest::prelude::*;

proptest! {
    #[test]
    fn set_then_get_returns_last_set(values in prop::collection::vec(any::<i32>(), 1..64)) {
        let (sig, owner) = create_root(|| Signal::new(0i32));
        for v in &values {
            sig.set(*v);
        }
        prop_assert_eq!(sig.untracked(), *values.last().unwrap());
        owner.dispose();
    }
}
