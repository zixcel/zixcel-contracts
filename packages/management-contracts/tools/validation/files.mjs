/**
 * Limits validation to regular, bounded files inside the package boundary.
 */
import fs from 'node:fs'
import path from 'node:path'

const maximumSchemaFiles = 256
const maximumFileBytes = 1024 * 1024

export function schemaFiles(root) {
  const directory = path.join(root, 'schemas')
  const metadata = fs.lstatSync(directory)
  if (metadata.isSymbolicLink() || !metadata.isDirectory()) {
    throw new Error('schemas must be a regular directory')
  }
  const files = collectSchemas(directory).sort()
  if (!files.length || files.length > maximumSchemaFiles) {
    throw new Error(`schema count must be between 1 and ${maximumSchemaFiles}`)
  }
  return files
}

function collectSchemas(directory, files = []) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const candidate = path.join(directory, entry.name)
    if (entry.isSymbolicLink()) {
      throw new Error(`${candidate} must not be a symbolic link`)
    }
    if (entry.isDirectory()) {
      collectSchemas(candidate, files)
    } else if (entry.isFile() && entry.name.endsWith('.json')) {
      if (!entry.name.endsWith('.schema.json')) {
        throw new Error(`${candidate} must use the .schema.json suffix`)
      }
      files.push(candidate)
    }
  }
  return files
}

export function readBoundedFile(file) {
  const metadata = fs.lstatSync(file)
  if (metadata.isSymbolicLink() || !metadata.isFile()) {
    throw new Error(`${file} must be a regular file`)
  }
  if (metadata.size > maximumFileBytes) {
    throw new Error(`${file} exceeds ${maximumFileBytes} bytes`)
  }
  return fs.readFileSync(file, 'utf8')
}
