//! Example smoke tests — each test complies with the pattern:
//! build the example to confirm it compiles, then (if `--include-ignored`
//! is passed) run the example with release optimisations for full coverage.

use std::process::Command;

fn cargo_build_example(name: &str) {
    let status = Command::new("cargo")
        .args(["build", "--example", name])
        .status()
        .expect("cargo build should spawn");
    assert!(status.success(), "cargo build --example {name} failed");
}

fn cargo_run_example(name: &str) {
    let status = Command::new("cargo")
        .args(["run", "--example", name, "--release"])
        .status()
        .expect("cargo run should spawn");
    assert!(status.success(), "cargo run --example {name} failed");
}

#[test]
fn moead_dtlz2() {
    // Smoke test: MOEA/D DTLZ2 example builds and runs to completion.
    cargo_build_example("moead_dtlz2");
    cargo_run_example("moead_dtlz2");
}

#[test]
fn de_rastrigin() {
    cargo_build_example("de_rastrigin");
    cargo_run_example("de_rastrigin");
}

#[test]
fn scatter_search() {
    cargo_build_example("scatter_search");
    cargo_run_example("scatter_search");
}

#[test]
fn cellular_ga() {
    cargo_build_example("cellular_ga");
    cargo_run_example("cellular_ga");
}

#[test]
fn alps() {
    cargo_build_example("alps");
    cargo_run_example("alps");
}

#[test]
fn gp_symbolic_regression() {
    cargo_build_example("gp_symbolic_regression");
    cargo_run_example("gp_symbolic_regression");
}
