# Verification results

Verified locally on 2026-08-07 before the live testnet deployment.

## Rust

- `cargo fmt --check`: passed
- `cargo test --target aarch64-apple-darwin --lib`: 8 passed, 0 failed
- `cargo clippy --target aarch64-apple-darwin --all-targets -- -D warnings`: passed
- `cargo build --target wasm32-wasip2 --release`: passed
- `wasm-tools validate target/wasm32-wasip2/release/samedaydesk_audit_gate.wasm`: passed

The eight tests cover:

1. a complete intake scores 100;
2. a weak intake returns the ordered remediation plan;
3. every private field is absent from serialized output;
4. unknown JSON fields are rejected;
5. opaque public ids are constrained;
6. impossible percentage values are rejected;
7. the policy version is valid semver;
8. the policy version matches the Cargo package version.

## WASM component

- Size: 149,166 bytes
- SHA-256: `4f53e7bdc04fa4307c8e17d8faefe7d4363026058252c02c3ad5d4c898638918`
- Export: `z:samedaydesk-audit/contracts@0.1.0::plan-audit`
- Terminal3 custom host imports: none
- Network or durable-storage host imports: none

`wasm-tools component wit` shows the standard Rust WASI CLI adapter imports
and the one application export. The contract code does not call the CLI
adapter; it declares no Terminal3 HTTP, socket, storage, logging, or tenant
host interface.

## Live testnet

- Authentication: passed
- Tenant lookup: passed
- Contract registration: passed as contract ID 484
- Version resolution: passed as `0.1.0`
- Real `plan-audit` invocation: passed
- Four-value output leak check: passed

The stable public result is in [`deployment.json`](deployment.json).
