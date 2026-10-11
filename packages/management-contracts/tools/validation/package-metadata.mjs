/**
 * Keeps the distributable artifact schema-only and accidental-publish safe.
 */
import path from 'node:path'
import { readBoundedFile } from './files.mjs'

const requiredFiles = ['README.md', 'schemas']
const dependencyFields = [
  'dependencies',
  'devDependencies',
  'optionalDependencies',
  'peerDependencies'
]

function finding(message) {
  return {
    code: 'package-metadata-invalid',
    subject: 'package.json',
    message
  }
}

export function inspectPackageMetadata(root) {
  let manifest
  try {
    manifest = JSON.parse(readBoundedFile(path.join(root, 'package.json')))
  } catch (error) {
    return [finding(error.message)]
  }
  const findings = []
  if (manifest.name !== '@wonderland-ecosystem/ecosystem-contracts' || manifest.type !== 'module') {
    findings.push(finding('name and module type must identify the ecosystem schema package'))
  }
  if (manifest.private !== true || manifest.license !== 'UNLICENSED') {
    findings.push(finding('local foundation metadata must prevent accidental publication'))
  }
  if (!/^npm@[0-9]+\.[0-9]+\.[0-9]+$/.test(manifest.packageManager ?? '')) {
    findings.push(finding('packageManager must pin an exact npm version'))
  }
  const packagedFiles = new Set(manifest.files ?? [])
  if (
    packagedFiles.size !== requiredFiles.length
    || requiredFiles.some(file => !packagedFiles.has(file))
  ) {
    findings.push(finding('files must publish only the README and schemas boundary'))
  }
  if (
    manifest.exports?.['./schemas/*.schema.json']
    !== './schemas/*.schema.json'
  ) {
    findings.push(finding('schema exports must preserve versioned filenames'))
  }
  for (const field of dependencyFields) {
    if (Object.keys(manifest[field] ?? {}).length) {
      findings.push(finding(`${field} must stay empty in a zero-dependency package`))
    }
  }
  for (const script of ['validate', 'test', 'check', 'prepack']) {
    if (typeof manifest.scripts?.[script] !== 'string') {
      findings.push(finding(`scripts.${script} is required`))
    }
  }
  return findings
}
