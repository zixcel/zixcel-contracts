// Iterative, bounded traversal of arbitrary schema values, including caller-created cycles.
import { object, type Finding } from './types.mjs'
export function inspectObjectBoundaries(value: unknown, subject: string, pointer = '$', findings: Finding[] = []): Finding[] {
  const pending = [{ value, pointer, depth: 0 }], seen = new WeakSet<object>()
  let visited = 0
  while (pending.length) {
    const current = pending.pop()!
    if (++visited > 100000 || current.depth > 128) {
      findings.push({ code: 'schema-traversal-limit', subject, message: 'schema traversal budget exceeded' }); break
    }
    if (current.value === null || typeof current.value !== 'object') continue
    if (seen.has(current.value)) {
      findings.push({ code: 'schema-traversal-invalid', subject, message: 'cyclic or shared caller object' }); break
    }
    seen.add(current.value)
    const node = object(current.value)
    if (node) {
      const isObjectSchema = node.type === 'object' || Array.isArray(node.type) && node.type.includes('object') || 'properties' in node
      const reason = node['x-estate-open-object-reason']
      if (isObjectSchema && node.additionalProperties !== false && !(typeof reason === 'string' && reason.trim())) {
        findings.push({ code: 'schema-object-boundary-open', subject, message: `${current.pointer} must set additionalProperties=false or document its extension point` })
      }
    }
    const entries = Object.entries(current.value)
    if (entries.length + pending.length + visited > 100000) {
      findings.push({ code: 'schema-traversal-limit', subject, message: 'schema traversal budget exceeded' }); break
    }
    for (const [key, child] of entries.reverse()) {
      const token = key.replaceAll('~', '~0').replaceAll('/', '~1')
      pending.push({ value: child, pointer: `${current.pointer}/${token}`, depth: current.depth + 1 })
    }
  }
  return findings
}
