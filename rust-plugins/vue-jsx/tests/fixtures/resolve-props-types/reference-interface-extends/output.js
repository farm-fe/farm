import { defineComponent } from 'vue';
export interface A {
    a(): void;
}
export interface B extends A {
    b: boolean;
}
interface C {
    c: string;
}
interface Aliased extends B, C {
    foo: number;
}
defineComponent((props: Aliased)=>{}, {
    props: {
        foo: {
            type: Number,
            required: true
        },
        b: {
            type: Boolean,
            required: true
        },
        a: {
            type: Function,
            required: true
        },
        c: {
            type: String,
            required: true
        }
    }
});
