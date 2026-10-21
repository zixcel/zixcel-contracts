export function object(value) {
    return value !== null && typeof value === 'object' && !Array.isArray(value) ? value : undefined;
}
export function message(error) { return error instanceof Error ? error.message : String(error); }
