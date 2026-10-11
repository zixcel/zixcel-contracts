/**
 * Keeps the distributable artifact schema-only and accidental-publish safe.
 */
import path from 'node:path'
import { readBoundedFile } from './files.mjs'

const requiredFiles = ['README.md', 'schemas', 'LICENSE', 'NOTICE']
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
  if (manifest.name !== '@zixcel/management-contracts' || manifest.type !== 'module') {
    findings.push(finding('name and module type must identify the ecosystem schema package'))
  }
  if (manifest.private !== false || manifest.license !== 'Apache-2.0'
    || manifest.publishConfig?.registry !== 'https://registry.npmjs.org'
    || manifest.repository?.url !== 'git+https://github.com/zixcel/zixcel-contracts.git'
    || manifest.repository?.directory !== 'packages/management-contracts') {
    findings.push(finding('publication must identify the reviewed source, license and official registry'))
  }
  if (!/^npm@[0-9]+\.[0-9]+\.[0-9]+$/.test(manifest.packageManager ?? '')) {
    findings.push(finding('packageManager must pin an exact npm version'))
  }
  const packagedFiles = new Set(manifest.files ?? [])
  if (
    packagedFiles.size !== requiredFiles.length
    || requiredFiles.some(file => !packagedFiles.has(file))
  ) {
    findings.push(finding('files must publish only schemas, README, license and notices'))
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
