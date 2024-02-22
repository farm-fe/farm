(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'index_b2bc.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"21da3aef": function(module, exports, farmRequire, farmDynamicRequire) {
// Default to a dummy "batch" implementation that just runs the callback
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    getBatch: function() {
        return getBatch;
    },
    setBatch: function() {
        return setBatch;
    }
});
function defaultNoopBatch(callback) {
    callback();
}
var batch = defaultNoopBatch; // Allow injecting another batching function later
var setBatch = function setBatch(newBatch) {
    return batch = newBatch;
}; // Supply a getter just to skip dealing with ESM bindings
var getBatch = function getBatch() {
    return batch;
};

},
"4c505b7a": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "useIsomorphicLayoutEffect", {
    enumerable: true,
    get: function() {
        return useIsomorphicLayoutEffect;
    }
});
const _react = farmRequire("a0fc9dfd");
var useIsomorphicLayoutEffect = typeof window !== 'undefined' && typeof window.document !== 'undefined' && typeof window.document.createElement !== 'undefined' ? _react.useLayoutEffect : _react.useEffect;

},
"6348e05a": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    createStoreHook: function() {
        return createStoreHook;
    },
    useStore: function() {
        return useStore;
    }
});
const _react = farmRequire("a0fc9dfd");
const _Context = farmRequire("f63d8fd9");
const _useReduxContext = farmRequire("d0d8a3f1");
function createStoreHook(context) {
    if (context === void 0) {
        context = _Context.ReactReduxContext;
    }
    var useReduxContext = context === _Context.ReactReduxContext ? _useReduxContext.useReduxContext : function() {
        return (0, _react.useContext)(context);
    };
    return function useStore() {
        var _useReduxContext = useReduxContext(), store = _useReduxContext.store;
        return store;
    };
}
var useStore = /*#__PURE__*/ createStoreHook();

},
"971db7df": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    createDispatchHook: function() {
        return createDispatchHook;
    },
    useDispatch: function() {
        return useDispatch;
    }
});
const _Context = farmRequire("f63d8fd9");
const _useStore = farmRequire("6348e05a");
function createDispatchHook(context) {
    if (context === void 0) {
        context = _Context.ReactReduxContext;
    }
    var useStore = context === _Context.ReactReduxContext ? _useStore.useStore : (0, _useStore.createStoreHook)(context);
    return function useDispatch() {
        var store = useStore();
        return store.dispatch;
    };
}
var useDispatch = /*#__PURE__*/ createDispatchHook();

},
"a1ea4c25": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    Provider: function() {
        return _Provider.default;
    },
    useDispatch: function() {
        return _useDispatch.useDispatch;
    },
    useSelector: function() {
        return _useSelector.useSelector;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _Provider = /*#__PURE__*/ _interop_require_default._(farmRequire("cfef4295"));
farmRequire("f63d8fd9");
const _useDispatch = farmRequire("971db7df");
const _useSelector = farmRequire("f7f337d0");

},
"b0653cdf": function(module, exports, farmRequire, farmDynamicRequire) {
/* eslint-disable import/no-unresolved */ "use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "unstable_batchedUpdates", {
    enumerable: true,
    get: function() {
        return _reactdom.unstable_batchedUpdates;
    }
});
const _reactdom = farmRequire("3d501ffc");

},
"cfef4295": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _interop_require_wildcard = farmRequire("@swc/helpers/_/_interop_require_wildcard");
const _react = /*#__PURE__*/ _interop_require_wildcard._(farmRequire("a0fc9dfd"));
const _proptypes = /*#__PURE__*/ _interop_require_default._(farmRequire("75a1a40c"));
const _Context = farmRequire("f63d8fd9");
const _Subscription = farmRequire("e645cbf8");
const _useIsomorphicLayoutEffect = farmRequire("4c505b7a");
function Provider(_ref) {
    var store = _ref.store, context = _ref.context, children = _ref.children;
    var contextValue = (0, _react.useMemo)(function() {
        var subscription = (0, _Subscription.createSubscription)(store);
        return {
            store: store,
            subscription: subscription
        };
    }, [
        store
    ]);
    var previousState = (0, _react.useMemo)(function() {
        return store.getState();
    }, [
        store
    ]);
    (0, _useIsomorphicLayoutEffect.useIsomorphicLayoutEffect)(function() {
        var subscription = contextValue.subscription;
        subscription.onStateChange = subscription.notifyNestedSubs;
        subscription.trySubscribe();
        if (previousState !== store.getState()) {
            subscription.notifyNestedSubs();
        }
        return function() {
            subscription.tryUnsubscribe();
            subscription.onStateChange = null;
        };
    }, [
        contextValue,
        previousState
    ]);
    var Context = context || _Context.ReactReduxContext;
    return /*#__PURE__*/ _react.default.createElement(Context.Provider, {
        value: contextValue
    }, children);
}
if ("production" !== 'production') {
    Provider.propTypes = {
        store: _proptypes.default.shape({
            subscribe: _proptypes.default.func.isRequired,
            dispatch: _proptypes.default.func.isRequired,
            getState: _proptypes.default.func.isRequired
        }),
        context: _proptypes.default.object,
        children: _proptypes.default.any
    };
}
const _default = Provider;

},
"d0d8a3f1": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "useReduxContext", {
    enumerable: true,
    get: function() {
        return useReduxContext;
    }
});
const _react = farmRequire("a0fc9dfd");
const _Context = farmRequire("f63d8fd9");
function useReduxContext() {
    var contextValue = (0, _react.useContext)(_Context.ReactReduxContext);
    if ("production" !== 'production' && !contextValue) {
        throw new Error('could not find react-redux context value; please ensure the component is wrapped in a <Provider>');
    }
    return contextValue;
}

},
"e429bf23": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _export_star = farmRequire("@swc/helpers/_/_export_star");
_export_star._(farmRequire("a1ea4c25"), exports);
const _reactBatchedUpdates = farmRequire("b0653cdf");
const _batch = farmRequire("21da3aef");
// with standard React renderers (ReactDOM, React Native)
(0, _batch.setBatch)(_reactBatchedUpdates.unstable_batchedUpdates);

},
"e645cbf8": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "createSubscription", {
    enumerable: true,
    get: function() {
        return createSubscription;
    }
});
const _batch = farmRequire("21da3aef");
// well as nesting subscriptions of descendant components, so that we can ensure the
// ancestor components re-render before descendants
function createListenerCollection() {
    var batch = (0, _batch.getBatch)();
    var first = null;
    var last = null;
    return {
        clear: function clear() {
            first = null;
            last = null;
        },
        notify: function notify() {
            batch(function() {
                var listener = first;
                while(listener){
                    listener.callback();
                    listener = listener.next;
                }
            });
        },
        get: function get() {
            var listeners = [];
            var listener = first;
            while(listener){
                listeners.push(listener);
                listener = listener.next;
            }
            return listeners;
        },
        subscribe: function subscribe(callback) {
            var isSubscribed = true;
            var listener = last = {
                callback: callback,
                next: null,
                prev: last
            };
            if (listener.prev) {
                listener.prev.next = listener;
            } else {
                first = listener;
            }
            return function unsubscribe() {
                if (!isSubscribed || first === null) return;
                isSubscribed = false;
                if (listener.next) {
                    listener.next.prev = listener.prev;
                } else {
                    last = listener.prev;
                }
                if (listener.prev) {
                    listener.prev.next = listener.next;
                } else {
                    first = listener.next;
                }
            };
        }
    };
}
var nullListeners = {
    notify: function notify() {},
    get: function get() {
        return [];
    }
};
function createSubscription(store, parentSub) {
    var unsubscribe;
    var listeners = nullListeners;
    function addNestedSub(listener) {
        trySubscribe();
        return listeners.subscribe(listener);
    }
    function notifyNestedSubs() {
        listeners.notify();
    }
    function handleChangeWrapper() {
        if (subscription.onStateChange) {
            subscription.onStateChange();
        }
    }
    function isSubscribed() {
        return Boolean(unsubscribe);
    }
    function trySubscribe() {
        if (!unsubscribe) {
            unsubscribe = parentSub ? parentSub.addNestedSub(handleChangeWrapper) : store.subscribe(handleChangeWrapper);
            listeners = createListenerCollection();
        }
    }
    function tryUnsubscribe() {
        if (unsubscribe) {
            unsubscribe();
            unsubscribe = undefined;
            listeners.clear();
            listeners = nullListeners;
        }
    }
    var subscription = {
        addNestedSub: addNestedSub,
        notifyNestedSubs: notifyNestedSubs,
        handleChangeWrapper: handleChangeWrapper,
        isSubscribed: isSubscribed,
        trySubscribe: trySubscribe,
        tryUnsubscribe: tryUnsubscribe,
        getListeners: function getListeners() {
            return listeners;
        }
    };
    return subscription;
}

},
"f63d8fd9": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "ReactReduxContext", {
    enumerable: true,
    get: function() {
        return ReactReduxContext;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
var ReactReduxContext = /*#__PURE__*/ _react.default.createContext(null);
if ("production" !== 'production') {
    ReactReduxContext.displayName = 'ReactRedux';
}

},
"f7f337d0": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    createSelectorHook: function() {
        return createSelectorHook;
    },
    useSelector: function() {
        return useSelector;
    }
});
const _react = farmRequire("a0fc9dfd");
const _useReduxContext = farmRequire("d0d8a3f1");
const _Subscription = farmRequire("e645cbf8");
const _useIsomorphicLayoutEffect = farmRequire("4c505b7a");
const _Context = farmRequire("f63d8fd9");
var refEquality = function refEquality(a, b) {
    return a === b;
};
function useSelectorWithStoreAndSubscription(selector, equalityFn, store, contextSub) {
    var _useReducer = (0, _react.useReducer)(function(s) {
        return s + 1;
    }, 0), forceRender = _useReducer[1];
    var subscription = (0, _react.useMemo)(function() {
        return (0, _Subscription.createSubscription)(store, contextSub);
    }, [
        store,
        contextSub
    ]);
    var latestSubscriptionCallbackError = (0, _react.useRef)();
    var latestSelector = (0, _react.useRef)();
    var latestStoreState = (0, _react.useRef)();
    var latestSelectedState = (0, _react.useRef)();
    var storeState = store.getState();
    var selectedState;
    try {
        if (selector !== latestSelector.current || storeState !== latestStoreState.current || latestSubscriptionCallbackError.current) {
            var newSelectedState = selector(storeState); // ensure latest selected state is reused so that a custom equality function can result in identical references
            if (latestSelectedState.current === undefined || !equalityFn(newSelectedState, latestSelectedState.current)) {
                selectedState = newSelectedState;
            } else {
                selectedState = latestSelectedState.current;
            }
        } else {
            selectedState = latestSelectedState.current;
        }
    } catch (err) {
        if (latestSubscriptionCallbackError.current) {
            err.message += "\nThe error may be correlated with this previous error:\n" + latestSubscriptionCallbackError.current.stack + "\n\n";
        }
        throw err;
    }
    (0, _useIsomorphicLayoutEffect.useIsomorphicLayoutEffect)(function() {
        latestSelector.current = selector;
        latestStoreState.current = storeState;
        latestSelectedState.current = selectedState;
        latestSubscriptionCallbackError.current = undefined;
    });
    (0, _useIsomorphicLayoutEffect.useIsomorphicLayoutEffect)(function() {
        function checkForUpdates() {
            try {
                var newStoreState = store.getState(); // Avoid calling selector multiple times if the store's state has not changed
                if (newStoreState === latestStoreState.current) {
                    return;
                }
                var _newSelectedState = latestSelector.current(newStoreState);
                if (equalityFn(_newSelectedState, latestSelectedState.current)) {
                    return;
                }
                latestSelectedState.current = _newSelectedState;
                latestStoreState.current = newStoreState;
            } catch (err) {
                // we ignore all errors here, since when the component
                // is re-rendered, the selectors are called again, and
                // will throw again, if neither props nor store state
                // changed
                latestSubscriptionCallbackError.current = err;
            }
            forceRender();
        }
        subscription.onStateChange = checkForUpdates;
        subscription.trySubscribe();
        checkForUpdates();
        return function() {
            return subscription.tryUnsubscribe();
        };
    }, [
        store,
        subscription
    ]);
    return selectedState;
}
function createSelectorHook(context) {
    if (context === void 0) {
        context = _Context.ReactReduxContext;
    }
    var useReduxContext = context === _Context.ReactReduxContext ? _useReduxContext.useReduxContext : function() {
        return (0, _react.useContext)(context);
    };
    return function useSelector(selector, equalityFn) {
        if (equalityFn === void 0) {
            equalityFn = refEquality;
        }
        if ("production" !== 'production') {
            if (!selector) {
                throw new Error("You must pass a selector to useSelector");
            }
            if (typeof selector !== 'function') {
                throw new Error("You must pass a function as a selector to useSelector");
            }
            if (typeof equalityFn !== 'function') {
                throw new Error("You must pass a function as an equality function to useSelector");
            }
        }
        var _useReduxContext = useReduxContext(), store = _useReduxContext.store, contextSub = _useReduxContext.subscription;
        var selectedState = useSelectorWithStoreAndSubscription(selector, equalityFn, store, contextSub);
        (0, _react.useDebugValue)(selectedState);
        return selectedState;
    };
}
var useSelector = /*#__PURE__*/ createSelectorHook();

},});