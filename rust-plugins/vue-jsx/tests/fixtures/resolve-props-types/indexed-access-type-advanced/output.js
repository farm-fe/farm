import { defineComponent } from 'vue';
type K = 'foo' | 'bar';
type T = {
    foo: string;
    bar: number;
};
type S = {
    foo: {
        foo: T[string];
    };
    bar: {
        bar: string;
    };
};
defineComponent((props: S[K])=>{}, {
    props: {
        foo: {
            type: [
                String,
                Number
            ],
            required: true
        },
        bar: {
            type: String,
            required: true
        }
    }
});
