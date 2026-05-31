import { defineComponent } from 'vue';
let a = 1;
const A = defineComponent({
  setup(_, { slots }) {
    return () => <span>{slots.default()}</span>;
  },
});

const _a2 = 2;

a = _a2;

a = <A>{a}</A>;
