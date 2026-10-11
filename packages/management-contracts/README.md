# @wonderland-ecosystem/ecosystem-contracts

Give independently built services common schemas for exchanging management metadata.

## What you can do

- Validate interoperable service messages.
- Keep ownership and disclosure rules explicit at integration boundaries.

## Current scope

This is a schema package. Applications supply runtime implementations and authorization.

## Getting started

Use `npm@11.12.1` and the Node.js version declared in `engines` in `package.json`. Run from this repository:

```sh
npm install
npm run check
npm run test
```

## Examples and interface details

## Contract catalog

- [Tool command](schemas/tool-command-v1.schema.json) — bounded request envelope
  for validation, planning, simulation, and observation.
- [Tool result](schemas/tool-result-v1.schema.json) — content-free normalized
  result envelope shared by independently executed tools.
- [Connector plan](schemas/connector-plan-v1.schema.json) — non-mutating
  provider plan that carries only an opaque secret reference.
- [Local observation](schemas/local-observation-v1.schema.json) — aggregate
  filesystem counts without paths or content.
- [Workflow input](schemas/workflow-input-v1.schema.json) — acquisition-to-
  workflow handoff containing identity, classification, size, and digest only.
- [Workflow plan](schemas/workflow-plan-v1.schema.json) — deterministic,
  non-executable ordered workflow plan.
- [Ecosystem handoff report](schemas/ecosystem-handoff-report-v1.schema.json) —
  correlated local simulation outcome without raw stage output.

Every document uses JSON Schema Draft 2020-12, a unique versioned `$id`, and a
closed root object. Nested objects are also closed unless a reviewed
`x-estate-open-object-reason` marks an intentional extension point. The generic
command `payload` and result `result` slots are the only current extension
points; the selected operation contract must validate them before use.

## Consumer use case

A consumer pins a released package version, resolves a named schema export,
then verifies the `$id` before compiling or applying it with its own validator:

```js
import { readFile } from 'node:fs/promises'

const location = import.meta.resolve(
  '@wonderland-ecosystem/ecosystem-contracts/schemas/workflow-input-v1.schema.json'
)
const contract = JSON.parse(await readFile(new URL(location), 'utf8'))

if (contract.$id !== 'estate://contracts/workflow-input/v1') {
  throw new Error('unexpected workflow input contract')
}
```

During the unpublished local-foundation phase, a consumer may instead read an
explicitly configured file in this repository. It must still pin the `$id`;
repository-relative source imports are not a runtime dependency contract.

## Documentation and source

[Interface reference](docs/interface-reference.md)

[Usage guide](docs/getting-started.md)

[Schemas](schemas) · [Verification cases](test) · [Contributing](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [License](LICENSE) · [Attribution notices](NOTICE)
