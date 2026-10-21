#!/usr/bin/env node
/**
 * Provides a zero-dependency release gate for this schema-only package.
 *
 * The command reports every contract defect in one pass so a publisher can
 * correct the complete boundary before creating an immutable release.
 */
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { validateRepository } from './validation/repository.mjs'

export function run(root = path.resolve(fileURLToPath(new URL('..', import.meta.url)))) {
  const result = validateRepository(root)
  for (const finding of result.findings) {
    process.stderr.write(`${finding.code}\t${finding.subject}\t${finding.message}\n`)
  }
  if (result.findings.length) {
    process.stderr.write(
      `${result.findings.length} schema contract finding(s) across `
      + `${result.schemaCount} document(s).\n`
    )
    return 1
  }
  process.stdout.write(
    `Validated ${result.schemaCount} JSON Schemas with unique IDs, `
    + 'reviewed object boundaries, zero dependencies, and complete README links.\n'
  )
  return 0
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.exitCode = run(process.argv[2] && path.resolve(process.argv[2]))
}
