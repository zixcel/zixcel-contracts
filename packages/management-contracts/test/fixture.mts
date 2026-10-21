/**
 * Builds disposable schema repositories without depending on package fixtures.
 */
import fs from 'node:fs'
import type { TestContext } from 'node:test'
import os from 'node:os'
import path from 'node:path'

export function contract(identifier: string, properties: Record<string, unknown> = {}) {
  return JSON.stringify({
    $schema: 'https://json-schema.org/draft/2020-12/schema',
    $id: identifier,
    type: 'object',
    additionalProperties: false,
    required: ['schema'],
    properties: {
      schema: { const: identifier },
      ...properties
    }
  }, null, 2)
}

export function repositoryFixture(t: TestContext, files: Record<string, string>, options: {links?: string[];extraReadme?: string;package?: Record<string, unknown>} = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'estate-contracts-'))
  const schemas = path.join(root, 'schemas')
  fs.mkdirSync(schemas)
  for (const [name, source] of Object.entries(files)) {
    const target = path.join(schemas, name)
    fs.mkdirSync(path.dirname(target), { recursive: true })
    fs.writeFileSync(target, source)
  }
  const links = options.links ?? Object.keys(files)
  const catalog = links
    .map(name => `- [${name}](schemas/${name})`)
    .join('\n')
  fs.writeFileSync(
    path.join(root, 'README.md'),
    `# Fixture contracts\n\n${catalog}\n${options.extraReadme ?? ''}`
  )
  fs.writeFileSync(
    path.join(root, 'package.json'),
    JSON.stringify({
      name: '@zixcel/management-contracts',
      version: '0.0.0-test',
      private: false,
      type: 'module',
      license: 'Apache-2.0',
      packageManager: 'npm@11.12.1',
      files: ['README.md', 'schemas', 'LICENSE', 'NOTICE'],
      publishConfig: { registry: 'https://registry.npmjs.org' },
      repository: { url: 'git+https://github.com/zixcel/zixcel-contracts.git', directory: 'packages/management-contracts' },
      exports: {
        './schemas/*.schema.json': './schemas/*.schema.json'
      },
      scripts: {
        validate: 'fixture',
        test: 'fixture',
        check: 'fixture',
        prepack: 'fixture'
      },
      ...options.package
    })
  )
  t.after(() => fs.rmSync(root, { recursive: true, force: true }))
  return root
}
