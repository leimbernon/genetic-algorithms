//! PSO engine test binary.
//!
//! Exposes `tests/engines/pso/test_pso.rs` as a standalone `cargo test --test test_pso`
//! target, parallel to how other engine tests are organized.

mod engines {
    mod pso {
        mod test_pso;
    }
}
