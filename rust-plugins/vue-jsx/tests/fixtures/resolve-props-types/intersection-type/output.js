import { defineComponent } from 'vue';
type Foo = {
    foo: number;
};
type Bar = {
    bar: string;
};
type Baz = {
    bar: string | boolean;
};
defineComponent((props: {
    self: any;
} & Foo & Bar & Baz)=>{}, {
    props: {
        self: {
            type: null,
            required: true
        },
        foo: {
            type: Number,
            required: true
        },
        bar: {
            type: [
                String,
                Boolean
            ],
            required: true
        }
    }
});
