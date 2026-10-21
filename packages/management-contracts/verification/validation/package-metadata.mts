/**
 * Keeps the distributable artifact schema-only and accidental-publish safe.
 */
import path from 'node:path'
import { object, type Finding } from './types.mjs'
import { readBoundedFile } from './files.mjs'

const requiredFiles = ['README.md', 'schemas', 'LICENSE', 'NOTICE']
const dependencyFields = [
  'dependencies',
  'devDependencies',
  'optionalDependencies',
  'peerDependencies'
]

function finding(message: string): Finding {
  return {
    code: 'package-metadata-invalid',
    subject: 'package.json',
    message
  }
}

export function inspectPackageMetadata(root: string): Finding[] {
  let manifest: Record<string, unknown>
  try {
    const parsed: unknown = JSON.parse(readBoundedFile(path.join(root, 'package.json')))
    const decoded = object(parsed)
    if (!decoded) return [finding('package manifest must be an object')]
    manifest = decoded
  } catch (error) {
    return [finding('package manifest could not be decoded')]
  }
  const findings: Finding[] = []
  if (manifest.name !== '@zixcel/management-contracts' || manifest.type !== 'module') {
    findings.push(finding('name and module type must identify the ecosystem schema package'))
  }
  if (manifest.private !== false || manifest.license !== 'Apache-2.0'
    || object(manifest.publishConfig)?.registry !== 'https://registry.npmjs.org'
    || object(manifest.repository)?.url !== 'git+https://github.com/zixcel/zixcel-contracts.git'
    || object(manifest.repository)?.directory !== 'packages/management-contracts') {
    findings.push(finding('publication must identify the reviewed source, license and official registry'))
  }
  if (!/^npm@[0-9]+\.[0-9]+\.[0-9]+$/.test(typeof manifest.packageManager === 'string' ? manifest.packageManager : '')) {
    findings.push(finding('packageManager must pin an exact npm version'))
  }
  const packagedFiles = new Set(Array.isArray(manifest.files) ? manifest.files : [])
  if (
    packagedFiles.size !== requiredFiles.length
    || requiredFiles.some(file => !packagedFiles.has(file))
  ) {
    findings.push(finding('files must publish only schemas, README, license and notices'))
  }
  if (
    object(manifest.exports)?.['./schemas/*.schema.json']
    !== './schemas/*.schema.json'
  ) {
    findings.push(finding('schema exports must preserve versioned filenames'))
  }
  for (const field of dependencyFields) {
    if (manifest[field] !== undefined && (!object(manifest[field]) || Object.keys(object(manifest[field])!).length)) {
      findings.push(finding(`${field} must stay empty in a zero-dependency package`))
    }
  }
  for (const script of ['validate', 'test', 'check', 'prepack']) {
    if (typeof object(manifest.scripts)?.[script] !== 'string') {
      findings.push(finding(`scripts.${script} is required`))
    }
  }
  return findings
}
