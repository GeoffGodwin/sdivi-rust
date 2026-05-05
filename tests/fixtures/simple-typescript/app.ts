// simple-typescript fixture: app.ts
// Imports: 2 | Exports: 1
import { helper } from './utils';
import type { User } from './models';

export function run(name: string): User {
    const path = helper('/tmp');
    return { name: path + name };
}
