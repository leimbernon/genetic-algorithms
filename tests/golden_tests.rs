/// Golden output regression tests for the four reference examples.
///
/// Each test runs the corresponding example binary with `--seed 42` and
/// asserts that the best-fitness (or Pareto front summary) line in stdout
/// matches the expected value stored in `tests/golden/<name>.txt`.
///
/// These tests use `include_str!` so that a missing .txt file is a compile
/// error rather than a runtime panic.
///
/// NOTE: The rastrigin example fixes rayon to 1 thread when `--seed` is
/// provided, ensuring deterministic RNG counter ordering across repeated runs.
use std::process::Command;

/// Path to the compiled `cargo` binary — reuse the same toolchain.
fn cargo_bin() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string())
}

#[test]
fn golden_rastrigin() {
    let expected = include_str!("golden/rastrigin.txt").trim();

    let output = Command::new(cargo_bin())
        .args([
            "run",
            "--example",
            "rastrigin",
            "--release",
            "--",
            "--seed",
            "42",
        ])
        .output()
        .expect("failed to run rastrigin example");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let captured = stdout
        .lines()
        .find(|l| l.contains("Finished. Best fitness:"))
        .unwrap_or_else(|| panic!("rastrigin stdout did not contain 'Finished. Best fitness:'\nFull stdout:\n{stdout}"));

    assert_eq!(
        captured, expected,
        "rastrigin golden output mismatch:\n  got:      {captured}\n  expected: {expected}"
    );
}

#[test]
fn golden_nsga2_zdt1() {
    let expected = include_str!("golden/nsga2_zdt1.txt").trim();

    let output = Command::new(cargo_bin())
        .args([
            "run",
            "--example",
            "nsga2_zdt1",
            "--release",
            "--",
            "--seed",
            "42",
        ])
        .output()
        .expect("failed to run nsga2_zdt1 example");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let captured = stdout
        .lines()
        .find(|l| l.contains("Completed. Pareto front:"))
        .unwrap_or_else(|| panic!("nsga2_zdt1 stdout did not contain 'Completed. Pareto front:'\nFull stdout:\n{stdout}"));

    assert_eq!(
        captured, expected,
        "nsga2_zdt1 golden output mismatch:\n  got:      {captured}\n  expected: {expected}"
    );
}

#[test]
fn golden_cma_es_rastrigin() {
    let expected = include_str!("golden/cma_es_rastrigin.txt").trim();

    let output = Command::new(cargo_bin())
        .args([
            "run",
            "--example",
            "cma_es_rastrigin",
            "--release",
            "--",
            "--seed",
            "42",
        ])
        .output()
        .expect("failed to run cma_es_rastrigin example");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let captured = stdout
        .lines()
        .find(|l| l.contains("Best fitness:"))
        .unwrap_or_else(|| {
            panic!(
                "cma_es_rastrigin stdout did not contain 'Best fitness:'\nFull stdout:\n{stdout}"
            )
        });

    assert_eq!(
        captured, expected,
        "cma_es_rastrigin golden output mismatch:\n  got:      {captured}\n  expected: {expected}"
    );
}

#[test]
fn golden_pso_rastrigin() {
    let expected = include_str!("golden/pso_rastrigin.txt").trim();

    let output = Command::new(cargo_bin())
        .args([
            "run",
            "--example",
            "pso_rastrigin",
            "--release",
            "--",
            "--seed",
            "42",
        ])
        .output()
        .expect("failed to run pso_rastrigin example");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let captured = stdout
        .lines()
        .find(|l| l.contains("Best fitness:"))
        .unwrap_or_else(|| {
            panic!("pso_rastrigin stdout did not contain 'Best fitness:'\nFull stdout:\n{stdout}")
        });

    assert_eq!(
        captured, expected,
        "pso_rastrigin golden output mismatch:\n  got:      {captured}\n  expected: {expected}"
    );
}
