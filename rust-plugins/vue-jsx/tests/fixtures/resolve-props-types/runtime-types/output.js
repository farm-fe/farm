import { defineComponent } from 'vue';
type FunctionType = {
    () : void;
};
type ObjectType = {
    foo: string;
};
type MixedType = {
    foo: string;
    () : void;
};
interface FunctionInterface {
    () : void;
}
interface ObjectInterface {
    foo: string;
}
interface MixedInterface {
    foo: string;
    () : void;
}
defineComponent((props: {
    a: string;
    b: number;
    c: boolean;
    d: object;
    e: null;
    f: bigint;
    g: symbol;
    h: any;
    i: () => void;
    j: new() => object;
    k: string[];
    l: Array<string>;
    m: [string, number];
    n: 'literal';
    o: 123;
    p: 123n;
    q: true;
    r: FunctionType;
    s: ObjectType;
    t: MixedType;
    u: FunctionInterface;
    v: ObjectInterface;
    w: MixedInterface;
})=>{}, {
    props: {
        a: {
            type: String,
            required: true
        },
        b: {
            type: Number,
            required: true
        },
        c: {
            type: Boolean,
            required: true
        },
        d: {
            type: Object,
            required: true
        },
        e: {
            type: null,
            required: true
        },
        f: {
            type: BigInt,
            required: true
        },
        g: {
            type: Symbol,
            required: true
        },
        h: {
            type: null,
            required: true
        },
        i: {
            type: Function,
            required: true
        },
        j: {
            type: Function,
            required: true
        },
        k: {
            type: Array,
            required: true
        },
        l: {
            type: Array,
            required: true
        },
        m: {
            type: Array,
            required: true
        },
        n: {
            type: String,
            required: true
        },
        o: {
            type: Number,
            required: true
        },
        p: {
            type: Number,
            required: true
        },
        q: {
            type: Boolean,
            required: true
        },
        r: {
            type: Function,
            required: true
        },
        s: {
            type: Object,
            required: true
        },
        t: {
            type: [
                Object,
                Function
            ],
            required: true
        },
        u: {
            type: Function,
            required: true
        },
        v: {
            type: Object,
            required: true
        },
        w: {
            type: [
                Object,
                Function
            ],
            required: true
        }
    }
});
defineComponent((props: {
    function: Function;
    object: Object;
    set: Set<any>;
    map: Map<any, any>;
    weakSet: WeakSet<any>;
    weakMap: WeakMap<any, any>;
    date: Date;
    promise: Promise<any>;
    error: Error;
    regexp: RegExp;
})=>{}, {
    props: {
        function: {
            type: Function,
            required: true
        },
        object: {
            type: Object,
            required: true
        },
        set: {
            type: Set,
            required: true
        },
        map: {
            type: Map,
            required: true
        },
        weakSet: {
            type: WeakSet,
            required: true
        },
        weakMap: {
            type: WeakMap,
            required: true
        },
        date: {
            type: Date,
            required: true
        },
        promise: {
            type: Promise,
            required: true
        },
        error: {
            type: Error,
            required: true
        },
        regexp: {
            type: RegExp,
            required: true
        }
    }
});
defineComponent((props: {
    partial: Partial<{
        foo: string;
    }>;
    required: Required<{
        foo?: string;
    }>;
    readonly: Readonly<{
        foo: string;
    }>;
    record: Record<string, number>;
    pick: Pick<{
        foo: string;
        bar: number;
    }, 'foo'>;
    omit: Omit<{
        foo: string;
        bar: number;
    }, 'foo'>;
    instance: InstanceType<typeof String>;
})=>{}, {
    props: {
        partial: {
            type: Object,
            required: true
        },
        required: {
            type: Object,
            required: true
        },
        readonly: {
            type: Object,
            required: true
        },
        record: {
            type: Object,
            required: true
        },
        pick: {
            type: Object,
            required: true
        },
        omit: {
            type: Object,
            required: true
        },
        instance: {
            type: Object,
            required: true
        }
    }
});
defineComponent((props: {
    uppercase: Uppercase<'foo'>;
    lowercase: Lowercase<'FOO'>;
    capitalize: Capitalize<'foo'>;
    uncapitalize: Uncapitalize<'FOO'>;
})=>{}, {
    props: {
        uppercase: {
            type: String,
            required: true
        },
        lowercase: {
            type: String,
            required: true
        },
        capitalize: {
            type: String,
            required: true
        },
        uncapitalize: {
            type: String,
            required: true
        }
    }
});
defineComponent((props: {
    parameters: Parameters<() => void>;
    constructorParameters: ConstructorParameters<typeof String>;
})=>{}, {
    props: {
        parameters: {
            type: Array,
            required: true
        },
        constructorParameters: {
            type: Array,
            required: true
        }
    }
});
defineComponent((props: {
    nonNullable: NonNullable<string | null | undefined>;
    exclude: Exclude<string | number | boolean, boolean>;
    extract: Extract<string | number | boolean, boolean>;
    omitThisParameter: OmitThisParameter<(this: string) => void>;
})=>{}, {
    props: {
        nonNullable: {
            type: String,
            required: true
        },
        exclude: {
            type: [
                String,
                Number,
                Boolean
            ],
            required: true
        },
        extract: {
            type: Boolean,
            required: true
        },
        omitThisParameter: {
            type: Function,
            required: true
        }
    }
});
