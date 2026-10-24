import { createFixtures } from '../../../packages/domain/index.mjs';
export type Order = ReturnType<typeof createFixtures>['orders'][number];
export type OrderFiltersValue = { search: string; status: string; channel: string; fromDay: number; toDay: number };
