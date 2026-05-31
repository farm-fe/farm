import { defineComponent } from 'vue';
defineComponent((props: {
    foo: number;
})=>{}, {
    props: {
        foo: {
            type: Number,
            required: true
        }
    }
});
// shouldn't be resolved
defineComponent((props: {
    foo: number;
})=>{}, {
    props: {
        bar: {
            type: String
        }
    }
});
