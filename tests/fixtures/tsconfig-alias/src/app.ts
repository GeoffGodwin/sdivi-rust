// tsconfig-alias fixture: app.ts
// Uses @/ alias — resolves to ./src/lib/utils.ts via tsconfig paths
import { helper } from '@/src/lib/utils';
import { greeting } from '~lib';

export function run(): string {
    return helper() + greeting();
}
