import { computed, defineComponent, ref } from 'vue';
import { createFixtures, queryProducts, updateProduct } from '../../packages/domain/index.mjs';

const StockBadge = defineComponent({
  props: { stock: { type: Number, required: true } },
  setup: props => () => <span class={{ low: props.stock < 25 }}>{props.stock < 25 ? 'Low stock' : 'Healthy'}</span>,
});

export default defineComponent({
  name: 'MerchantInventory',
  setup() {
    const state = ref(createFixtures(42));
    const tenant = ref('north');
    const role = ref('admin');
    const search = ref('');
    const lowOnly = ref(false);
    const products = computed(() => queryProducts(state.value, tenant.value, search.value).filter(product => !lowOnly.value || product.stock < 25));
    function receive(id: string, stock: number) {
      if (role.value !== 'admin') return;
      state.value = updateProduct(state.value, tenant.value, id, { stock: stock + 25 });
    }
    return () => <main data-testid="vue-workspace-ready">
      <header><p>Commerce Lab / Vue JSX</p><h1>Merchant inventory</h1></header>
      <section class="controls" aria-label="Inventory controls">
        <label>Merchant<select v-model={tenant.value}>{state.value.tenants.map(merchant => <option key={merchant.id} value={merchant.id}>{merchant.name}</option>)}</select></label>
        <label>Role<select v-model={role.value}><option value="admin">Admin</option><option value="viewer">Viewer</option></select></label>
        <label>Search inventory<input v-model={search.value} placeholder="SKU, name or category" /></label>
        <label><input type="checkbox" v-model={lowOnly.value} />Low stock only</label>
      </section>
      <p aria-live="polite">{products.value.length} products</p>
      <table><thead><tr><th>Product</th><th>SKU</th><th>Available</th><th>Health</th><th>Action</th></tr></thead>
        <tbody>{products.value.map(product => <tr key={product.id} data-testid="inventory-row">
          <td>{product.name}</td><td>{product.sku}</td><td data-testid="stock">{product.stock}</td>
          <td><StockBadge stock={product.stock} /></td>
          <td><button disabled={role.value !== 'admin'} onClick={() => receive(product.id, product.stock)}>Receive 25 units</button></td>
        </tr>)}</tbody>
      </table>
      {products.value.length === 0 && <p role="status">No products match this merchant and search.</p>}
    </main>;
  },
});
