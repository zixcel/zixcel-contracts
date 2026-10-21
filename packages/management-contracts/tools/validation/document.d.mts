/**
 * Protects the stable identity and fail-closed root of one published contract.
 */
import { type Finding } from './types.mjs';
export declare function inspectDocument(source: string, subject: string): {
    document?: unknown;
    identifier?: string;
    findings: Finding[];
};
