// simple-typescript fixture: utils.ts
// Imports: 1 | Exports: 1
import { User } from './models';

export function helper(path: string): string {
    const _u: User = { name: path };
    return path.replace(/\/$/, '');
}
