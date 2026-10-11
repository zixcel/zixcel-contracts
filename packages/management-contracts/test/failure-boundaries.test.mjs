/**
 * Locks down defects that must block a schema package release.
 */
import assert from 'node:assert/strict'
import test from 'node:test'
import { validateRepository } from '../tools/validation/repository.mjs'
import { contract, repositoryFixture } from './fixture.mjs'

function codes(result) {
  return result.findings.map(finding => finding.code)
}

test('malformed JSON is reported without hiding the remaining catalog', t => {
  const root = repositoryFixture(t, {
    'broken.schema.json': '{',
    'valid.schema.json': contract('estate://contracts/valid/v1')
  })

  const result = validateRepository(root)

  assert.equal(result.schemaCount, 2)
  assert.ok(codes(result).includes('schema-json-invalid'))
  assert.deepEqual(result.identifiers, ['estate://contracts/valid/v1'])
})

test('schema discovery includes responsibility-based subdirectories', t => {
  const root = repositoryFixture(t, {
    'commands/request.schema.json': contract('estate://contracts/request/v1'),
    'results/receipt.schema.json': contract('estate://contracts/receipt/v1')
  })

  const result = validateRepository(root)

  assert.equal(result.schemaCount, 2)
  assert.deepEqual(result.findings, [])
})

test('duplicate identifiers fail the package boundary', t => {
  const identifier = 'estate://contracts/shared/v1'
  const root = repositoryFixture(t, {
    'first.schema.json': contract(identifier),
    'second.schema.json': contract(identifier)
  })

  const result = validateRepository(root)

  assert.equal(codes(result).filter(code => code === 'schema-id-duplicate').length, 1)
})

test('implicit nested object openness is rejected', t => {
  const root = repositoryFixture(t, {
    'open.schema.json': contract('estate://contracts/open/v1', {
      details: { type: 'object' }
    })
  })

  const result = validateRepository(root)

  assert.ok(codes(result).includes('schema-object-boundary-open'))
})

test('a documented operation-specific extension point is reviewable', t => {
  const root = repositoryFixture(t, {
    'extension.schema.json': contract('estate://contracts/extension/v1', {
      payload: {
        type: 'object',
        'x-estate-open-object-reason': 'A selected operation contract closes this payload.'
      }
    })
  })

  assert.deepEqual(validateRepository(root).findings, [])
})

test('README must list every schema and resolve local links', t => {
  const root = repositoryFixture(
    t,
    { 'unlisted.schema.json': contract('estate://contracts/unlisted/v1') },
    { links: [], extraReadme: '\n[Missing](docs/missing.md)\n' }
  )

  const result = validateRepository(root)

  assert.ok(codes(result).includes('readme-schema-unlisted'))
  assert.ok(codes(result).includes('readme-link-broken'))
})

test('envelope identity must match the published identifier', t => {
  const document = JSON.parse(contract('estate://contracts/envelope/v1'))
  document.properties.schema.const = 'estate://contracts/other/v1'
  const root = repositoryFixture(t, {
    'envelope.schema.json': JSON.stringify(document)
  })

  assert.ok(codes(validateRepository(root)).includes('schema-envelope-id-mismatch'))
})

test('package metadata cannot introduce a runtime dependency', t => {
  const root = repositoryFixture(
    t,
    { 'dependency.schema.json': contract('estate://contracts/dependency/v1') },
    { package: { dependencies: { ajv: '8.0.0' } } }
  )

  assert.ok(codes(validateRepository(root)).includes('package-metadata-invalid'))
})
