//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_5314f59b4724231225830e70b3ddb5bb_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_a = farmRequire.i(farmRequire("fa8c9120"));
        console.log(farmRequire.f(_f_a)());
    },
    "fa8c9120": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        function invariant(condition, message) {
            if (condition) return;
            var error = new Error('loadable: ' + message);
            error.framesToPop = 1;
            error.name = 'Invariant Violation';
            throw error;
        }
        var Context = React.createContext();
        var LOADABLE_SHARED = {
            initialChunks: {}
        };
        var STATUS_PENDING = 'PENDING';
        var STATUS_RESOLVED = 'RESOLVED';
        var STATUS_REJECTED = 'REJECTED';
        function resolveConstructor(ctor) {
            if (typeof ctor === 'function') {
                return {
                    requireAsync: ctor,
                    resolve: function resolve() {
                        return undefined;
                    },
                    chunkName: function chunkName() {
                        return undefined;
                    }
                };
            }
            return ctor;
        }
        var withChunkExtractor = function withChunkExtractor(Component) {
            var LoadableWithChunkExtractor = function LoadableWithChunkExtractor(props) {
                return React.createElement(Context.Consumer, null, function(extractor) {
                    return React.createElement(Component, Object.assign({
                        __chunkExtractor: extractor
                    }, props));
                });
            };
            if (Component.displayName) {
                LoadableWithChunkExtractor.displayName = Component.displayName + 'WithChunkExtractor';
            }
            return LoadableWithChunkExtractor;
        };
        var identity = function identity(v) {
            return v;
        };
        function createLoadable(_ref) {
            var _ref$defaultResolveCo = _ref.defaultResolveComponent, defaultResolveComponent = _ref$defaultResolveCo === void 0 ? identity : _ref$defaultResolveCo, _render = _ref.render, onLoad = _ref.onLoad;
            function loadable(loadableConstructor, options) {
                if (options === void 0) {
                    options = {};
                }
                var ctor = resolveConstructor(loadableConstructor);
                var cache = {};
                function _getCacheKey(props) {
                    if (options.cacheKey) {
                        return options.cacheKey(props);
                    }
                    if (ctor.resolve) {
                        return ctor.resolve(props);
                    }
                    return 'static';
                }
                function resolve(module, props, Loadable) {
                    var Component = options.resolveComponent ? options.resolveComponent(module, props) : defaultResolveComponent(module);
                    if (options.resolveComponent && !isValidElementType(Component)) {
                        throw new Error('resolveComponent returned something that is not a React component!');
                    }
                    hoistNonReactStatics(Loadable, Component, {
                        preload: true
                    });
                    return Component;
                }
                var cachedLoad = function cachedLoad(props) {
                    var cacheKey = _getCacheKey(props);
                    var promise = cache[cacheKey];
                    if (!promise || promise.status === STATUS_REJECTED) {
                        promise = ctor.requireAsync(props);
                        promise.status = STATUS_PENDING;
                        cache[cacheKey] = promise;
                        promise.then(function() {
                            promise.status = STATUS_RESOLVED;
                        }, function(error) {
                            console.error('loadable-components: failed to asynchronously load component', {
                                fileName: ctor.resolve(props),
                                chunkName: ctor.chunkName(props),
                                error: error ? error.message : error
                            });
                            promise.status = STATUS_REJECTED;
                        });
                    }
                    return promise;
                };
                var InnerLoadable = function(_React$Component) {
                    _inheritsLoose(InnerLoadable, _React$Component);
                    InnerLoadable.getDerivedStateFromProps = function getDerivedStateFromProps(props, state) {
                        var cacheKey = _getCacheKey(props);
                        return _extends({}, state, {
                            cacheKey: cacheKey,
                            loading: state.loading || state.cacheKey !== cacheKey
                        });
                    };
                    function InnerLoadable(props) {
                        var _this;
                        _this = _React$Component.call(this, props) || this;
                        _this.state = {
                            result: null,
                            error: null,
                            loading: true,
                            cacheKey: _getCacheKey(props)
                        };
                        invariant(!props.__chunkExtractor || ctor.requireSync, 'SSR requires `@loadable/babel-plugin`, please install it');
                        if (props.__chunkExtractor) {
                            if (options.ssr === false) {
                                return _assertThisInitialized(_this);
                            }
                            ctor.requireAsync(props)['catch'](function() {
                                return null;
                            });
                            _this.loadSync();
                            props.__chunkExtractor.addChunk(ctor.chunkName(props));
                            return _assertThisInitialized(_this);
                        }
                        if (options.ssr !== false && (ctor.isReady && ctor.isReady(props) || ctor.chunkName && LOADABLE_SHARED.initialChunks[ctor.chunkName(props)])) {
                            _this.loadSync();
                        }
                        return _this;
                    }
                    var _proto = InnerLoadable.prototype;
                    _proto.componentDidMount = function componentDidMount() {
                        this.mounted = true;
                        var cachedPromise = this.getCache();
                        if (cachedPromise && cachedPromise.status === STATUS_REJECTED) {
                            this.setCache();
                        }
                        if (this.state.loading) {
                            this.loadAsync();
                        }
                    };
                    _proto.componentDidUpdate = function componentDidUpdate(prevProps, prevState) {
                        if (prevState.cacheKey !== this.state.cacheKey) {
                            this.loadAsync();
                        }
                    };
                    _proto.componentWillUnmount = function componentWillUnmount() {
                        this.mounted = false;
                    };
                    _proto.safeSetState = function safeSetState(nextState, callback) {
                        if (this.mounted) {
                            this.setState(nextState, callback);
                        }
                    };
                    _proto.getCacheKey = function getCacheKey() {
                        return _getCacheKey(this.props);
                    };
                    _proto.getCache = function getCache() {
                        return cache[this.getCacheKey()];
                    };
                    _proto.setCache = function setCache(value) {
                        if (value === void 0) {
                            value = undefined;
                        }
                        cache[this.getCacheKey()] = value;
                    };
                    _proto.triggerOnLoad = function triggerOnLoad() {
                        var _this2 = this;
                        if (onLoad) {
                            setTimeout(function() {
                                onLoad(_this2.state.result, _this2.props);
                            });
                        }
                    };
                    _proto.loadSync = function loadSync() {
                        if (!this.state.loading) return;
                        try {
                            var loadedModule = ctor.requireSync(this.props);
                            var result = resolve(loadedModule, this.props, Loadable);
                            this.state.result = result;
                            this.state.loading = false;
                        } catch (error) {
                            console.error('loadable-components: failed to synchronously load component, which expected to be available', {
                                fileName: ctor.resolve(this.props),
                                chunkName: ctor.chunkName(this.props),
                                error: error ? error.message : error
                            });
                            this.state.error = error;
                        }
                    };
                    _proto.loadAsync = function loadAsync() {
                        var _this3 = this;
                        var promise = this.resolveAsync();
                        promise.then(function(loadedModule) {
                            var result = resolve(loadedModule, _this3.props, Loadable);
                            _this3.safeSetState({
                                result: result,
                                loading: false
                            }, function() {
                                return _this3.triggerOnLoad();
                            });
                        })['catch'](function(error) {
                            return _this3.safeSetState({
                                error: error,
                                loading: false
                            });
                        });
                        return promise;
                    };
                    _proto.resolveAsync = function resolveAsync() {
                        var _this$props = this.props, __chunkExtractor = _this$props.__chunkExtractor, forwardedRef = _this$props.forwardedRef, props = _objectWithoutPropertiesLoose(_this$props, [
                            '__chunkExtractor',
                            'forwardedRef'
                        ]);
                        return cachedLoad(props);
                    };
                    _proto.render = function render() {
                        var _this$props2 = this.props, forwardedRef = _this$props2.forwardedRef, propFallback = _this$props2.fallback, __chunkExtractor = _this$props2.__chunkExtractor, props = _objectWithoutPropertiesLoose(_this$props2, [
                            'forwardedRef',
                            'fallback',
                            '__chunkExtractor'
                        ]);
                        var _this$state = this.state, error = _this$state.error, loading = _this$state.loading, result = _this$state.result;
                        if (options.suspense) {
                            var cachedPromise = this.getCache() || this.loadAsync();
                            if (cachedPromise.status === STATUS_PENDING) {
                                throw this.loadAsync();
                            }
                        }
                        if (error) {
                            throw error;
                        }
                        var fallback = propFallback || options.fallback || null;
                        if (loading) {
                            return fallback;
                        }
                        return _render({
                            fallback: fallback,
                            result: result,
                            options: options,
                            props: _extends({}, props, {
                                ref: forwardedRef
                            })
                        });
                    };
                    return InnerLoadable;
                }(React.Component);
                var EnhancedInnerLoadable = withChunkExtractor(InnerLoadable);
                var Loadable = React.forwardRef(function(props, ref) {
                    return React.createElement(EnhancedInnerLoadable, Object.assign({
                        forwardedRef: ref
                    }, props));
                });
                Loadable.displayName = 'Loadable';
                Loadable.preload = function(props) {
                    Loadable.load(props);
                };
                Loadable.load = function(props) {
                    return cachedLoad(props);
                };
                return Loadable;
            }
            function lazy(ctor, options) {
                return loadable(ctor, _extends({}, options, {
                    suspense: true
                }));
            }
            return {
                loadable: loadable,
                lazy: lazy
            };
        }
        function defaultResolveComponent(loadedModule) {
            return loadedModule.__esModule ? loadedModule['default'] : loadedModule['default'] || loadedModule;
        }
        var _createLoadable = createLoadable({
            defaultResolveComponent: defaultResolveComponent,
            render: function render(_ref) {
                var Component = _ref.result, props = _ref.props;
                return React.createElement(Component, props);
            }
        }), loadable = _createLoadable.loadable, lazy = _createLoadable.lazy;
        var _createLoadable$1 = createLoadable({
            onLoad: function onLoad(result, props) {
                if (result && props.forwardedRef) {
                    if (typeof props.forwardedRef === 'function') {
                        props.forwardedRef(result);
                    } else {
                        props.forwardedRef.current = result;
                    }
                }
            },
            render: function render(_ref) {
                var result = _ref.result, props = _ref.props;
                if (props.children) {
                    return props.children(result);
                }
                return null;
            }
        }), loadable$1 = _createLoadable$1.loadable, lazy$1 = _createLoadable$1.lazy;
        var loadable$2 = loadable;
        loadable$2.lib = loadable$1;
        var lazy$2 = lazy;
        lazy$2.lib = lazy$1;
        exports.default = loadable$2;
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");export default __farm_entry__.__esModule && __farm_entry__.default ? __farm_entry__.default : __farm_entry__;