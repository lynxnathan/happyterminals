//! Criterion bench: `Memo<T>` equality-skip overhead.
//!
//! Gates: every `bench_function` must complete in < 1µs per iteration.
//! Enforced by CI: plan `01.0-05` Task 5 parses
//! `target/criterion/<bench>/base/estimates.json` (criterion's machine-
//! readable output) and fails if `mean.point_estimate >= 1000`
//! (nanoseconds). See VERIFICATION.md §FLAG "Criterion `< 1µs` gate is
//! asserted but not enforced" — this plan makes the gate automated.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use happyterminals_core::{Memo, Signal, create_root};

fn bench_memo_f32(c: &mut Criterion) {
    c.bench_function("memo_f32_eq_skip", |b| {
        let (stuff, _owner) = create_root(|| {
            let s = Signal::new(0.0f32);
            let s_clone = s.clone();
            let m = Memo::new(move || s_clone.get() * 2.0);
            (s, m)
        });
        let (s, m) = stuff;

        b.iter(|| {
            s.set(black_box(0.0));
            black_box(m.get());
        });
    });
}

fn bench_memo_small_struct(c: &mut Criterion) {
    #[derive(Clone, PartialEq)]
    struct Vec3 {
        x: f32,
        y: f32,
        z: f32,
    }

    c.bench_function("memo_vec3_eq_skip", |b| {
        let (stuff, _owner) = create_root(|| {
            let s = Signal::new(Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            });
            let s_clone = s.clone();
            let m = Memo::new(move || {
                let v = s_clone.get();
                Vec3 {
                    x: v.x + 1.0,
                    y: v.y,
                    z: v.z,
                }
            });
            (s, m)
        });
        let (s, m) = stuff;

        b.iter(|| {
            s.set(black_box(Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }));
            black_box(m.get());
        });
    });
}

criterion_group!(benches, bench_memo_f32, bench_memo_small_struct);
criterion_main!(benches);
