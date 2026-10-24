<script setup lang="ts">
import { computed, ref } from 'vue';
import { createFixtures, queryProducts, updateProduct } from '../../packages/domain/index.mjs';
import StockBadge from './StockBadge.vue';

const state = ref(createFixtures(42));
const tenant = ref('north');
const role = ref('admin');
const search = ref('');
const lowOnly = ref(false);
const products = computed(() => queryProducts(state.value, tenant.value, search.value).filter(product => !lowOnly.value || product.stock < 25));
const stockTotal = computed(() => products.value.reduce((total, product) => total + product.stock, 0));
function receive(id: string, stock: number) {
  if (role.value !== 'admin') return;
  state.value = updateProduct(state.value, tenant.value, id, { stock: stock + 25 });
}
</script>
<template>
  <main class="inventory-workspace" data-testid="vue-workspace-ready">
    <header><p>Commerce Lab / Vue SFC</p><h1>Merchant inventory</h1><p>Receive deliveries, review stock and keep merchant data separate.</p></header>
    <section class="controls" aria-label="Inventory controls">
      <label>Merchant<select v-model="tenant"><option v-for="merchant in state.tenants" :key="merchant.id" :value="merchant.id">{{ merchant.name }}</option></select></label>
      <label>Role<select v-model="role"><option value="admin">Admin</option><option value="viewer">Viewer</option></select></label>
      <label>Search inventory<input v-model="search" placeholder="SKU, name or category" /></label>
      <label class="checkbox"><input v-model="lowOnly" type="checkbox" />Low stock only</label>
    </section>
    <p aria-live="polite">{{ products.length }} products · {{ stockTotal }} available units</p>
    <div class="table-wrap"><table><thead><tr><th>Product</th><th>SKU</th><th>Available</th><th>Health</th><th>Action</th></tr></thead>
      <tbody><tr v-for="product in products" :key="product.id" data-testid="inventory-row"><td>{{ product.name }}</td><td>{{ product.sku }}</td><td data-testid="stock">{{ product.stock }}</td><td><StockBadge :stock="product.stock" /></td><td><button :disabled="role !== 'admin'" @click="receive(product.id, product.stock)">Receive 25 units</button></td></tr></tbody>
    </table></div>
    <p v-if="products.length === 0" role="status">No products match this merchant and search.</p>
  </main>
</template>
<style scoped>
.inventory-workspace { max-width: 1100px; margin: 32px auto; padding: 32px; font-family: system-ui, sans-serif; color: #263a31; background: #faf8f1; border-radius: 24px; }
h1 { color: #194e3e; font-size: 36px; }
.controls { display: flex; flex-wrap: wrap; gap: 20px; padding: 20px 0; }
label { display: flex; flex-direction: column; gap: 8px; }
.checkbox { flex-direction: row; align-items: center; }
input, select, button { padding: 9px; border: 1px solid #b5c5ba; border-radius: 8px; background: white; }
button { color: white; background: #194e3e; cursor: pointer; }
button:disabled { opacity: .5; cursor: default; }
.table-wrap { overflow: auto; }
table { width: 100%; border-collapse: collapse; text-align: left; }
th, td { padding: 14px 10px; border-bottom: 1px solid #d8ded5; }
</style>
