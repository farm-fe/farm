import { defineComponent, type SetupContext } from 'vue';
defineComponent((_, ctx: SetupContext<(e: 'foo' | 'bar') => void>)=>{}, {
    emits: [
        "foo",
        "bar"
    ]
});
defineComponent((_, ctx: SetupContext<((e: 'foo' | 'bar') => void) | ((e: 'baz', id: number) => void)>)=>{}, {
    emits: [
        "foo",
        "bar",
        "baz"
    ]
});
defineComponent((_, ctx: SetupContext<{
    (e: 'foo' | 'bar') : void;
    (e: 'baz', id: number) : void;
}>)=>{}, {
    emits: [
        "foo",
        "bar",
        "baz"
    ]
});
