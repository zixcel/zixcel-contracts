/**
 * Protects the stable identity and fail-closed root of one published contract.
 */
import { inspectObjectBoundaries } from './object-boundaries.mjs'

const jsonSchemaDraft = 'https://json-schema.org/draft/2020-12/schema'
const identifierPattern = /^estate:\/\/contracts\/[a-z][a-z0-9-]*\/v[1-9][0-9]*$/

function finding(code, subject, message) {
  return { code, subject, message }
}

export function inspectDocument(source, subject) {
  let document
  try {
    document = JSON.parse(source)
  } catch (error) {
    return {
      findings: [finding('schema-json-invalid', subject, error.message)]
    }
  }
  const findings = []
  if (!document || Array.isArray(document) || typeof document !== 'object') {
    findings.push(finding('schema-document-invalid', subject, 'schema must be a JSON object'))
    return { document, findings }
  }
  if (document.$schema !== jsonSchemaDraft) {
    findings.push(finding(
      'schema-draft-invalid',
      subject,
      `$schema must be ${jsonSchemaDraft}`
    ))
  }
  if (typeof document.$id !== 'string' || !identifierPattern.test(document.$id)) {
    findings.push(finding(
      'schema-id-invalid',
      subject,
      '$id must be a versioned estate://contracts identifier'
    ))
  }
  if (document.type !== 'object' || document.additionalProperties !== false) {
    findings.push(finding(
      'schema-root-open',
      subject,
      'root must be an object with additionalProperties=false'
    ))
  }
  if (document.properties?.schema?.const !== document.$id) {
    findings.push(finding(
      'schema-envelope-id-mismatch',
      subject,
      'properties.schema.const must equal $id'
    ))
  }
  inspectObjectBoundaries(document, subject, '$', findings)
  return { document, identifier: document.$id, findings }
}
