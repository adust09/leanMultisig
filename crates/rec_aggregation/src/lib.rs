#![cfg_attr(not(test), allow(unused_crate_dependencies))]

// Expose modules so benches can reuse internal logic.
mod recursion;
pub use recursion::bench_recursion;
mod xmss_aggregate;
