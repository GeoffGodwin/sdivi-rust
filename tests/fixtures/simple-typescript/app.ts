// simple-typescript fixture: app.ts
// Imports: 2 | Exports: 1
// Extended in M33: added console.log (logging) and fetch (data_access) calls.
import { helper } from './utils';
import type { User } from './models';

export function run(name: string): User {
    console.log("Starting run");
    const path = helper('/tmp');
    return { name: path + name };
}

export async function fetchUser(id: string): Promise<User> {
    const resp = await fetch(`/api/users/${id}`);
    return { name: id };
}
