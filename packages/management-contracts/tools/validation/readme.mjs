/**
 * Keeps the human contract catalog complete and free of broken local links.
 */
import fs from 'node:fs'
import path from 'node:path'
import { readBoundedFile } from './files.mjs'

const markdownLink = /!?\[[^\]]*]\(([^)\s]+)(?:\s+["'][^)]*["'])?\)/g
const externalScheme = /^[a-z][a-z0-9+.-]*:/i

function relativePath(root, file) {
  return path.relative(root, file).split(path.sep).join('/')
}

export function inspectReadme(root, schemas) {
  const file = path.join(root, 'README.md')
  const findings = []
  let source
  try {
    source = readBoundedFile(file)
  } catch (error) {
    return [{
      code: 'readme-unreadable',
      subject: 'README.md',
      message: error.message
    }]
  }
  const links = new Set()
  for (const match of source.matchAll(markdownLink)) {
    const target = match[1]
    if (target.startsWith('#') || externalScheme.test(target)) continue
    let decoded
    try {
      decoded = decodeURIComponent(target.split('#', 1)[0])
    } catch {
      findings.push({
        code: 'readme-link-invalid',
        subject: target,
        message: 'local link contains invalid percent encoding'
      })
      continue
    }
    const resolved = path.resolve(root, decoded)
    const local = relativePath(root, resolved)
    if (local.startsWith('../') || path.isAbsolute(local) || !fs.existsSync(resolved)) {
      findings.push({
        code: 'readme-link-broken',
        subject: target,
        message: 'local link must resolve inside this repository'
      })
      continue
    }
    links.add(local)
  }
  for (const schema of schemas) {
    const local = relativePath(root, schema)
    if (!links.has(local)) {
      findings.push({
        code: 'readme-schema-unlisted',
        subject: local,
        message: 'every published schema must have a README link'
      })
    }
  }
  return findings
}
