# ecosystem-contracts interface reference

Use the [usage guide](getting-started.md) for the first steps. This reference preserves the current interface details and operational limits. Run command examples from the repository root, after preparing the exact declared dependencies and registered configuration.

## Contract catalog

- [Tool command](../schemas/tool-command-v1.schema.json) — bounded request envelope
  for validation, planning, simulation, and observation.
- [Tool result](../schemas/tool-result-v1.schema.json) — content-free normalized
  result envelope shared by independently executed tools.
- [Connector plan](../schemas/connector-plan-v1.schema.json) — non-mutating
  provider plan that carries only an opaque secret reference.
- [Local observation](../schemas/local-observation-v1.schema.json) — aggregate
  filesystem counts without paths or content.
- [Workflow input](../schemas/workflow-input-v1.schema.json) — acquisition-to-
  workflow handoff containing identity, classification, size, and digest only.
- [Workflow plan](../schemas/workflow-plan-v1.schema.json) — deterministic,
  non-executable ordered workflow plan.
- [Ecosystem handoff report](../schemas/ecosystem-handoff-report-v1.schema.json) —
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

if (contract.$id !== 'estate://contracts/workflow-input/v1') {
  throw new Error('unexpected workflow input contract')
}
```

During the unpublished local-foundation phase, a consumer may instead read an
explicitly configured file in this repository. It must still pin the `$id`;
repository-relative source imports are not a runtime dependency contract.

## Release and compatibility

- Treat a released schema file as immutable. Incompatible changes receive a new
  filename and `$id` version.
- Run `npm run check` before packing or publishing through an approved channel.
  `npm pack --dry-run` also invokes the same gate.
- Keep credentials, raw customer content, report bodies, local paths, and
  provider URLs outside these envelopes. `secret_ref` is an opaque reference
  resolved only by an authorized runtime.
- Keep this package private until NERP approves publication and licensing.
  The current package metadata deliberately prevents accidental publication.
