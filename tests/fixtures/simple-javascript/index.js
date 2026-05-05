// simple-javascript fixture: index.js
// Imports: 2 | Exports: 1
import { format } from './utils';
const helpers = require('./helpers');

export function run(value) {
    return format(helpers.clean(value));
}
