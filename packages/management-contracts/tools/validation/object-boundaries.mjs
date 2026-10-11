/**
 * Makes every object-shaped extension point closed or explicitly reviewed.
 */

function includesObject(type) {
  return type === 'object' || Array.isArray(type) && type.includes('object')
}

function pointerToken(value) {
  return value.replaceAll('~', '~0').replaceAll('/', '~1')
}

export function inspectObjectBoundaries(
  node,
  subject,
  pointer = '$',
  findings = []
) {
  if (Array.isArray(node)) {
    node.forEach((child, index) => {
      inspectObjectBoundaries(child, subject, `${pointer}/${index}`, findings)
    })
    return findings
  }
  if (!node || typeof node !== 'object') return findings
  const isObjectSchema = includesObject(node.type) || 'properties' in node
  const reason = node['x-estate-open-object-reason']
  const reviewedOpenBoundary = typeof reason === 'string' && reason.trim()
  if (isObjectSchema && node.additionalProperties !== false && !reviewedOpenBoundary) {
    findings.push({
      code: 'schema-object-boundary-open',
      subject,
      message: `${pointer} must set additionalProperties=false or document its extension point`
    })
  }
  for (const [key, child] of Object.entries(node)) {
    inspectObjectBoundaries(child, subject, `${pointer}/${pointerToken(key)}`, findings)
  }
  return findings
}
