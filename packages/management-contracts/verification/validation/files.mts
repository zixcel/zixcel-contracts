// Canonical TypeScript; generated JavaScript remains usable without a TypeScript runtime.
import fs from 'node:fs'
import path from 'node:path'
const maximumSchemaFiles = 256, maximumFileBytes = 1024 * 1024
export function schemaFiles(root: string): string[] {
  const directory = path.join(root, 'schemas'), metadata = fs.lstatSync(directory)
  if (metadata.isSymbolicLink() || !metadata.isDirectory()) throw new Error('schemas must be a regular directory')
  const files: string[] = [], pending = [{ directory, depth: 0 }]
  let entries = 0
  while (pending.length) {
    const current = pending.pop()!
    if (current.depth > 32) throw new Error('schema directory depth limit exceeded')
    const stream = fs.opendirSync(current.directory)
    try {
      let entry: fs.Dirent | null
      while ((entry = stream.readSync()) !== null) {
        if (++entries > 4096) throw new Error('schema directory entry limit exceeded')
        const candidate = path.join(current.directory, entry.name)
        if (entry.isSymbolicLink()) throw new Error(`${candidate} must not be a symbolic link`)
        if (entry.isDirectory()) pending.push({ directory: candidate, depth: current.depth + 1 })
        else if (entry.isFile() && entry.name.endsWith('.json')) {
          if (!entry.name.endsWith('.schema.json')) throw new Error(`${candidate} must use the .schema.json suffix`)
          if (files.length >= maximumSchemaFiles) throw new Error(`schema count must be between 1 and ${maximumSchemaFiles}`)
          files.push(candidate)
        }
      }
    } finally { stream.closeSync() }
  }
  if (!files.length) throw new Error(`schema count must be between 1 and ${maximumSchemaFiles}`)
  return files.sort()
}
export function readBoundedFile(file: string): string {
  const initial = fs.lstatSync(file)
  if (initial.isSymbolicLink() || !initial.isFile()) throw new Error(`${file} must be a regular file`)
  const fd = fs.openSync(file, fs.constants.O_RDONLY | fs.constants.O_NOFOLLOW | fs.constants.O_NONBLOCK)
  try {
    const metadata = fs.fstatSync(fd)
    if (!metadata.isFile()) throw new Error(`${file} must be a regular file`)
    if (metadata.size > maximumFileBytes) throw new Error(`${file} exceeds ${maximumFileBytes} bytes`)
    const bytes = Buffer.alloc(metadata.size + 1)
    const length = fs.readSync(fd, bytes, 0, bytes.length, 0)
    if (length > maximumFileBytes || length !== metadata.size) throw new Error('schema file changed or exceeded its limit')
    return bytes.subarray(0, length).toString('utf8')
  } finally { fs.closeSync(fd) }
}
