import { defineComponent } from 'vue';
import Button from './button.vue';
import Button1 from './button1.vue';
import Button2 from './button2.vue';
import Button3 from './button3.vue';
import Button4 from './button4.vue';

export default defineComponent({
  name: 'vue-jsx',
  setup() {
    return () => {
      return <div>123456 <Button />
        <Button1 />
        <Button2 />
        <Button3 />
        <Button4 />
      </div>;
    };
  },
});
