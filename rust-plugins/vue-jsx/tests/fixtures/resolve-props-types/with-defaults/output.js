import { mergeDefaults as _mergeDefaults } from "vue";
import { defineComponent } from 'vue';
import { defaults } from './foo';
defineComponent((props: {
    foo?: string;
    bar?: number;
    baz: boolean;
    qux?(): number;
    quux?(): void;
    quuxx?: Promise<string>;
    fred?: string;
} = {
    foo: 'hi',
    qux () {
        return 1;
    },
    ['quux'] () {},
    async quuxx () {
        return await Promise.resolve('hi');
    },
    get fred () {
        return 'fred';
    }
})=>{}, {
    props: {
        foo: {
            type: String,
            required: false,
            default: 'hi'
        },
        bar: {
            type: Number,
            required: false
        },
        baz: {
            type: Boolean,
            required: true
        },
        qux: {
            type: Function,
            required: false,
            default: function() {
                return 1;
            }
        },
        quux: {
            type: Function,
            required: false,
            default: function() {}
        },
        quuxx: {
            type: Promise,
            required: false,
            default: async function() {
                return await Promise.resolve('hi');
            }
        },
        fred: {
            type: String,
            required: false,
            default: ()=>{
                return 'fred';
            }
        }
    }
});
defineComponent((props: {
    foo?: string;
    bar?: number;
    baz: boolean;
} = {
    ...defaults
})=>{}, {
    props: /*#__PURE__*/ _mergeDefaults({
        foo: {
            type: String,
            required: false
        },
        bar: {
            type: Number,
            required: false
        },
        baz: {
            type: Boolean,
            required: true
        }
    }, {
        ...defaults
    })
});
defineComponent((props: {
    foo?: string;
    bar?: number;
    baz: boolean;
} = defaults)=>{}, {
    props: /*#__PURE__*/ _mergeDefaults({
        foo: {
            type: String,
            required: false
        },
        bar: {
            type: Number,
            required: false
        },
        baz: {
            type: Boolean,
            required: true
        }
    }, defaults)
});
defineComponent((props: {
    foo?: () => 'string';
} = {
    ['fo' + 'o'] () {
        return 'foo';
    }
})=>{}, {
    props: /*#__PURE__*/ _mergeDefaults({
        foo: {
            type: Function,
            required: false
        }
    }, {
        ['fo' + 'o'] () {
            return 'foo';
        }
    })
});
