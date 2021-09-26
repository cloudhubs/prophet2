# Prophet v2

## Test
### Run all tests
`cargo test --workspace --locked --all-targets`

### Run tests for a specific workspace
`cargo test -p <workspace_name> --lib`

## Lint
`cargo clippy --workspace --locked --all-targets -- -D clippy::all`

## Format
`cargo fmt --all`