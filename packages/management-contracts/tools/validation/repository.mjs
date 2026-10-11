/**
 * Composes file, document, identity, and documentation checks for one release.
 */
import path from 'node:path'
import { inspectDocument } from './document.mjs'
import { readBoundedFile, schemaFiles } from './files.mjs'
import { inspectPackageMetadata } from './package-metadata.mjs'
import { inspectReadme } from './readme.mjs'

function failure(code, subject, error) {
  return {
    code,
    subject,
    message: error instanceof Error ? error.message : String(error)
  }
}

export function validateRepository(root) {
  let schemas
  try {
    schemas = schemaFiles(root)
  } catch (error) {
    return {
      schemaCount: 0,
      identifiers: [],
      findings: [failure('schema-discovery-invalid', 'schemas', error)]
    }
  }
  const findings = []
  const identifiers = new Map()
  for (const file of schemas) {
    const subject = path.relative(root, file).split(path.sep).join('/')
    let inspection
    try {
      inspection = inspectDocument(readBoundedFile(file), subject)
    } catch (error) {
      findings.push(failure('schema-file-unreadable', subject, error))
      continue
    }
    findings.push(...inspection.findings)
    if (!inspection.identifier) continue
    const previous = identifiers.get(inspection.identifier)
    if (previous) {
      findings.push({
        code: 'schema-id-duplicate',
        subject,
        message: `$id duplicates ${previous}`
      })
    } else {
      identifiers.set(inspection.identifier, subject)
    }
  }
  findings.push(...inspectPackageMetadata(root))
  findings.push(...inspectReadme(root, schemas))
  findings.sort((left, right) => (
    left.subject.localeCompare(right.subject)
    || left.code.localeCompare(right.code)
  ))
  return {
    schemaCount: schemas.length,
    identifiers: [...identifiers.keys()].sort(),
    findings
  }
}
