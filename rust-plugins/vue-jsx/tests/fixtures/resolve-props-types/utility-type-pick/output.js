import { defineComponent } from 'vue';
type T = {
    foo: number;
    bar: string;
    baz: boolean;
};
type K = 'foo' | 'bar';
defineComponent((props: Pick<T, K>)=>{}, {
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
