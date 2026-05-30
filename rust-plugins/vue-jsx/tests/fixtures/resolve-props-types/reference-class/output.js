import { defineComponent } from 'vue';
class Foo {
}
defineComponent((props: {
    foo: Foo;
})=>{}, {
    props: {
        foo: {
            type: Object,
            required: true
        }
    }
});
