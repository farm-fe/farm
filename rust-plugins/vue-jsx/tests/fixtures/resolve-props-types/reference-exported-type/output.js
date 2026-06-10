import { defineComponent } from 'vue';
export type Aliased = {
    foo: number;
};
defineComponent((props: Aliased)=>{}, {
    props: {
        foo: {
            type: Number,
            required: true
        }
    }
});
