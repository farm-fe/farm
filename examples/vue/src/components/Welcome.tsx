import { defineComponent, ref } from 'vue';

export default defineComponent({
  name: 'Welcome',
  setup() {
    const count = ref(0);
    const increment = () => {
      count.value++;
    };
    const show = ref(true);

    return () => (
      <div class="welcome-jsx">
        <p class="jsx-badge">
          Rendered by <code>@farmfe/plugin-vue-jsx</code>
        </p>
        <div class="jsx-card">
          <strong>JSX count: {count.value}</strong>
          <button onClick={increment}>+1</button>
          <button
            v-show={show.value}
            onClick={() => {
              show.value = false;
            }}
          >
            Hide me
          </button>
          {!show.value && <p class="jsx-reveal">v-show directive works!</p>}
        </div>
      </div>
    );
  },
});
