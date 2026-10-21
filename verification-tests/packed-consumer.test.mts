import assert from 'node:assert/strict'
import test from 'node:test'
import fs from 'node:fs'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { execFileSync } from 'node:child_process'
function record(value: unknown): Record<string, unknown> {
  assert.ok(value !== null && typeof value === 'object' && !Array.isArray(value)); return value as Record<string, unknown>
}
test('schema archive remains dependency-free and works in an independent offline consumer', () => {
  const directory = fs.mkdtempSync(path.join(tmpdir(), 'management-schema-consumer-'))
  const root = new URL('../packages/management-contracts/', import.meta.url)
  try {
    for (const file of ['user.npmrc', 'global.npmrc']) fs.writeFileSync(path.join(directory, file), '')
    const env = { ...process.env, NODE_AUTH_TOKEN: '', NPM_TOKEN: '', NPM_CONFIG_USERCONFIG: path.join(directory, 'user.npmrc'), NPM_CONFIG_GLOBALCONFIG: path.join(directory, 'global.npmrc'), NPM_CONFIG_CACHE: path.join(directory, 'cache') }
    const output = execFileSync('npm', ['pack', '--ignore-scripts', '--json', '--pack-destination', directory], { cwd: root, encoding: 'utf8', timeout: 30000, maxBuffer: 1048576, env })
    const decoded: unknown = JSON.parse(output)
    const rows: unknown[] = Array.isArray(decoded) ? decoded : Object.values(record(decoded))
    assert.equal(rows.length, 1)
    const packed = record(rows[0]); assert.equal(typeof packed.filename, 'string'); assert.ok(Array.isArray(packed.files))
    const filename = packed.filename as string; assert.equal(path.basename(filename), filename)
    const files = packed.files.map((value: unknown) => { const file = record(value); assert.equal(typeof file.path, 'string'); return file.path as string })
    assert.ok(files.every(file => file.startsWith('schemas/') || ['package.json', 'README.md', 'LICENSE', 'NOTICE'].includes(file)))
    const consumer = path.join(directory, 'consumer'); fs.mkdirSync(consumer)
    fs.writeFileSync(path.join(consumer, 'package.json'), '{"private":true,"type":"module"}')
    execFileSync('npm', ['install', '--ignore-scripts', '--offline', '--no-audit', '--no-fund', path.join(directory, filename)], { cwd: consumer, env, encoding: 'utf8', timeout: 30000, maxBuffer: 1048576 })
    const schemas = files.filter(file => file.endsWith('.schema.json')).map(file => '@zixcel/management-contracts/' + file)
    const code = `const fs = require('node:fs'); for (const name of ${JSON.stringify(schemas)}) { const schema = JSON.parse(fs.readFileSync(require.resolve(name))); if (schema.type !== 'object' || schema.additionalProperties !== false) throw Error('invalid schema'); }`
    execFileSync(process.execPath, ['-e', code], { cwd: consumer, encoding: 'utf8', timeout: 10000, maxBuffer: 1048576 })
    assert.equal(schemas.length, 7)
  } finally { fs.rmSync(directory, { recursive: true, force: true }) }
})
