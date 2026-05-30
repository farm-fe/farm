import { defineComponent } from 'vue';
interface Aliased {
    foo: number;
}
defineComponent((props: Aliased)=>{}, {
    props: {
        foo: {
            type: Number,
            required: true
        }
    }
});
defineComponent((props: (Aliased))=>{}, {
    props: {
        foo: {
            type: Number,
            required: true
        }
    }
});
