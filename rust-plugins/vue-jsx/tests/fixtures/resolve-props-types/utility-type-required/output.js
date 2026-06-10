import { defineComponent } from 'vue';
type T = {
    foo?: number;
    bar?: string;
};
defineComponent((props: Required<T>)=>{}, {
    props: {
        foo: {
            type: Number,
            required: true
        },
        bar: {
            type: String,
            required: true
        }
    }
});
