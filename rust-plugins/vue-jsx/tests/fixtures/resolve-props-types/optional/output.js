import { defineComponent } from 'vue';
defineComponent((props: {
    foo?: number;
    bar?(): void;
    'baz'?: string;
    untyped1?;
    untyped2?();
})=>{}, {
    props: {
        foo: {
            type: Number,
            required: false
        },
        bar: {
            type: Function,
            required: false
        },
        'baz': {
            type: String,
            required: false
        },
        untyped1: {
            type: null,
            required: false
        },
        untyped2: {
            type: Function,
            required: false
        }
    }
});
