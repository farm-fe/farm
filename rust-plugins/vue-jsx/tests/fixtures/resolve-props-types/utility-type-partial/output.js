import { defineComponent } from 'vue';
type T = {
    foo: number;
    bar: string;
};
defineComponent((props: Partial<T>)=>{}, {
    props: {
        foo: {
            type: Number,
            required: false
        },
        bar: {
            type: String,
            required: false
        }
    }
});
