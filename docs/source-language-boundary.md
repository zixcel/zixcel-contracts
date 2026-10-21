# Canonical verification source and generated consumers

The schema validator's canonical implementation is strict TypeScript under
`packages/management-contracts/verification`. `npm run build:verification` emits
its JavaScript and declarations into `tools`; CI regenerates and compares those
bytes. This preserves the existing zero-dependency Node CLI and package commands
without treating generated JavaScript as handwritten source or requiring a
TypeScript runtime in schema consumers. New or modified implementations belong
in the typed source. Emission is refused on type errors.

Tests and fixtures are strict TypeScript, including the consumers that import the generated JavaScript validator. No handwritten JavaScript test exceptions remain. Unknown first-party JavaScript entrypoints are refused.

The old root `reviewed-npm.mjs`, its self-tests and unused review request are
retired. No current workflow, CLI or consumer called that publication path; the
owner's existing version-tag publisher and release admission remain authoritative.

Directory enumeration now streams and bounds entries, depth and file count as
it walks. Schema traversal is iterative with explicit node/depth and cycle
refusals. Reads reject symlinks and non-regular inputs, check an opened descriptor
and bound read bytes. Malformed JSON diagnostics do not echo source contents.
These controls are not a global host-memory or hostile same-user sandbox claim.

Public JSON Schemas, package version, exports and runtime dependencies
are unchanged. The private root npm manifest contains development tooling only;
its dependencies are covered by repository-owned Dependabot. The published
archive still contains only schemas, README, license and notices. Source/tool
acceptance does not publish a new schema version or change repository permissions.

First-party tests and fixtures are TypeScript and included in the strict test typecheck. Only reproducibly generated verifier JavaScript is admitted by the tracked-source guard.
