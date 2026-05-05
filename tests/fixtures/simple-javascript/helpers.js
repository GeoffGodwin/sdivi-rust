// simple-javascript fixture: helpers.js
// Imports: 0 | Exports: 1

export function clean(value) {
    return value == null ? '' : String(value).trim();
}
