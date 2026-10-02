# zixcel-contracts interface reference

Use the [usage guide](getting-started.md) for the first steps. This reference preserves the current interface details and operational limits. Run command examples from the repository root, after preparing the exact declared dependencies and registered configuration.

## Boundaries

- Configuration and events carry only `secret://...` references, never credentials.
- Validation recursively rejects credential keys such as `password`, `access_token` and `private_key`, including arbitrary fields.
- Consumers reference the published contracts as a versioned dependency; bundled JSON Schema validates boundaries until publication.
- Breaking schema URI changes require a new major version.

## Library use

The crate contains no provider implementation and can be used independently by a JSON-consuming process.

```rust
use zixcel_contracts::{Validate, parse_request_json};

let request = parse_request_json(input_bytes)?;
request.validate()?;
```

`parse_request_json` applies a 1 MiB limit, `deny_unknown_fields` and recursive identifier, secret-reference and extended-JSON validation at one entry point. The crate publishes only to `zixcel-private`; schemas and crate are pinned to the same version and archive checksum.

`owner-recovery-request/v1` declares the seed phrase as the primary recovery method but never carries phrases or derived keys. Zixcel composes recovery procedures and user choices; only the designated Crowsi custody package generates, verifies and stores secret material. Movement receipts default to 730 days of retention, owner-configurable from one second to ten years.
