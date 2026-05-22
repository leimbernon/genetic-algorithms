//! Permutation engine.
//!
//! Provides `PermutateEngine<U>` for exhaustive evaluation of a user-supplied
//! candidate set with a configurable safety gate. Implements `Strategy<U>` for
//! runtime algorithm swapping via `Box<dyn Strategy<U>>`.

pub mod configuration;
pub mod engine;

pub use configuration::PermutateConfiguration;
pub use engine::PermutateEngine;
