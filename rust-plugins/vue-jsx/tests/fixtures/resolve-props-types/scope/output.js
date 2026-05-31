import { defineComponent } from 'vue';
type Props = {
    foo: string;
};
function scope() {
    type Props = {
        bar: number;
    };
    defineComponent((props: Props)=>{}, {
        props: {
            bar: {
                type: Number,
                required: true
            }
        }
    });
}
defineComponent((props: Props)=>{}, {
    props: {
        foo: {
            type: String,
            required: true
        }
    }
});
