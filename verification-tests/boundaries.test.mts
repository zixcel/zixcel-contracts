import assert from 'node:assert/strict'
import test from 'node:test'
import fs from 'node:fs'
import { tmpdir } from 'node:os'
import { execFileSync } from 'node:child_process'
import path from 'node:path'
import { inspectObjectBoundaries } from '../packages/management-contracts/tools/validation/object-boundaries.mjs'
import { schemaFiles, readBoundedFile } from '../packages/management-contracts/tools/validation/files.mjs'
import { inspectDocument } from '../packages/management-contracts/tools/validation/document.mjs'
function temporary(): string { return fs.mkdtempSync(path.join(tmpdir(), 'contract-boundaries-')) }
test('cyclic and deep schema values refuse without recursive stack failure', () => {
  const cyclic: Record<string, unknown> = {}; cyclic.loop = cyclic
  assert.ok(inspectObjectBoundaries(cyclic, 'synthetic').some(f => f.code === 'schema-traversal-invalid'))
  let value: unknown = true
  for (let i = 0; i < 140; i++) value = { nested: value }
  assert.ok(inspectObjectBoundaries(value, 'synthetic').some(f => f.code === 'schema-traversal-limit'))
})
test('malformed schemas never echo source bytes into findings', () => {
  const result = inspectDocument('private-source-marker malformed', 'synthetic')
  assert.ok(result.findings.some(f => f.code === 'schema-json-invalid'))
  assert.ok(!JSON.stringify(result).includes('private-source-marker'))
})
test('directory depth and file counts refuse during bounded discovery', () => {
  const root = temporary()
  try {
    let current = path.join(root, 'schemas'); fs.mkdirSync(current)
    for (let i = 0; i < 34; i++) { current = path.join(current, 'child'); fs.mkdirSync(current) }
    assert.throws(() => schemaFiles(root), /depth limit/)
    fs.rmSync(path.join(root, 'schemas'), { recursive: true }); fs.mkdirSync(path.join(root, 'schemas'))
    for (let i = 0; i < 257; i++) fs.writeFileSync(path.join(root, 'schemas', `${i}.schema.json`), '{}')
    assert.throws(() => schemaFiles(root), /between 1 and 256/)
  } finally { fs.rmSync(root, { recursive: true, force: true }) }
})
test('bounded file reads refuse symlink substitution', () => {
  const root = temporary()
  try {
    fs.writeFileSync(path.join(root, 'source'), 'valid')
    assert.equal(readBoundedFile(path.join(root, 'source')), 'valid')
    fs.symlinkSync(path.join(root, 'source'), path.join(root, 'alias'))
    assert.throws(() => readBoundedFile(path.join(root, 'alias')))
  } finally { fs.rmSync(root, { recursive: true, force: true }) }
})
test('every generated tool has a canonical typed source and no handwritten implementation remains', () => {
  const root = new URL('../packages/management-contracts/', import.meta.url)
  function names(directory: URL, suffix: string, prefix = ''): string[] {
    return fs.readdirSync(directory, { withFileTypes: true }).flatMap(entry => entry.isDirectory()
      ? names(new URL(entry.name + '/', directory), suffix, prefix + entry.name + '/')
      : entry.name.endsWith(suffix) ? [prefix + entry.name.slice(0, -suffix.length)] : [])
  }
  assert.deepEqual(names(new URL('tools/', root), '.mjs').sort(), names(new URL('verification/', root), '.mts').sort())
  assert.equal(fs.existsSync(new URL('../../scripts/reviewed-npm.mjs', root)), false)
})

test('unclassified handwritten JavaScript entrypoints cannot return', () => {
  const root = new URL('../', import.meta.url)
  const paths = execFileSync('git', ['--no-optional-locks', '-c', 'core.fsmonitor=false', '-c', 'core.hooksPath=/dev/null', 'ls-files', '-z'], { cwd: root, encoding: 'utf8', timeout: 5000, maxBuffer: 1048576 }).split('\0').filter(name => /\.(?:c|m)?js$/.test(name))
  const interop = new Set(['test/repository.test.mjs', 'test/failure-boundaries.test.mjs', 'test/fixture.mjs'].map(name => 'packages/management-contracts/' + name))
  for (const name of paths) {
    assert.ok(name.startsWith('packages/management-contracts/tools/') || interop.has(name), `Unclassified JavaScript: ${name}`)
  }
})
