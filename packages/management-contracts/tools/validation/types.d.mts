export interface Finding {
    code: string;
    subject: string;
    message: string;
}
export interface RepositoryResult {
    schemaCount: number;
    identifiers: string[];
    findings: Finding[];
}
export declare function object(value: unknown): Record<string, unknown> | undefined;
export declare function message(error: unknown): string;
