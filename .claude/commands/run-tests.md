Run the full test and quality suite for the genetic_algorithms library.

Execute these commands and report results:

1. **Compile check**: `cargo check 2>&1`
2. **Clippy lint**: `cargo clippy -- -D warnings 2>&1`
3. **Unit tests**: `cargo test 2>&1`
4. **Serde tests**: `cargo test --features serde 2>&1`
5. **Doc check**: `cargo doc --no-deps 2>&1`

For each step, report: PASS/FAIL + summary of any errors.

If any step fails:
- Identify the root cause
- Suggest a fix
- Do NOT proceed to the next step until the current one is fixed

If all pass, confirm the codebase is in a clean state for commit/PR.

Optional argument: `bench` — also run `cargo bench` and report results.
