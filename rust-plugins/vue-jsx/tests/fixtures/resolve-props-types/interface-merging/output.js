import { defineComponent } from 'vue';
interface Foo {
    a: string;
}
interface Foo {
    b: number;
}
defineComponent((props: {
    foo: Foo['a'];
    bar: Foo['b'];
})=>{}, {
    props: {
        foo: {
            type: String,
            required: true
        },
        bar: {
            type: Number,
            required: true
        }
    }
});
