import { defineComponent } from 'vue';
defineComponent((props: {
    foo: number;
    bar(): void;
    'baz': string;
    get qux(): number;
    (e: 'foo') : void;
    (e: 'bar') : void;
    untyped1;
    get untyped2();
    untyped3();
})=>{}, {
    props: {
        foo: {
            type: Number,
            required: true
        },
        bar: {
            type: Function,
            required: true
        },
        'baz': {
            type: String,
            required: true
        },
        qux: {
            type: Number,
            required: true
        },
        untyped1: {
            type: null,
            required: true
        },
        untyped2: {
            type: null,
            required: true
        },
        untyped3: {
            type: Function,
            required: true
        }
    }
});
