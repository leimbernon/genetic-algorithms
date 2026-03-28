//! Compile-time verification that observer-related types are re-exported at crate root.
//! Covers OBS-01 (TerminationCause, ExtensionEvent) and OBS-02 (NoopObserver).

use genetic_algorithms::NoopObserver;
use genetic_algorithms::ExtensionEvent;
use genetic_algorithms::TerminationCause;

#[test]
fn test_reexport_noop_observer() {
    let _obs = NoopObserver;
    // If this compiles, OBS-02 is satisfied.
}

#[test]
fn test_reexport_extension_event() {
    let _event = ExtensionEvent {
        generation: 0,
        diversity: 0.5,
        extension_type: "MassExtinction",
        threshold: 0.1,
    };
    // If this compiles, OBS-01 (ExtensionEvent) is satisfied.
}

#[test]
fn test_reexport_termination_cause() {
    let cause = TerminationCause::GenerationLimitReached;
    match cause {
        TerminationCause::GenerationLimitReached => {},
        _ => {},
    }
    // If this compiles, OBS-01 (TerminationCause) is satisfied.
}
