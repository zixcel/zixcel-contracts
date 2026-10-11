/**
 * Proves the shipped package satisfies its standalone release contract.
 */
import assert from 'node:assert/strict'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'
import { validateRepository } from '../tools/validation/repository.mjs'

const repositoryRoot = path.resolve(fileURLToPath(new URL('..', import.meta.url)))

test('bundled schemas have unique identities and documented boundaries', () => {
  const result = validateRepository(repositoryRoot)

  assert.deepEqual(result.findings, [])
  assert.equal(result.schemaCount, 7)
  assert.equal(result.identifiers.length, result.schemaCount)
  assert.deepEqual(result.identifiers, [...result.identifiers].sort())
})
