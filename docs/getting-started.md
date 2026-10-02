# Using zixcel-contracts

Give integrations a shared vocabulary for requests and results so callers can validate service boundaries.

## Before you start

This package supplies contracts. A caller selects and configures the adapter that implements them.

## First steps

Run from the repository root:

```sh
cargo test --locked
```

## How to assess the result

- Use versioned request and result types.
- Keep adapter-specific work behind declared interfaces.

A passing source-level check establishes only what that check observes. Keep missing configuration, unavailable services and unverified deployment paths visible.

## Continue reading

[Repository overview](../README.md)
