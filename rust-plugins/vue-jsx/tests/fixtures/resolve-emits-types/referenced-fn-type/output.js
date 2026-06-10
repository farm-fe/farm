import { defineComponent, type SetupContext } from 'vue';
type Emits = (e: 'foo' | 'bar') => void;
defineComponent((_, ctx: SetupContext<Emits>)=>{}, {
    emits: [
        "foo",
        "bar"
    ]
});
