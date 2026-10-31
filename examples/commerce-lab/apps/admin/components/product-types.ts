import { createFixtures } from '../../../packages/domain/index.mjs';
export type Product = ReturnType<typeof createFixtures>['products'][number];
export type ProductFiltersValue = { search: string; category: string; status: string };
