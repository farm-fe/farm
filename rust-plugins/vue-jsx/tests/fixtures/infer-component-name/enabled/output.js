import { defineComponent } from 'vue';
const Component1 = defineComponent(()=>{}, {
    name: "Component1"
});
const Component2 = defineComponent(()=>{}, {
    foo: 'bar',
    name: "Component2"
});
const Component3 = defineComponent(()=>{}, {
    name: "Component3",
    ...opts
});
const Component4 = defineComponent(()=>{}, ...args);
