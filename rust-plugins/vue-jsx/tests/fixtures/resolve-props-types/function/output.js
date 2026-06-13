import { defineComponent } from 'vue';
defineComponent(({}: {
    foo: number;
})=>{}, {
    props: {
        foo: {
            type: Number,
            required: true
        }
    }
});
defineComponent(([]: {
    foo: number;
})=>{}, {
    props: {
        foo: {
            type: Number,
            required: true
        }
    }
});
defineComponent(function(props: {
    foo: number;
}) {}, {
    props: {
        foo: {
            type: Number,
            required: true
        }
    }
});
defineComponent(function({}: {
    foo: number;
}) {}, {
    props: {
        foo: {
            type: Number,
            required: true
        }
    }
});
defineComponent(function([]: {
    foo: number;
}) {}, {
    props: {
        foo: {
            type: Number,
            required: true
        }
    }
});
