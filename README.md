# zixcel-contracts

Version-pinned JSON contracts exchanged between Zixcel connectors. They define request, plan and execution-result boundaries independently of execution engines and provider implementations.

## Boundaries

- Configuration and events carry only `secret://...` references, never credentials.
- Validation recursively rejects credential keys such as `password`, `access_token` and `private_key`, including arbitrary fields.
- Consumers reference the published contracts as a versioned dependency; bundled JSON Schema validates boundaries until publication.
- Breaking schema URI changes require a new major version.

## Verification

```bash
# WONDERLAND_ROOT is the workspace checkout root.
"$WONDERLAND_ROOT/bin/verify-repositories" --rust --tier standard
```

This gate runs within this crate without connecting to providers.

JSON Schema definitions are in [`schemas/`](schemas/).

`behavior/package/v1` injects registry-sourced implementations into a fixed semantic interface. It carries only provided behaviors, input/output types, effects, dependent behaviors, implemented operation/target/revision and artifact digest. Grammar, semantic definitions, loaders, credentials and local paths cannot be added.

`workspace-action/v1` and `workspace-receipt/v1` place filesystem, search and source control within one workspace authorization boundary. Requests carry only `workspace_ref`, `grant_ref`, normalized relative paths or artifact digests. Absolute local paths, file contents, credentials, IP addresses, transports and process placement are excluded. `WorkspaceActionRequestV1::effect()` provides callers with a closed effect classification for preflight checks.

## Library use

The crate contains no provider implementation and can be used independently by a JSON-consuming process.

```rust
use zixcel_contracts::{Validate, parse_request_json};

let request = parse_request_json(input_bytes)?;
request.validate()?;
```

`parse_request_json` applies a 1 MiB limit, `deny_unknown_fields` and recursive identifier, secret-reference and extended-JSON validation at one entry point. The crate publishes only to `zixcel-private`; schemas and crate are pinned to the same version and archive checksum.

`owner-recovery-request/v1` declares the seed phrase as the primary recovery method but never carries phrases or derived keys. Zixcel composes recovery procedures and user choices; only the designated Crowsi custody package generates, verifies and stores secret material. Movement receipts default to 730 days of retention, owner-configurable from one second to ten years.

## License

Apache-2.0. Copyright 2026 HAT Inc. See [LICENSE](LICENSE) and [NOTICE](NOTICE). External dependencies retain their respective licenses.

## Package integration

The package is an independently consumable unit. Callers reference its documented
interface through a versioned dependency and own application-specific composition
and integration.
