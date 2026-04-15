//! Property test: nested batches equivalent to flat batch — REACT-05.

#![allow(
    clippy::unwrap_used,
    reason = "proptest collection range `1..16` guarantees non-empty; .last() is safe"
)]

use happyterminals_core::{Signal, batch, create_root};
use proptest::prelude::*;

proptest! {
    #[test]
    fn nested_batches_equivalent_to_flat(
        flat_values in prop::collection::vec(any::<i32>(), 1..16),
    ) {
        let flat_final = *flat_values.last().unwrap();
        let (sig, owner) = create_root(|| Signal::new(0i32));
        batch(|| {
            batch(|| {
                for v in &flat_values[..flat_values.len() / 2] {
                    sig.set(*v);
                }
                batch(|| {
                    for v in &flat_values[flat_values.len() / 2..] {
                        sig.set(*v);
                    }
                });
            });
        });
        prop_assert_eq!(sig.untracked(), flat_final);
        owner.dispose();
    }
}
