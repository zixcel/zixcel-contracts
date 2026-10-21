/**
 * Protects the stable identity and fail-closed root of one published contract.
 */
import { object } from './types.mjs';
import { inspectObjectBoundaries } from './object-boundaries.mjs';
const jsonSchemaDraft = 'https://json-schema.org/draft/2020-12/schema';
const identifierPattern = /^estate:\/\/contracts\/[a-z][a-z0-9-]*\/v[1-9][0-9]*$/;
function finding(code, subject, message) {
    return { code, subject, message };
}
export function inspectDocument(source, subject) {
    let document;
    try {
        document = JSON.parse(source);
    }
    catch (error) {
        return {
            findings: [finding('schema-json-invalid', subject, 'schema must contain valid JSON')]
        };
    }
    const findings = [];
    const record = object(document);
    if (!record) {
        findings.push(finding('schema-document-invalid', subject, 'schema must be a JSON object'));
        return { document, findings };
    }
    if (record.$schema !== jsonSchemaDraft) {
        findings.push(finding('schema-draft-invalid', subject, `$schema must be ${jsonSchemaDraft}`));
    }
    if (typeof record.$id !== 'string' || !identifierPattern.test(record.$id)) {
        findings.push(finding('schema-id-invalid', subject, '$id must be a versioned estate://contracts identifier'));
    }
    if (record.type !== 'object' || record.additionalProperties !== false) {
        findings.push(finding('schema-root-open', subject, 'root must be an object with additionalProperties=false'));
    }
    if (object(object(record.properties)?.schema)?.const !== record.$id) {
        findings.push(finding('schema-envelope-id-mismatch', subject, 'properties.schema.const must equal $id'));
    }
    inspectObjectBoundaries(document, subject, '$', findings);
    return { document, ...(typeof record.$id === 'string' ? { identifier: record.$id } : {}), findings };
}
