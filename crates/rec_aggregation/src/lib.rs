#![cfg_attr(not(test), allow(unused_crate_dependencies))]

// Expose modules so benches can reuse internal logic.
mod recursion;
mod xmss_aggregate;

pub mod bench_api {
    use std::time::Duration;

    // Re-export bench helpers with stable signatures for Criterion benches.
    pub fn bench_recursion() -> Duration {
        // Implemented inside `recursion` module to reuse its internals.
        crate::recursion::bench_recursion()
    }

    pub fn bench_xmss(n: usize, log_lifetime: usize) -> Duration {
        crate::xmss_aggregate::bench_xmss(n, log_lifetime)
    }
}
