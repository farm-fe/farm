import { defineComponent, type SetupContext } from 'vue';
defineComponent((_, ctx: SetupContext<{
    foo: [];
    bar: [];
}>)=>{}, {
    emits: [
        "foo",
        "bar"
    ]
});
defineComponent((_, ctx: SetupContext<{
    'foo:bar': [];
}>)=>{}, {
    emits: [
        "foo:bar"
    ]
});
