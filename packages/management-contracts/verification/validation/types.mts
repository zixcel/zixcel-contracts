// Canonical TypeScript; tools/*.mjs are reproducibly generated for zero-dependency Node consumers.
export interface Finding { code: string; subject: string; message: string }
export interface RepositoryResult { schemaCount: number; identifiers: string[]; findings: Finding[] }
export function object(value: unknown): Record<string, unknown> | undefined {
  return value !== null && typeof value === 'object' && !Array.isArray(value) ? value as Record<string, unknown> : undefined
}
export function message(error: unknown): string { return error instanceof Error ? error.message : String(error) }
