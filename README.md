# zixcel-contracts

Versioned, provider-neutral wire contracts for connector requests, execution plans, receipts, workspace actions and owner recovery. Use this library when independent adapters need to exchange validated data without sharing their implementation.

## Install

```toml
[dependencies]
zixcel-contracts = "0.10.1"
```

This dependency uses crates.io; no private registry is required.

## Capabilities

- Parse and validate versioned requests and artifact-backed declarations.
- Represent execution plans and receipts with explicit effect and result types.
- Pass secret references instead of embedding credentials.

## Example

```rust
use zixcel_contracts::{REQUEST_SCHEMA_V1, WORKSPACE_RECEIPT_SCHEMA_V2};

assert!(REQUEST_SCHEMA_V1.ends_with("/v1"));
assert!(WORKSPACE_RECEIPT_SCHEMA_V2.ends_with("/v2"));
```

## Features and requirements

Requires Rust 1.97 or newer. The crate is a reusable library, not a command-line application.

## Boundaries

This library describes and validates contracts. Applications supply transports, authorization, storage and execution; a valid request does not grant permission.

## Development

```sh
cargo fmt --all -- --check
cargo test --locked --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
```

## Documentation and license

[API documentation](https://docs.rs/zixcel-contracts) · [Source](https://github.com/zixcel/zixcel-contracts) · [Usage guide](https://github.com/zixcel/zixcel-contracts/blob/main/docs/getting-started.md)

Apache-2.0. Retain the package LICENSE and NOTICE; see the source repository for security reporting and contribution guidelines.
