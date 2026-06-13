import { defineComponent, type SetupContext } from 'vue';
interface Emits {
    (e: 'foo' | 'bar') : void;
}
defineComponent((_, ctx: SetupContext<Emits>)=>{}, {
    emits: [
        "foo",
        "bar"
    ]
});
